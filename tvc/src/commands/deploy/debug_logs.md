# tvc deploy debug-logs

*Depends on: Part 00 (../00_preliminaries.md). Conformance unit: the deploy debug-logs command.*

## Purpose (informative)

The command fetches enclave debug logs for one deployment through the unary `get_tvc_deployment_debug_logs` API. It runs as a one-shot dump, or as a follow-style poll loop with `--poll`. Debug logs exist only for a deployment created in debug mode (`tvc deploy create --dangerous-deploy-debug-mode`) inside an app that permits debug deployments (`tvc app create --dangerous-enable-debug-mode-deployments`). The typical use is to watch a misbehaving debug deployment. Implementation: `tvc/src/commands/deploy/debug_logs.rs`.

## Acceptance scenario (normative)

The scenario starts from a logged-in tvc config whose active org owns the debug-mode deployment `5376f492-d014-4e01-a6bb-20fc97448e25`. The deployment has one log entry: replica `replica 1/2`, content `hello`, timestamp seconds `1710000000`, nanos `123456789`.

| Step | Action | Expected observation |
|---|---|---|
| 1 | `tvc deploy debug-logs --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | stdout prints `replica 1/2 hello`; the terminal outcome renders nothing else; exit code 0. |
| 2 | Step 1 invocation plus `--include-platform-timestamp` | stdout prints `2024-03-09T16:00:00.123456789Z replica 1/2 hello`; exit code 0. |
| 3 | Step 1 invocation plus `--message-format json` | stdout is NDJSON: one `debug_log_line` object, then `{"reason":"debug_logs_fetched","deploymentId":"5376f492-d014-4e01-a6bb-20fc97448e25","lineCount":1}`; exit code 0. |
| 4 | Step 1 invocation plus `--poll` | stderr prints `Connected; polling for debug logs...` after the first fetch; new lines keep printing about every 2 seconds. |

Pass criterion: steps 1 to 3 pass in one run each from the same logged-in start. Step 4 streams through at least two poll cycles and stops only when the operator stops the process.

## Inputs (normative)

| Input | Flag | Env var | Default | Constraint |
|---|---|---|---|---|
| deployment id | `-d, --deploy-id <UUID>` | `TVC_DEPLOY_ID` | required | UUID (debug_logs.rs:72) |
| follow mode | `--poll` | `TVC_DEBUG_LOGS_POLL` | false | boolean flag (debug_logs.rs:76) |
| poll interval | `--poll-interval-seconds <N>` | `TVC_DEBUG_LOGS_POLL_INTERVAL_SECONDS` | 2 | 1 to i64::MAX minus 2 (debug_logs.rs:27) |
| history limit | `--tail-lines <N>` | `TVC_DEBUG_LOGS_TAIL_LINES` | 0, server applies no limit | i32, 0 or more (debug_logs.rs:31) |
| time window | `--since-seconds <N>` | `TVC_DEBUG_LOGS_SINCE_SECONDS` | 0, server applies no limit | i64, 0 or more (debug_logs.rs:35) |
| platform timestamp | `--include-platform-timestamp` | `TVC_DEBUG_LOGS_INCLUDE_PLATFORM_TIMESTAMP` | false | boolean flag (debug_logs.rs:110) |
| dedupe switch | `--disable-dedupe` | `TVC_DEBUG_LOGS_DISABLE_DEDUPE` | false | boolean flag (debug_logs.rs:114) |
| dedupe window size | `--recent-line-capacity <N>` | `TVC_DEBUG_LOGS_RECENT_LINE_CAPACITY` | 1000 | 1 or more (debug_logs.rs:39) |

The command MUST resolve each input in the Part 00 value resolution order. No input reads the tvc config or a command config, so the order reduces to flag, then environment variable, then built-in default.

Credential resolution follows `crate::client::build_client` (`tvc/src/client.rs:48`). When `TVC_ORG_ID`, `TVC_API_KEY_PUBLIC`, and `TVC_API_KEY_PRIVATE` are all set, the command MUST authenticate from those environment variables. `TVC_API_BASE_URL` MAY override the API base URL, which defaults to `https://api.turnkey.com` (`tvc/src/client.rs:24`). When none of the three is set, the command MUST authenticate from the active org in the tvc config (`tvc/src/client.rs:103`). When only some of the three are set, the command MUST fail and name the missing variables (`tvc/src/client.rs:226`). The command MUST NOT merge environment credentials with stored credentials.

