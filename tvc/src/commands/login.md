# tvc login

## Purpose

Authenticate with Turnkey and set up local credentials for one organization: select (or
interactively create) a config profile, ensure an API key exists and verifies via
`get_whoami`, ensure the org's default operator backend is usable (find-or-generate the
local key file, or resolve the registered hosted/YubiKey operator), set the profile as
the active org, and persist everything to `~/.config/turnkey/`. Run it once per machine
per org, and again to switch the active org or after restoring/rotating credentials.
Entry: `tvc/src/commands/login.rs:97` (`run`), dispatched from `tvc/src/cli.rs:302`.
(This file also hosts `profile delete` — `DeleteArgs`/`run_delete` — specced separately.)

## Inputs

| input | flag | env | config key | default | prompted? |
|---|---|---|---|---|---|
| organization (alias or ID) | `--org` | `TVC_ORG` | — (`active_org` is NOT consulted) | none | yes: picker over configured orgs + `[new]` (login.rs:601-640) |
| API base URL | `--api-base-url` | `TVC_API_BASE_URL` | `orgs.<alias>.api_base_url` (existing orgs) | `https://api.turnkey.com` (new orgs, login.rs:764-768) | no |
| YubiKey serial | `--serial` | — | — | sole YubiKey operator of the org | yes: picker when the org has several (login.rs:370-387); interactive only |
| new-org organization ID | — | — | — | — | prompt-only (login.rs:655-658) |
| new-org alias | — | — | — | `"default"` | prompt-only (login.rs:660) |
| new-org operator key type | — | — | — | — | prompt-only: `Local key file` / `YubiKey` (login.rs:663-710); hosted not offered |
| operator-key backup destination | — | — | — | `operator-<alias>-backup.json` | prompt-only nudge after generating a local key (login.rs:879-923) |
| non-interactive (global) | `--non-interactive` | `TVC_NON_INTERACTIVE` | — | false | — |

Resolution-order deviations:
- `--api-base-url`/`TVC_API_BASE_URL` does not merely override for the run — for an
  existing org it is **written into** the stored config (login.rs:426-430, 770-781) and
  saved (login.rs:472).
- The config layer is skipped for org selection: unlike sibling commands, login never
  falls back to `active_org`.
- `--serial` is silently ignored when the selected org's default backend is not yubikey
  (documented in its help text, login.rs:42-44).

## Interactive behavior

Interactive plan building (`build_login_plan_interactive`, login.rs:345-401):

1. `--org`/`TVC_ORG` absent → org picker listing every configured alias (`alias (id)`,
   active suffixed `(active)`) plus `[new] Add a new organization` (login.rs:619-640).
   No orgs configured → straight to the new-org prompts (login.rs:613-617).
2. New org: prints the dashboard welcome URL, prompts **Organization ID** (bails if
   empty), **Organization alias** (default `default`), **Operator key type**
   (Local / YubiKey). YubiKey: `--serial` must be in the device registry, else the sole
   registered serial is auto-picked, several → picker, none → bail directing to
   `tvc keys refresh-yubikey` (login.rs:683-708).
3. Existing org whose default backend is yubikey: `--serial` validated against the
   org's operators; absent with several YubiKey operators → "Select YubiKey operator"
   picker (login.rs:360-393).
4. During execution: missing API key file → generate one, print dashboard registration
   steps, then block on `Press Enter when done...` read from raw stdin
   (login.rs:496-497, 824-830).
5. Local backend with missing operator key → generate it, then (only when stdin is a
   TTY) offer to back it up: confirm → destination prompt → copy; any failure or escape
   degrades to a warning since config and keys are already saved (login.rs:879-923).

Non-interactive / JSON mode (`build_login_plan_non_interactive`, login.rs:403-414):
- `--org` becomes a hard requirement (`missing_required_input`, login.rs:404-406).
- Only existing orgs: the plan is always `OrgPlan::Existing`; there is no new-org path.
- API key file must already exist (`ApiKeyPolicy::RequireExisting`, login.rs:500-505).
- Yubikey-default org with several YubiKey operators and no `--serial` →
  `missing_required_input` naming `--serial` (login.rs:552-559).
- A missing local operator key is still silently generated (login.rs:523, 832-860) —
  see gap 9.

## Outputs

Human mode: progress lines (`Selected org: …`, new-yubikey-org public key + "register
this as an operator" guidance, `Using existing API key.` / `API Key Generated!` +
dashboard steps, `Verifying credentials...`, `Using hosted|YubiKey operator …`,
operator-key generation and backup messages), then the `LoggedIn` display block
(login.rs:1079-1147): organization/user identity from whoami, alias, API public key,
and an operator-variant-specific section (local: operator key path; hosted: name +
operator id; yubikey: name + serial) plus config/API-key paths.

