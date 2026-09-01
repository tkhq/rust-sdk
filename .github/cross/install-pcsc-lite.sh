#!/usr/bin/env bash

set -euo pipefail

# Outcome: install a target-specific static libpcsclite client in the Cross image
# so TVC's Rust PC/SC dependency can be linked into a portable musl binary. Cross
# does not provide that target library, so building it here with Cross's musl
# toolchain keeps its ABI aligned with TVC. The resulting client still talks to
# the host's pcscd at runtime; this image does not supply the runtime daemon.
# See https://github.com/cross-rs/cross/wiki/Configuration

target="${1:?missing target triple}"

# These versions and hashes are supply-chain security controls. Review upstream
# provenance and update each version and digest together; the PC/SC release check
# only alerts when a newer release exists.
version="2.5.1"
sha512="027851359b38cf56c2ea97a969c0ae8c5eabbb977fc2a86c650c6c8e0e2caba43934f3de35573deb408bf6365e9deb2a9c443b3712b313692069421c77222057"
meson_version="0.58.2"
meson_sha256="7634ec32955d3f897d623b88e9d2988451035f43d73c17a29caf767387baedb7"

# Keep the accepted targets explicit so Meson cannot silently select a host
# compiler or wrong machine description. Adding a target requires validating its
# Cross toolchain and Meson machine settings.
# See https://mesonbuild.com/Cross-compilation.html
case "${target}" in
  aarch64-unknown-linux-musl)
    compiler="aarch64-linux-musl-gcc"
    archiver="aarch64-linux-musl-ar"
    stripper="aarch64-linux-musl-strip"
    cpu_family="aarch64"
    cpu="aarch64"
    ;;
  x86_64-unknown-linux-musl)
    compiler="x86_64-linux-musl-gcc"
    archiver="x86_64-linux-musl-ar"
    stripper="x86_64-linux-musl-strip"
    cpu_family="x86_64"
    cpu="x86_64"
    ;;
  *)
    echo "unsupported target: ${target}" >&2
    exit 1
    ;;
esac

export DEBIAN_FRONTEND=noninteractive

# Install the download and build tools explicitly because the pinned Cross images
# do not provide all of them. They are only used to produce libpcsclite;
# --no-install-recommends and the cleanup below keep the derived image smaller.
apt-get update
apt-get install --assume-yes --no-install-recommends \
  ca-certificates \
  curl \
  flex \
  ninja-build \
  perl \
  pkg-config \
  python3-pip \
  python3-setuptools \
  xz-utils

# Install the exact Meson source distribution without resolving dependencies.
# Where pip supports it, disabling build isolation also prevents downloads of
# additional PEP 517 build requirements. This is a supply-chain security control.
# See https://pip.pypa.io/en/stable/topics/secure-installs/
meson_requirements="/tmp/meson-requirements.txt"
printf 'meson==%s --hash=sha256:%s\n' \
  "${meson_version}" \
  "${meson_sha256}" > "${meson_requirements}"
pip_build_options=()
if python3 -m pip install --help |
  awk '/--no-build-isolation/ { found=1 } END { exit !found }'; then
  pip_build_options+=(--no-build-isolation)
fi
python3 -m pip install \
  --no-cache-dir \
  --no-deps \
  --no-binary=:all: \
  --require-hashes \
  "${pip_build_options[@]}" \
  --requirement "${meson_requirements}"
rm -f "${meson_requirements}"

# Keep target artifacts separate from host tools and other targets. Cross.toml
# points pcsc-sys at this prefix through PKG_CONFIG_PATH.
archive="/tmp/pcsc-lite-${version}.tar.xz"
source_dir="/tmp/pcsc-lite-${version}"
build_dir="/tmp/pcsc-lite-build"
cross_file="/tmp/pcsc-lite-${target}.ini"
prefix="/opt/pcsc/${target}"

# Verify the repository-pinned archive bytes before processing any upstream
# source. Changing the URL, version, or digest requires a supply-chain review.
curl \
  --fail \
  --location \
  --retry 5 \
  --output "${archive}" \
  "https://pcsclite.apdu.fr/files/pcsc-lite-${version}.tar.xz"
echo "${sha512}  ${archive}" | sha512sum --check -
tar -xJf "${archive}" -C /tmp

# Meson needs an explicit target toolchain and must not try to run target binaries
# while configuring inside the builder. See the Meson cross-compilation guide:
# https://mesonbuild.com/Cross-compilation.html
{
  echo '[binaries]'
  echo "c = '${compiler}'"
  echo "ar = '${archiver}'"
  echo "strip = '${stripper}'"
  echo
  echo '[host_machine]'
  echo "system = 'linux'"
  echo "cpu_family = '${cpu_family}'"
  echo "cpu = '${cpu}'"
  echo "endian = 'little'"
  echo
  echo '[built-in options]'
  # FORTIFY, stack protection, and format checks harden memory-unsafe C linked
  # into TVC; weakening them requires a security review. -fPIC is required for
  # link compatibility with position-independent outputs.
  # See https://gcc.gnu.org/onlinedocs/gcc/Instrumentation-Options.html
  echo "c_args = ['-D_FORTIFY_SOURCE=2', '-fPIC', '-fstack-protector-strong', '-Wformat', '-Wformat-security', '-Werror=format-security']"
  echo
  echo '[properties]'
  echo 'needs_exe_wrapper = true'
} > "${cross_file}"

# The release build type enables the optimization required by _FORTIFY_SOURCE,
# so changing it is security-sensitive. Static output is a portability requirement:
# TVC must not depend on a target system's shared libpcsclite. The other feature
# flags remove pcscd's daemon, device-discovery, and policy integrations because
# TVC consumes only the client archive. Those flags reduce dependencies and image
# size; they are not security invariants. See the upstream option descriptions:
# https://github.com/LudovicRousseau/PCSC/blob/2.5.1/meson_options.txt
meson setup \
  "${build_dir}" \
  "${source_dir}" \
  --buildtype=release \
  --cross-file "${cross_file}" \
  --libdir=lib \
  --prefix="${prefix}" \
  -Ddefault_library=static \
  -Dlibsystemd=false \
  -Dlibudev=false \
  -Dlibusb=false \
  -Dpolkit=false \
  -Dserial=false \
  -Dusb=false
meson compile -C "${build_dir}"
meson install -C "${build_dir}"

# This only reduces the derived Cross image size; it does not affect the released
# binary's security properties.
rm -rf \
  "${archive}" \
  "${build_dir}" \
  "${cross_file}" \
  "${source_dir}" \
  /var/lib/apt/lists/*