In poll mode `--tail-lines` and `--since-seconds` MUST apply to the first request only. Every later poll request MUST carry `tail_lines` 0 and `since_seconds` equal to the poll interval plus 2 (`debug_logs.rs:173`).

## Interactive behavior (normative)

The command MUST NOT prompt in any mode (`debug_logs.rs:130`). Interactive mode and non-interactive mode behave the same. A missing `--deploy-id` MUST fail as a clap usage error with exit code 2 in both modes (`tvc/tests/deploy_debug_logs.rs:60`). The sibling deploy status, get-status, delete, and restore commands share this shape: an explicit deploy id through flag or environment variable.

## Outputs (normative)

**Human mode**: Each printed log entry MUST render on stdout as `<replica label> <content>` (`debug_logs.rs:413`). With `--include-platform-timestamp`, an entry with a valid timestamp MUST render as `<RFC 3339 nanosecond timestamp> <replica label> <content>` (`debug_logs.rs:417`, test `debug_logs.rs:533`). When the timestamp is missing or fails to parse, the line MUST render without a timestamp (`debug_logs.rs:416`, tests `debug_logs.rs:543`, `debug_logs.rs:553`). An entry without a `line` payload MUST NOT print (`debug_logs.rs:392`, test `debug_logs.rs:589`). In poll mode the command MUST print `Connected; polling for debug logs...` to stderr after the first fetch (`debug_logs.rs:212`). The notice rides the human-only stderr channel (`tvc/src/output.rs:272`). The terminal outcome MUST render nothing in human mode (`debug_logs.rs:267`, test `debug_logs.rs:697`).

**JSON mode**: Each printed line MUST emit one `debug_log_line` NDJSON object with `replica`, `content`, and `ts` fields (`debug_logs.rs:226`, test `debug_logs.rs:667`). `ts` MUST be null when the entry has no timestamp (`debug_logs.rs:230`). The `debug_log_line` message is the only streaming message outside the terminal `Outcome` vocabulary (`tvc/src/outcome.rs:24`). A non-poll run MUST end with one `debug_logs_fetched` object that carries `deploymentId` and `lineCount` (`debug_logs.rs:205`). `lineCount` MUST count the lines printed after dedupe (`debug_logs.rs:201`, `debug_logs.rs:263`). A poll run MUST NOT emit a terminal outcome (`debug_logs.rs:203`). The `Connected` notice MUST NOT appear in JSON mode (`tvc/src/output.rs:272`).

**Dedupe**: When `--disable-dedupe` is absent, the command MUST drop a timestamped line that matches a retained key (`debug_logs.rs:341`). The key is the tuple of replica, content, seconds, and nanos (`debug_logs.rs:285`, test `debug_logs.rs:505`). The retained set covers the last `--recent-line-capacity` printed timestamped lines (`debug_logs.rs:324`, test `debug_logs.rs:622`). A line without a timestamp MUST always print (`debug_logs.rs:343`, test `debug_logs.rs:513`). With `--disable-dedupe`, every returned line MUST print (`debug_logs.rs:365`, test `debug_logs.rs:612`).

## Side effects (normative)

A non-poll run MUST make exactly one `get_tvc_deployment_debug_logs` call (`debug_logs.rs:200`). A poll run MUST add one call per poll cycle (`debug_logs.rs:217`). The call is a read-only query. The command MUST NOT submit an activity, write a file, or change the tvc config. Global dispatch creates the tvc config when the file is absent (INV-G4).

## Failure modes (normative)

- An invalid flag value MUST cause a clap usage error with exit code 2 (`tvc/tests/deploy_debug_logs.rs:96`). Examples: a malformed UUID, an out-of-range number. In JSON mode the same failure MUST surface as one NDJSON object with `code` `usage_error` (`tvc/src/cli.rs:154`).
- A missing `--deploy-id` MUST fail the same way (`tvc/tests/deploy_debug_logs.rs:60`).
- With no active org and no environment credentials, the command MUST fail with `code` `command_error` and exit code 1 (`tvc/src/client.rs:106`). A missing stored API key for the active org MUST produce the same failure (`tvc/src/client.rs:117`). A partial environment credential set MUST produce the same failure (`tvc/src/client.rs:226`).
- A failed log fetch MUST carry the context `failed to fetch debug logs` and exit with code 1 (`debug_logs.rs:281`). Classification follows the typed client error (`tvc/src/errors.rs:212`). HTTP 401 and 403 map to `unauthorized`, and 404 maps to `not_found`. Another non-success status maps to `api_error`. A connect or timeout failure maps to `network_error`.
- The run path performs no debug-mode pre-check (`debug_logs.rs:130`). A request against a non-debug deployment surfaces whatever the server returns, classified through the generic taxonomy (see Gap 4).
- In poll mode an error in any cycle MUST end the loop with exit code 1 after partial output (`debug_logs.rs:217`). A poll run has no success exit path; the process ends through a signal (see Gap 1).

