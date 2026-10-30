//! Verify a file of minted tokens against the requests they were minted for.
//!
//! ```text
//! cargo run -p turnkey_jwt_profiles --example verify_tokens -- tokens.json
//! ```
//!
//! The file is `{signer_public_uncompressed_hex, organization_id,
//! cases: [{name, expect_ok, jwt, method, path, query, headers, body,
//! audience}]}`. `signer_public_uncompressed_hex` is the Turnkey signer
//! public key as a 65-byte uncompressed SEC1 point. Each case passes when the
//! outcome matches `expect_ok`. The verifier runs at `iat + 1`, so expiry
//! does not matter.

use std::process::ExitCode;
use std::sync::Arc;

use http::{HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use turnkey_jwt_profiles::http_authz::jwks::{PinnedKeySet, thumbprint_p256};
use turnkey_jwt_profiles::http_authz::{ReceivedRequest, RouteRule, Verifier, VerifierConfig, b64};

#[derive(Deserialize)]
struct Fixture {
    signer_public_uncompressed_hex: String,
    organization_id: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    expect_ok: bool,
    jwt: String,
    method: String,
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: String,
    audience: String,
}

fn jwks(point_hex: &str) -> Result<String, String> {
    const BAD_POINT: &str = "signer_public_uncompressed_hex must be a 65-byte 04||x||y point";
    let point = hex::decode(point_hex)
        .ok()
        .filter(|p| p.len() == 65 && p.first() == Some(&0x04))
        .ok_or(BAD_POINT)?;
    let (x, y) = (
        b64::url_encode(point.get(1..33).ok_or(BAD_POINT)?),
        b64::url_encode(point.get(33..).ok_or(BAD_POINT)?),
    );
    let kid = thumbprint_p256(&x, &y);
    Ok(format!(
        r#"{{"keys":[{{"kty":"EC","crv":"P-256","x":"{x}","y":"{y}","kid":"{kid}"}}]}}"#
    ))
}

/// The unverified `iat`, used only to pick a `now` inside the token lifetime.
fn iat(jwt: &str) -> Option<u64> {
    let payload = b64::url_decode(jwt.split('.').nth(1)?)?;
    serde_json::from_slice::<serde_json::Value>(&payload)
        .ok()?
        .get("iat")?
        .as_u64()
}

fn run_case(keys: &Arc<PinnedKeySet>, organization_id: &str, case: &Case) -> Result<(), String> {
    let config = VerifierConfig::new(&case.audience, organization_id);
    let verifier = Verifier::new(config, keys.clone()).map_err(|e| e.to_string())?;
    let mut headers = HeaderMap::new();
    for (name, value) in &case.headers {
        let name = HeaderName::from_bytes(name.as_bytes()).map_err(|e| e.to_string())?;
        let value = HeaderValue::from_str(value).map_err(|e| e.to_string())?;
        headers.append(name, value);
    }
    let received = ReceivedRequest {
        method: &case.method,
        path: &case.path,
        raw_query: (!case.query.is_empty()).then_some(case.query.as_str()),
        headers: &headers,
        body: case.body.as_bytes(),
    };
    let now = iat(&case.jwt)
        .and_then(|iat| iat.checked_add(1))
        .ok_or("token has no readable iat")?;
    verifier
        .authorize(&case.jwt, &RouteRule::default(), &received, now)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: verify_tokens <tokens.json>");
        return ExitCode::from(2);
    };
    let fixture: Fixture = match std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{path}: {e}");
            return ExitCode::from(2);
        }
    };
    let keys = match jwks(&fixture.signer_public_uncompressed_hex)
        .and_then(|j| PinnedKeySet::from_jwks_json(j.as_bytes()).map_err(|e| e.to_string()))
    {
        Ok(k) => Arc::new(k),
        Err(e) => {
            eprintln!("signer key: {e}");
            return ExitCode::from(2);
        }
    };
    let mut failed = 0_usize;
    for case in &fixture.cases {
        let outcome = run_case(&keys, &fixture.organization_id, case);
        let pass = outcome.is_ok() == case.expect_ok;
        failed = failed.saturating_add(usize::from(!pass));
        let verdict = if pass { "PASS" } else { "FAIL" };
        match outcome {
            Ok(()) => println!("{}: {verdict} (accepted)", case.name),
            Err(e) => println!("{}: {verdict} ({e})", case.name),
        }
    }
    println!("{} cases, {failed} failed", fixture.cases.len());
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
