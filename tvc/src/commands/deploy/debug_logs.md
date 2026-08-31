# tvc deploy debug-logs

## Purpose
Fetches enclave debug logs for one deployment via the unary
`get_tvc_deployment_debug_logs` API, either as a one-shot dump or as a
follow-style poll loop (`--poll`). Only works for deployments created in debug
mode (`tvc deploy create --dangerous-deploy-debug-mode`) inside an app that
permits debug deployments (`tvc app create
--dangerous-enable-debug-mode-deployments`) — you'd run it while debugging a
misbehaving debug deployment. Implementation:
`tvc/src/commands/deploy/debug_logs.rs`.

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| deployment id | `-d, --deploy-id <UUID>` | `TVC_DEPLOY_ID` | — | required | no (clap usage error) |
| follow mode | `--poll` | `TVC_DEBUG_LOGS_POLL` | — | false | no |
| poll interval | `--poll-interval-seconds` | `TVC_DEBUG_LOGS_POLL_INTERVAL_SECONDS` | — | 2 (range `1..=i64::MAX-2`) | no |
| history limit | `--tail-lines` | `TVC_DEBUG_LOGS_TAIL_LINES` | — | 0 = server applies no limit | no |
| time window | `--since-seconds` | `TVC_DEBUG_LOGS_SINCE_SECONDS` | — | 0 = server applies no limit | no |
| show k8s timestamp | `--include-platform-timestamp` | `TVC_DEBUG_LOGS_INCLUDE_PLATFORM_TIMESTAMP` | — | false | no |
| disable dedupe | `--disable-dedupe` | `TVC_DEBUG_LOGS_DISABLE_DEDUPE` | — | false | no |
| dedupe window size | `--recent-line-capacity` | `TVC_DEBUG_LOGS_RECENT_LINE_CAPACITY` | — | 1000 (min 1) | no |
| auth: org + API key | — | `TVC_ORG_ID` + `TVC_API_KEY_PUBLIC` + `TVC_API_KEY_PRIVATE` (opt. `TVC_API_BASE_URL`) | active org in `~/.config/turnkey/config.toml` | — | no |

Flag > env resolution is standard clap; no command input has a config-file key
(consistent with the other `deploy` subcommands — the config file carries login
state, not command defaults). Auth deviates from per-value merging by design:
env credentials are all-or-nothing; a partial set errors instead of falling
back to disk (`tvc/src/client.rs:226-234`).

In poll mode, `--tail-lines`/`--since-seconds` apply only to the first request;
every subsequent poll forces `tail_lines=0` and
`since_seconds = interval + 2` (`debug_logs.rs:173-179`).

## Interactive behavior
None. The command never prompts in any mode; `--non-interactive` / JSON mode
changes nothing except output shape. A missing `--deploy-id` is a clap parse
failure (exit 2) in both modes, matching the sibling `deploy
status`/`get-status`/`delete`/`restore` commands, which all require an explicit
deploy id via flag/env.

## Outputs
Human mode: one stdout line per log entry, `<replica label> <content>`, or
`<rfc3339-ns ts> <replica label> <content>` with
`--include-platform-timestamp` (timestamp silently omitted when missing or
unparseable, `debug_logs.rs:411-428`). Poll mode prints
`Connected; polling for debug logs...` to stderr after the first fetch
(`debug_logs.rs:212`). The terminal outcome renders nothing in human mode
(`debug_logs.rs:267-272`).

JSON mode: NDJSON stream of
`{"reason":"debug_log_line","replica":…,"content":…,"ts":{seconds,nanos}|null}`
per printed line — the CLI's only streaming (non-`Outcome`) message
(`tvc/src/outcome.rs:24-28`). Non-poll runs terminate with
`{"reason":"debug_logs_fetched","deploymentId":…,"lineCount":N}` where
`lineCount` counts lines actually printed after dedupe. Poll runs never emit a
terminal outcome (`debug_logs.rs:203-210`). The "Connected" notice is
human-only (suppressed via `shell_eprintln!`, `tvc/src/output.rs:272-276`).

Dedupe: unless `--disable-dedupe`, timestamped lines identical in
(replica, content, seconds, nanos) within the last `--recent-line-capacity`
printed lines are dropped; untimestamped lines always print
(`debug_logs.rs:338-351`).

## Side effects
Read-only against Turnkey: one `get_tvc_deployment_debug_logs` call, plus one
per poll iteration — a query, not an activity. No file writes, no local config
mutation, no device interaction. (Global dispatch behavior: a missing
`~/.config/turnkey/config.toml` is created with defaults before any command
runs, `tvc/src/cli.rs:219-223`.)