## Test vectors (normative)

Vector comparison excludes the global nondeterministic fields that Part 00 names. Vectors V-1 to V-4, V-11, and V-12 fix the fixture log entry from the acceptance scenario. Live log content varies with the deployment.

| # | Given | Invocation | Expected observation |
|---|---|---|---|
| V-1 | Logged-in tvc config; the server returns the fixture entry once. | `tvc deploy debug-logs --deploy-id 5376f492-d014-4e01-a6bb-20fc97448e25` | stdout `replica 1/2 hello`; no terminal rendering; exit code 0 (debug_logs.rs:413; tests debug_logs.rs:523, 697). |
| V-2 | Same as V-1. | V-1 plus `--include-platform-timestamp` | stdout `2024-03-09T16:00:00.123456789Z replica 1/2 hello`; exit code 0 (debug_logs.rs:417; test debug_logs.rs:533). |
| V-3 | Same as V-1. | V-1 plus `--message-format json` | stdout NDJSON: `{"reason":"debug_log_line","replica":"replica 1/2","content":"hello","ts":{"seconds":"1710000000","nanos":"123456789"}}` then `{"reason":"debug_logs_fetched","deploymentId":"5376f492-d014-4e01-a6bb-20fc97448e25","lineCount":1}`; exit code 0 (test debug_logs.rs:667; debug_logs.rs:205). |
| V-4 | Same as V-1. | V-1 plus `--poll --poll-interval-seconds 7 --tail-lines 100` | First request carries `tail_lines` 100; every later request carries `tail_lines` 0 and `since_seconds` 9 (test debug_logs.rs:483); stderr prints `Connected; polling for debug logs...` (debug_logs.rs:212); no terminal outcome (debug_logs.rs:203). |
| V-5 | Any HOME. | `tvc deploy debug-logs` | stderr names the missing `--deploy-id <DEPLOY_ID>`; exit code 2; with `--message-format json`, one NDJSON object with `code` = `usage_error` (test tvc/tests/deploy_debug_logs.rs:60; tvc/src/cli.rs:154). |
| V-6 | Any HOME. | V-1 plus `--tail-lines -1` | stderr contains `invalid value '-1'` and `is not in 0..`; exit code 2 (test tvc/tests/deploy_debug_logs.rs:96). |
| V-7 | Any HOME. | V-1 plus `--poll-interval-seconds 0`, with or without `--poll` | stderr contains `invalid value '0'` and `is not in 1..`; exit code 2 (tests tvc/tests/deploy_debug_logs.rs:112, 150). |
| V-8 | Any HOME. | V-1 plus `--recent-line-capacity 0` | stderr contains `invalid value '0'` and `is not in 1..`; exit code 2 (test tvc/tests/deploy_debug_logs.rs:184). |
| V-9 | Fresh HOME, no auth env vars set. | Same as V-1 | stderr contains `No active organization. Run `tvc login` first.`; exit code 1; JSON `code` = `command_error` (test tvc/tests/deploy_debug_logs.rs:73; tvc/src/client.rs:106). |
| V-10 | Only `TVC_ORG_ID=6c1f7e2a-0b3d-4e5f-8a9b-1c2d3e4f5a6b` set. | Same as V-1 | Error `partial env var auth: missing TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE. Set all three (TVC_ORG_ID, TVC_API_KEY_PUBLIC, TVC_API_KEY_PRIVATE) env vars or none.`; exit code 1 (tvc/src/client.rs:226). |
| V-11 | The server returns the fixture entry twice in one response. | Same as V-1 | The line prints once; JSON `lineCount` = 1 (tests debug_logs.rs:573, 589). |
| V-12 | Same as V-11. | V-1 plus `--disable-dedupe` | Both lines print (test debug_logs.rs:612). |

## Invariants (normative)

