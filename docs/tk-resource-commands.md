# Shared user, policy, and API-key commands

These commands operate on the organization and credentials selected by shared auth. They do not initialize TVC operator keys. Query output retains the API's named resource arrays/objects; mutations retain the complete activity, including its ID, status, and any result IDs.

## Structured inputs

Create/update/register commands require exactly one of `--input-json JSON` or `--input-file PATH`. Use `--input-file -` for stdin. Supply the **parameters object**, without organization ID, timestamp, or activity envelope. Inputs are parsed before credential resolution. Unsupported fields, malformed JSON, invalid resource UUIDs, and empty create batches are rejected locally; the server remains authoritative for policy language and authorization.

Policy expressions are passed through unchanged. Policy creation uses `effect`, `condition`, `consensus`, and `notes`; policy updates use `policyEffect`, `policyCondition`, `policyConsensus`, and `policyNotes`. Using a create field in an update is an error rather than silently dropping the field.

```sh
tk user list
tk user get 11111111-1111-4111-8111-111111111111
tk user create --input-file users.json
tk user update --input-file user-update.json
tk user delete 11111111-1111-4111-8111-111111111111

tk user tag list
tk user tag create --input-file tag.json
tk user tag update --input-file tag-update.json
tk user tag delete 22222222-2222-4222-8222-222222222222

tk policy list
tk policy get 33333333-3333-4333-8333-333333333333
tk policy create --input-file policy.json
tk policy create-batch --input-file policies.json
tk policy update --input-file policy-update.json
tk policy delete 33333333-3333-4333-8333-333333333333
tk policy evaluations 44444444-4444-4444-8444-444444444444

tk api-key list --user-id 11111111-1111-4111-8111-111111111111
tk api-key register --input-file public-keys.json
tk api-key delete --user-id 11111111-1111-4111-8111-111111111111 55555555-5555-4555-8555-555555555555
```

Delete commands accept multiple positional resource UUIDs. API-key list without a user filter follows the API's default scope. List endpoints here expose no cursor parameters; no unsupported pagination flags are added.

## Example parameters

`users.json` creates a tagged non-root user; root quorum membership is a separate operation.

```json
{
  "users": [{
    "userName": "agent",
    "userTags": ["22222222-2222-4222-8222-222222222222"],
    "apiKeys": [{
      "apiKeyName": "agent-key",
      "publicKey": "REPLACE_WITH_PUBLIC_KEY_HEX",
      "curveType": "API_KEY_CURVE_P256"
    }]
  }]
}
```

`tag-update.json` adds/removes tag members explicitly:

```json
{
  "userTagId": "22222222-2222-4222-8222-222222222222",
  "addUserIds": ["11111111-1111-4111-8111-111111111111"],
  "removeUserIds": []
}
```

`policy.json` contains the exact policy expression to submit. This is a shape example; choose the intended resource scope before creating it.

```json
{
  "policyName": "agent signing",
  "effect": "EFFECT_ALLOW",
  "condition": "activity.action == 'SIGN'",
  "consensus": "approvers.any(user, user.id == '11111111-1111-4111-8111-111111111111')",
  "notes": "Replace condition with the intended wallet scope before use"
}
```

Batch creation wraps those objects in `{"policies": [...]}`. `public-keys.json` uses `{"userId": "...", "apiKeys": [...]}` with public keys only. Local credential generation is separate from remote registration.

## Activity and rotation behavior

Each mutation submits once. Pending, consensus-needed, failed, and rejected activities are returned intact to the shared output boundary. Follow pending work with `tk activity get/wait`; resuming an activity must not repeat resource creation. If a transport error makes the outcome uncertain, inspect activity history before retrying a mutation.

For rotation, register the replacement public key, verify the replacement through `tk whoami` using its explicit credentials/profile, and then revoke the old key. Registration does not select the replacement profile or delete local credentials. The CLI supplies execution; skills retain workflow judgment and already-authorized approval decisions.

This module implements remote API-key management. Contract-interface upload/list/delete remain request recipes; wallet/signing commands and local key generation use their own command families. Tests use a loopback mock server and synthetic keys, with no live organization calls.
