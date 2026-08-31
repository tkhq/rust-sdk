#!/usr/bin/env bash

set -euo pipefail

target="${1:?missing target triple}"
version="2.5.1"
sha512="027851359b38cf56c2ea97a969c0ae8c5eabbb977fc2a86c650c6c8e0e2caba43934f3de35573deb408bf6365e9deb2a9c443b3712b313692069421c77222057"
meson_version="0.58.2"
meson_sha256="7634ec32955d3f897d623b88e9d2988451035f43d73c17a29caf767387baedb7"

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

archive="/tmp/pcsc-lite-${version}.tar.xz"
source_dir="/tmp/pcsc-lite-${version}"
build_dir="/tmp/pcsc-lite-build"
cross_file="/tmp/pcsc-lite-${target}.ini"
prefix="/opt/pcsc/${target}"

curl \
  --fail \
  --location \
  --retry 5 \
  --output "${archive}" \
  "https://pcsclite.apdu.fr/files/pcsc-lite-${version}.tar.xz"
echo "${sha512}  ${archive}" | sha512sum --check -
tar -xJf "${archive}" -C /tmp

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
  echo '[properties]'
  echo 'needs_exe_wrapper = true'
} > "${cross_file}"

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

rm -rf \
  "${archive}" \
  "${build_dir}" \
  "${cross_file}" \
  "${source_dir}" \
  /var/lib/apt/lists/*
