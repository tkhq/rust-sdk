#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
case "$(uname -s)" in
    Darwin) ;;
    *) echo "OS entropy injection currently supports macOS only" >&2; exit 1 ;;
esac
cargo build -p tvc --bin tvc --message-format=json > "$scratch/bin-build.jsonl"
cargo test -p tvc --lib --no-run --message-format=json > "$scratch/lib-build.jsonl"
cc -dynamiclib tvc/tests/support/fail_stamp_entropy.c -o "$scratch/fail_entropy.dylib"
cc -dynamiclib tvc/tests/support/deny_network.c -o "$scratch/deny_network.dylib"
python3 - "$scratch" <<'PY'
import json, os, pathlib, subprocess, sys
scratch = pathlib.Path(sys.argv[1])
def executable(log, test):
    return next(row['executable'] for line in (scratch / log).read_text().splitlines()
                if (row := json.loads(line)).get('reason') == 'compiler-artifact'
                and row.get('target', {}).get('name') == 'tvc'
                and row.get('profile', {}).get('test') == test and row.get('executable'))
binary = executable('bin-build.jsonl', False)
(scratch / 'armed').write_text('armed')
g = '046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5'
g2 = '047cf27b188d034f7e8a52380304b51ac3c08969e277f21b35a60b48fc4766997807775510db8ed040293d9ac69f7430dbba7dade63ce982299e04b79d227873d1'
for api in ['https://api.turnkey.com', 'https://api.preprod.turnkey.engineering']:
    for operator in [True, False]:
        home = scratch / (('operator-' if operator else 'quorum-') + api.split('//')[1])
        config = home / '.config/turnkey/tvc.config.toml'
        config.parent.mkdir(parents=True)
        baseline = ('version = 1\nactive_org = "synthetic"\n[orgs.synthetic]\n'
                    'id = "synthetic-org"\napi_key_path = "unused.json"\n'
                    'api_base_url = "' + api + '"\ndefault_operator_kind = "local"\noperators = []\n').encode()
        config.write_bytes(baseline)
        env = {'HOME': str(home), 'TVC_ORG_ID': 'synthetic-org', 'TVC_API_BASE_URL': api,
               'TVC_API_KEY_PRIVATE': '00' * 31 + '01',
               'TVC_API_KEY_PUBLIC': '03' + g[2:66],
               'TVC572_ENTROPY_FAILURE_MARKER': str(scratch / 'armed'),
               'TVC572_CONNECT_MARKER': str(scratch / 'connected'),
               'DYLD_INSERT_LIBRARIES': str(scratch / 'fail_entropy.dylib') + ':' + str(scratch / 'deny_network.dylib')}
        command = [binary, '--message-format', 'json']
        command += ['operator', 'create'] if operator else ['keys', 'create-quorum-key', '--threshold', '2', '--operator-encrypt-keys', g + ',' + g2]
        run = subprocess.run(command, env=env, capture_output=True, text=True, timeout=20)
        assert run.returncode == 1, (command, run.returncode, run.stdout, run.stderr)
        error, = [json.loads(line) for line in run.stdout.splitlines()]
        assert error['reason'] == 'command_error' and error['code'] == 'entropy_unavailable', error
        assert 'creation was never submitted' in error['message'], error
        assert 'may have succeeded' not in error['message'], error
        assert not ({'encryptPublicKey', 'quorumPublicKey'} & error.keys()), error
        assert config.read_bytes() == baseline
        assert not (scratch / 'connected').exists(), 'entropy failure reached connect'
        assert not run.stderr, run.stderr
print('Actual tvc binary: production/preprod operator/quorum first-send entropy failure; no connect, no config mutation, no accepted output.')
(scratch / 'armed').unlink()
test_env = dict(os.environ, TVC572_ENTROPY_FAILURE_MARKER=str(scratch / 'armed'),
                DYLD_INSERT_LIBRARIES=str(scratch / 'fail_entropy.dylib'))
subprocess.run([executable('lib-build.jsonl', True), '--exact',
                'operator::hosted::tests::pending_entropy_failure_preserves_cli_rendering_for_both_curves',
                '--ignored', '--nocapture'], env=test_env, check=True)
PY