| Invariant | Requirement | Mechanism |
|---|---|---|
| INV-1 | Every poll request after the first MUST carry `tail_lines` 0 and `since_seconds` equal to the poll interval plus `POLL_OVERLAP_SECONDS` (2). | `DebugLogQueryRequest::into_poll_request` (debug_logs.rs:173); behavioral check debug_logs.rs:483. |
| INV-2 | The dedupe window capacity MUST be at least 1. | clap range parser (debug_logs.rs:39, 123) plus the constructor assert (debug_logs.rs:311); behavioral checks tvc/tests/deploy_debug_logs.rs:184 and debug_logs.rs:584. |
| INV-3 | Dedupe MUST key on replica, content, seconds, and nanos, and a line without a timestamp MUST always print. | `LogLineKey` fields (debug_logs.rs:285) and the early return in `record_if_new` (debug_logs.rs:343); behavioral checks debug_logs.rs:505, 513. |
| INV-4 | Human mode and JSON mode MUST print the same log lines: both render from the one emitted `DebugLogLine` value. | `Shell::emit` on `DebugLogLine`, whose `Display` defers to `format_log_line` (debug_logs.rs:236); behavioral check debug_logs.rs:685. |
| INV-5 | Dedupe state MUST NOT grow past its initial capacity. | Eviction on a full window in `RecentLogLineDeduper::insert` (debug_logs.rs:324); behavioral check debug_logs.rs:639. |

## Gaps (informative)

1. **[capability] Poll mode has no termination condition and no clean-exit path.** The loop at `debug_logs.rs:215` runs until an error or a kill signal. No `--poll-timeout`, max-iteration, or duration flag exists, and the terminal `debug_logs_fetched` outcome is unreachable in poll mode (`debug_logs.rs:203`). A CI job that wants logs for N seconds needs an external `timeout(1)` and always sees a nonzero exit. A JSON consumer never sees a terminal reason.

2. **[capability][bug?] One transient fetch error kills a long-running poll session.** The `fetch_debug_logs` call inside the loop propagates any error immediately (`debug_logs.rs:217`). A single DNS blip or timeout (`network_error`) ends a follow session that runs unattended by design. No retry or backoff exists, even for the error classes the taxonomy itself marks as transient ("request never reached the server", `tvc/src/cli.rs:62`).

3. **[bug?] Poll mode can silently drop lines.** Poll requests ask for `since_seconds` equal to the interval plus the fixed 2 second overlap (`debug_logs.rs:176`, `debug_logs.rs:25`). The server evaluates the window at receipt time. The client sleeps for the interval between receipt of one response and dispatch of the next (`debug_logs.rs:215`). Consecutive server evaluations therefore sit the interval plus one round trip apart. When travel plus server collection time exceeds 2 seconds, consecutive windows stop overlapping, and lines in the gap drop with no indication. The overlap has no flag, so a slow link has no mitigation beyond `--disable-dedupe` plus a larger `--since-seconds`, and poll mode overrides the latter anyway.

4. **[capability] The most likely failure, a deployment without debug mode, gets no tailored error or hint.** The precondition story lives in the `LONG_ABOUT` (`debug_logs.rs:43`). The run path does no pre-flight check (`debug_logs.rs:130`), and the server error classifies generically (`tvc/src/errors.rs:212`). The CLI already reports a deployment's debug mode (`tvc/src/commands/deploy/status.rs:132`, `status.rs:166`) and has a `hint:` channel (`tvc/src/output.rs:176`). A user who points the command at a normal deployment gets a raw API error with no pointer to `--dangerous-deploy-debug-mode`.

5. **[docs] The CLI help does not document the zero sentinel on `--tail-lines` and `--since-seconds`.** The proto defines zero as "no limit applied" (`proto/services/coordinator/public/v1/public_api.proto:3707`). The flag help (`debug_logs.rs:89`, `debug_logs.rs:99`) shows a default of 0 next to text such as "Return logs newer than this many seconds ago". That reading suggests an empty window. Neither the help strings nor the LONG_ABOUT state that 0 disables the limit.

6. **[bug?] `--recent-line-capacity` sets a floor, and the true dedupe window can be larger.** Eviction triggers at `order.len() == self.order.capacity()` (`debug_logs.rs:324`). `VecDeque::with_capacity(n)` (`debug_logs.rs:314`) may allocate more than `n`, so the dedupe window retains the allocator-rounded capacity. The direction is benign, since the window dedupes more than promised, and the test at `debug_logs.rs:639` only asserts no growth. The knob still deviates from its help text.

7. **[capability] No replica filter exists.** Every entry carries a `replica_label`, and multi-replica output interleaves (`debug_logs.rs:391`). No `--replica` flag scopes output to one replica in either output mode. The API takes no filter either, so the feature would be a client-side convenience.