JSON mode: one NDJSON outcome, `reason: "logged_in"` (outcome.rs:33), camelCase fields
`organizationName/organizationId/username/userId/alias/apiPublicKey/configFilePath/
apiKeyPath` plus a flattened `operatorKind` tag of `local|hosted|yubikey` with variant
fields (login.rs:966-1005; contract pinned by tests login.rs:1177-1259 — the local
shape is an explicit compatibility contract). All progress lines are suppressed
(`shell_println!` is human-only, output.rs:246-254). Errors emit `command_error` /
`missing_required_input` envelopes per the global contract.

## Side effects

- Writes `~/.config/turnkey/tvc.config.toml`: adds the new org entry (login.rs:463;
  `Config::add_org` inserts/replaces, config/turnkey/mod.rs:661), persists any
  `--api-base-url` override onto an existing org (login.rs:426-430), switches
  `active_org` (login.rs:470), all saved at login.rs:472 — before credential
  verification.
- Writes `orgs/<alias>/api_key.json` when generating an API key (owner-only perms,
  api_key.rs:68); writes the local operator key file (owner-only,
  qos_operator_key.rs:151) when missing; optional backup copy at a user-chosen path
  (default/umask perms — TVC-241, backup_operator_key.rs:181-185).
- Turnkey API: exactly one call, `get_whoami` (login.rs:947-954), via a client built
  from the stored API key (login.rs:511, 945). No activities submitted. CI env auth
  (`TVC_ORG_ID`/`TVC_API_KEY_PUBLIC`/`TVC_API_KEY_PRIVATE`, client.rs:48-64) is NOT
  consulted — login is file-based only.
- YubiKey: no device I/O; only the registry's cached public key is read
  (login.rs:562-570), so the device need not be connected.

## Failure modes

- `--org` missing in non-interactive → `missing_required_input`, exit 1 (login.rs:405).
- Multiple YubiKey operators, no `--serial`, non-interactive →
  `missing_required_input` naming `--serial`, exit 1 (login.rs:553-559).
- Org query not found → plain bail "Organization '…' not found" → `command_error`
  (login.rs:421-424), exit 1 — not `not_found` (see gap 11).
- API key file absent in non-interactive → bail "API key is required in
  non-interactive mode" → `command_error`, exit 1 (login.rs:500-505).
- Serial not in device registry (new org or yubikey login) → bail directing to
  `tvc keys refresh-yubikey` → `command_error` (login.rs:446-453, 562-570, 686-691).
- `get_whoami` failure → classified by `crate::errors::classify`: `unauthorized`
  (401/403), `not_found` (404), `api_error`, `network_error`, `client_version_too_old`
  (errors.rs:212-243); exit 1. Config mutations from earlier in the run persist.
- No/multiple local operators, or multiple hosted operators → typed selection errors
  with org context → `command_error` (login.rs:519-533; config/turnkey/mod.rs:437-483).
- Prompt escape/cancel or non-TTY stdin under inquire → `command_error`, exit 1.
- Malformed `--serial` hex → clap `usage_error`, exit 2 (yubikey.rs serial parser;
  cf. cli.rs:568-573).

## Gaps

1. **[capability] Non-interactive/JSON mode cannot bootstrap: no way to create an org
   profile or supply API-key material.** `build_login_plan_non_interactive` only ever
   produces `OrgPlan::Existing` (login.rs:403-414); org ID, alias, and operator kind are
   prompt-only (login.rs:642-717) with no flag/env equivalents, and
   `ApiKeyPolicy::RequireExisting` (login.rs:500-505) has no `--api-key`/path escape
   hatch even though the CLI already defines env credentials (client.rs:20-23) that
   login ignores. Headless setup requires hand-writing `tvc.config.toml` and key files.

2. **[capability] Login always validates the org's `default_operator_kind`; the user
   cannot pick a backend or operator.** Hard match at login.rs:518 (deliberate per the
   comment at login.rs:512-517), with no `--operator-kind`/operator-name input. The only
   way to change the default is `operator create --default` — which must create a new
   operator (operator/create.rs:93-95, 230-232, 399-401) — or hand-editing; nothing can
   flip a default back to `local`, so e.g. an org holding local + yubikey records with a
   yubikey default can never have login find-or-generate its local key. Same
   default-state-constrains-explicit-choice shape as the `re-encrypt-local-share`
   canonical example.

3. **[capability] An org with multiple hosted operators cannot log in at all.**
   `select_hosted_operator` is sole-or-error (config/turnkey/mod.rs:473-483) and login
   offers no disambiguator in either mode (login.rs:530-533) — contrast the yubikey
   backend, which gets both `--serial` (login.rs:42-44) and an interactive picker
   (login.rs:370-387). `operator create` happily appends a second hosted operator
   (operator/create.rs:228), making the state reachable through supported commands.

