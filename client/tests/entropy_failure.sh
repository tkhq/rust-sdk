#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
cargo test -p turnkey_client --lib --no-run --message-format=json > "$scratch/build.jsonl"
test_binary=$(python3 -c 'import json,sys; print(next(row["executable"] for line in open(sys.argv[1]) if (row:=json.loads(line)).get("reason")=="compiler-artifact" and row.get("target",{}).get("name")=="turnkey_client" and row.get("profile",{}).get("test") and row.get("executable")))' "$scratch/build.jsonl")
case "$(uname -s)" in
    Darwin)
        cc -dynamiclib client/tests/support/fail_entropy.c -o "$scratch/fail_entropy.dylib"
        TVC572_ENTROPY_FAILURE_MARKER="$scratch/armed" DYLD_INSERT_LIBRARIES="$scratch/fail_entropy.dylib" "$test_binary" --exact receipts::tests::actual_os_entropy_failure_tracks_first_and_pending_sends_for_both_curves --ignored --nocapture
        ;;
    *)
        echo "OS entropy injection currently supports macOS only" >&2
        exit 1
        ;;
esac