## Failure modes
- Bad/missing flags (invalid UUID, out-of-range numbers): clap usage error,
  exit 2; `code: usage_error` NDJSON when `--message-format json`
  (`cli.rs:154-182`).
- No active org / no stored key and no env auth, or partial env auth:
  `command_error`, exit 1 (`client.rs:103-125, 226-234`).
- API failure fetching logs: context `failed to fetch debug logs`
  (`debug_logs.rs:274-282`); classified from the typed client error — 401/403
  `unauthorized`, 404 `not_found`, other statuses `api_error`, connect/timeout
  `network_error` (`tvc/src/errors.rs:212-258`), exit 1.
- Deployment exists but is not debug-mode: no client-side pre-check; whatever
  the server returns surfaces as a generic classified error (see Gaps #4).
- Poll mode: any error on any iteration aborts the loop with exit 1 after
  partial output; a clean exit only happens by killing the process (signal
  exit, not 0).

## Gaps

1. **[capability] Poll mode has no termination condition or clean-exit path.**
   The loop at `debug_logs.rs:215-219` runs until killed or errored; there is
   no `--poll-timeout`/max-iterations/duration flag, and the terminal
   `debug_logs_fetched` outcome is unreachable in poll mode
   (`debug_logs.rs:203-210`). "Collect logs for N seconds in CI" requires
   external `timeout(1)` and always yields a nonzero exit; a JSON consumer
   never sees a terminal reason.

2. **[capability][bug?] One transient fetch error kills a long-running poll
   session.** `fetch_debug_logs(...).await?` inside the loop
   (`debug_logs.rs:217`) propagates immediately: a single DNS blip or timeout
   (`network_error`) ends a follow session that is by design left running
   unattended. No retry/backoff even for the error classes the taxonomy itself
   marks as transient ("request never reached the server", `cli.rs:62-63`).

3. **[bug?] Lines can be silently missed when round-trip overhead exceeds the
   fixed 2s overlap.** Poll requests ask for `since = interval + 2s`
   (`debug_logs.rs:176`, `POLL_OVERLAP_SECONDS` hard-coded at
   `debug_logs.rs:25`), evaluated at server receipt time, but the client
   sleeps `interval` between *receiving* one response and *sending* the next
   (`debug_logs.rs:215-217`), so consecutive server evaluations are separated
   by `interval + round-trip`. When response+request travel (plus server log
   collection time) exceeds 2s, consecutive windows no longer overlap and
   lines in the gap are dropped without any indication. The overlap is not
   user-tunable, so slow links have no mitigation short of `--disable-dedupe`
   plus a larger `--since-seconds` (which poll mode overrides anyway).

4. **[capability] The command's most likely failure — a non-debug deployment —
   gets no tailored error or hint.** The whole precondition story lives in the
   LONG_ABOUT (`debug_logs.rs:43-65`), but the run path does no pre-flight
   (`debug_logs.rs:130-146`) and the server error classifies generically
   (`errors.rs:212-258`); the CLI already knows how to show a deployment's
   debug mode (`deploy status` reports it, `status.rs:132,166`) and has a
   `hint:` channel (`output.rs:176-179`), yet a user pointing this at a
   normal deployment gets a raw API error with no pointer to
   `--dangerous-deploy-debug-mode`.

5. **[docs] The `0 = unlimited` sentinel on `--tail-lines`/`--since-seconds` is
   undocumented in the CLI.** The proto defines zero as "no limit applied"
   (`proto/services/coordinator/public/v1/public_api.proto:3707-3708`), but
   the flag help (`debug_logs.rs:89, 99`) shows `default_value_t = 0` with
   help text ("Return logs newer than this many seconds ago") that reads as
   "0 seconds ago" = nothing; neither help string nor LONG_ABOUT states that 0
   disables the limit.

6. **[bug?] `--recent-line-capacity` is a floor, not an exact window.**
   Eviction triggers at `order.len() == self.order.capacity()`
   (`debug_logs.rs:324`), but `VecDeque::with_capacity(n)`
   (`debug_logs.rs:310-317`) may allocate more than `n`, so the dedupe window
   actually retains the allocator-rounded capacity. Benign direction (dedupes
   more than promised) and the test at `debug_logs.rs:639-652` only asserts
   no growth, but the knob does not do literally what its help text says.

7. **[capability] No replica filter.** Every entry carries a `replica_label`
   and multi-replica output interleaves (`debug_logs.rs:391-406`), but there
   is no `--replica` flag to scope output to one replica in either human or
   JSON mode; the API takes no filter either, so this would be a client-side
   convenience.