4. **[consistency] Non-interactive login does not default to the active org.**
   `--org` is required (login.rs:404-406) even when `active_org` is set, while sibling
   `keys backup-operator-key --org` explicitly "Defaults to the active organization"
   (backup_operator_key.rs:26-29, 58-60). A CI re-verify of the current profile must
   restate the org.

5. **[consistency] A set `TVC_ORG` env var permanently suppresses the org picker and
   the entire new-org flow.** Clap env feeds `args.org` (login.rs:36-37), so
   `prompt_for_org_plan` is unreachable while `TVC_ORG` is exported — plausible, since
   `backup-operator-key` shares the var (backup_operator_key.rs:28). When the value
   matches nothing, the error says "Run `tvc login` without --org" (login.rs:421-424),
   advice that fails identically because the env var, not the flag, is the source.

6. **[bug?] `--api-base-url`/`TVC_API_BASE_URL` silently and persistently rewrites an
   existing org's stored base URL — before credentials are verified.**
   `update_api_base_url_from_override` mutates the org (login.rs:426-430, 770-781) and
   the config is saved (login.rs:472) ahead of `get_whoami` (login.rs:511), so a failed
   login still leaves the URL switched. `TVC_API_BASE_URL` doubles as the optional CI
   env-auth base URL (client.rs:21, 196-198), so an exported CI environment leaks into
   persistent config mutation. [docs] The flag's help ("Defaults to production for
   newly configured orgs", login.rs:38) never mentions the rewrite.

7. **[bug?] A failed login still switches (and persists) the active org.**
   `set_active_org` + `config.save()` run at login.rs:470-472, before the API-key check,
   whoami, and operator resolution; a login that fails verification (revoked key, wrong
   env, missing operator) leaves `active_org` pointing at the failed profile, silently
   redirecting every subsequent command's credentials (client.rs:103-117). New-org
   entries likewise persist on failure (retry-friendly, but combined with the active-org
   flip it changes global behavior on error).

8. **[bug?] New-org alias collision silently clobbers an existing profile.**
   `prompt_for_new_org_inputs` never checks the alias against `config.orgs`
   (login.rs:660), and `Config::add_org` is an unconditional insert/replace
   (config/turnkey/mod.rs:661). Entering an existing alias (the default is literally
   `"default"`) replaces that profile's org id, backend, and operator records —
   including hosted operator identities recoverable from nowhere else.

9. **[consistency] Operator-key generation is ungated in non-interactive mode, unlike
   API-key generation.** `ApiKeyPolicy::RequireExisting` blocks API-key creation in CI
   (login.rs:494-505), but `find_or_generate_operator_key` fabricates a new local
   operator key whenever the file is missing, in any mode (login.rs:523, 846-860). In
   JSON mode the "register this as an operator" guidance is suppressed (human-only
   macros, output.rs:246-254), so a partially-restored machine silently gains an
   unregistered key that later fails approvals; only the JSON payload's changed
   `operatorPublicKey` hints at it.

10. **[bug?] `wait_for_dashboard_registration` reads raw stdin with no TTY guard.**
    login.rs:824-830 bypasses the ctx/prompt layer; with piped or closed stdin in
    interactive human mode (reachable without any inquire prompt when `--org` is
    supplied), `read_line` returns immediately on EOF, so the flow generates a key,
    skips the registration wait, and fails whoami confusingly. The backup nudge ten
    lines later explicitly guards with `prompts::stdin_can_prompt()` (login.rs:879).

11. **[consistency] Lookup failures classify as `command_error`, not `not_found`.**
    "Organization '…' not found" is a plain string bail (login.rs:421-424), so JSON
    consumers get `code: "command_error"` although the documented taxonomy assigns
    `not_found` to "a resource that resolved to empty" (cli.rs:58). Same for the
    API-key-required bail (login.rs:500-505), which is conceptually missing input but
    cannot be `missing_required_input` because no flag exists to satisfy it (ties to
    gap 1).

12. **[bug?] Minor input-hygiene holes in the new-org prompts and org picker.** The
    alias is unvalidated and flows into filesystem paths — `default_org_dir` joins it
    verbatim (config/turnkey/mod.rs:573-585), so `../x` or `a/b` shapes directories
    outside `orgs/`; the org-ID check only catches the truly empty string
    (login.rs:656-658), accepting whitespace; and both the org picker and `find_org`'s
    ID fallback iterate a `HashMap` (login.rs:619-634, 738-744), so picker order is
    nondeterministic per run and a duplicated org ID resolves to an arbitrary alias.
