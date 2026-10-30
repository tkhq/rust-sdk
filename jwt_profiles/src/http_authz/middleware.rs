//! Axum route layer that enforces http-authz/v1 on one route (feature
//! `axum`).
//!
//! [`AuthzLayer`] runs, in order:
//!
//! 1. `Authorization: Bearer` extraction and the token checks
//!    ([`Verifier::verify_token`]);
//! 2. buffering the body (at most the layer's body limit, 4096 bytes by
//!    default; see [`AuthzLayer::with_max_body_bytes`]);
//! 3. the `iat`/`exp` checks again at the current time, so the token must
//!    still be valid when the full request has arrived, not only when its
//!    headers did;
//! 4. [`super::bind_request`] with the route's [`RouteRule`]: method, path,
//!    query, required and bound headers (rebuilt from the lowercase names
//!    that hyper delivers), and the body digest.
//!
//! Tokens are not single-use: the same request passes until `exp`.
//!
//! On success it stores [`Verified`] in the request extensions, puts the
//! buffered body back, and calls the handler.
//!
//! A rejected request does not tell the caller which check failed. Every
//! verification failure is a `401` with `WWW-Authenticate: Bearer` and the
//! body `{"error": "unauthorized"}`. An oversized body is a `413` with the
//! body `{"error": "payload_too_large"}`. The layer logs the specific reason
//! (never the token) with `tracing` at `info` level.
//!
//! For debugging, [`AuthzLayer::with_detailed_errors`] puts the specific code
//! and message in the body: `{"error": <code>, "message": <text>}`. Do not
//! turn it on in production: the detail helps an attacker probe the checks.
//!
//! ```no_run
//! use std::sync::Arc;
//! use axum::routing::post;
//! use axum::{Extension, Router};
//! use turnkey_jwt_profiles::http_authz::middleware::{Verified, route_layer};
//! use turnkey_jwt_profiles::http_authz::{RouteRule, Verifier};
//!
//! # fn build(verifier: Arc<Verifier>) -> Router {
//! let rule = RouteRule {
//!     required_headers: vec!["accept".into()],
//!     require_body_digest: true,
//! };
//! let handler = |Extension(Verified(claims)): Extension<Verified>| async move {
//!     format!("{} in {}", claims.sub, claims.iss)
//! };
//! Router::new().route(
//!     "/v1/things",
//!     post(handler).route_layer(route_layer(verifier, rule)),
//! )
//! # }
//! ```

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::{OriginalUri, Request};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;
use tower::{Layer, Service};

use super::jwt::Claims;
use super::{AuthError, RouteRule, Verifier, unix_now};

/// Default body limit of [`AuthzLayer`], in bytes.
pub const DEFAULT_MAX_BODY_BYTES: usize = 4 * 1024;

/// The verified claims, available to the handler as an extension.
#[derive(Debug, Clone)]
pub struct Verified(pub Claims);

fn reject_with(
    status: StatusCode,
    detailed_code: &str,
    generic_code: &str,
    reason: &dyn std::fmt::Display,
    detailed: bool,
) -> Response {
    tracing::info!(reason = %reason, "authorization rejected");
    let body = if detailed {
        serde_json::json!({"error": detailed_code, "message": reason.to_string()})
    } else {
        serde_json::json!({"error": generic_code})
    };
    (status, axum::Json(body)).into_response()
}

fn reject(err: &AuthError, detailed: bool) -> Response {
    let mut resp = reject_with(
        StatusCode::UNAUTHORIZED,
        err.code(),
        "unauthorized",
        err,
        detailed,
    );
    resp.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        header::HeaderValue::from_static("Bearer"),
    );
    resp
}

/// The generic `401`: the body does not say which check failed. The reason is
/// logged with `tracing`.
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        reject(&self, false)
    }
}

#[derive(Clone)]
struct Inner {
    verifier: Arc<Verifier>,
    rule: RouteRule,
    max_body_bytes: usize,
    detailed_errors: bool,
}

/// A [`tower::Layer`] that puts one route behind http-authz/v1 verification.
#[derive(Clone)]
pub struct AuthzLayer {
    inner: Arc<Inner>,
}

impl AuthzLayer {
    /// A layer for `rule`, with the body limit [`DEFAULT_MAX_BODY_BYTES`].
    #[must_use]
    pub fn new(verifier: Arc<Verifier>, rule: RouteRule) -> Self {
        Self {
            inner: Arc::new(Inner {
                verifier,
                rule,
                max_body_bytes: DEFAULT_MAX_BODY_BYTES,
                detailed_errors: false,
            }),
        }
    }

    /// Set the largest request body the layer buffers, in bytes. A larger
    /// body is rejected with `413` before the handler runs. The layer must
    /// hold the whole body to check `tk_body_digest`, so keep this bound to
    /// what the route needs.
    #[must_use]
    pub fn with_max_body_bytes(mut self, max_body_bytes: usize) -> Self {
        Arc::make_mut(&mut self.inner).max_body_bytes = max_body_bytes;
        self
    }

    /// Put the specific error code and message in rejection bodies
    /// (`{"error": <code>, "message": <text>}`). Off by default, so a
    /// rejection does not tell the caller which check failed. Use it for
    /// debugging only.
    #[must_use]
    pub fn with_detailed_errors(mut self, detailed_errors: bool) -> Self {
        Arc::make_mut(&mut self.inner).detailed_errors = detailed_errors;
        self
    }
}

impl<S> Layer<S> for AuthzLayer {
    type Service = AuthzService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthzService {
            inner,
            layer: self.clone(),
        }
    }
}

/// The service that [`AuthzLayer`] wraps around a route.
#[derive(Clone)]
pub struct AuthzService<S> {
    inner: S,
    layer: AuthzLayer,
}

impl<S> Service<Request> for AuthzService<S>
where
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Infallible>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let layer = self.layer.clone();
        Box::pin(async move {
            match authorize(&layer.inner, req).await {
                Ok(req) => inner.call(req).await,
                Err(resp) => Ok(resp),
            }
        })
    }
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, AuthError> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let (Some(v), None) = (values.next(), values.next()) else {
        return Err(AuthError::MissingAuthorization);
    };
    let v = v.to_str().map_err(|_| AuthError::MissingAuthorization)?;
    match v.split_once(' ') {
        Some((scheme, token)) if scheme.eq_ignore_ascii_case("bearer") && !token.is_empty() => {
            Ok(token)
        }
        _ => Err(AuthError::MissingAuthorization),
    }
}

async fn authorize(layer: &Inner, req: Request) -> Result<Request, Response> {
    let now = unix_now();
    let (mut parts, body) = req.into_parts();
    let detailed = layer.detailed_errors;
    let token = bearer_token(&parts.headers).map_err(|e| reject(&e, detailed))?;
    let claims = layer
        .verifier
        .verify_token(token, now)
        .map_err(|e| reject(&e, detailed))?;
    let body = axum::body::to_bytes(body, layer.max_body_bytes)
        .await
        .map_err(|_| {
            let message = format!("body exceeds {} bytes", layer.max_body_bytes);
            reject_with(
                StatusCode::PAYLOAD_TOO_LARGE,
                "body_too_large",
                "payload_too_large",
                &message,
                detailed,
            )
        })?;
    layer
        .verifier
        .check_still_valid(&claims, unix_now())
        .map_err(|e| reject(&e, detailed))?;
    let uri = parts
        .extensions
        .get::<OriginalUri>()
        .map_or(&parts.uri, |o| &o.0);
    super::bind_request(
        &claims,
        &layer.rule,
        parts.method.as_str(),
        uri.path(),
        uri.query(),
        &parts.headers,
        &body,
    )
    .map_err(|e| reject(&e, detailed))?;
    parts.extensions.insert(Verified(claims));
    Ok(Request::from_parts(parts, Body::from(body)))
}

/// A layer for `rule`, for `MethodRouter::route_layer` or `Router::layer`.
#[must_use]
pub fn route_layer(verifier: Arc<Verifier>, rule: RouteRule) -> AuthzLayer {
    AuthzLayer::new(verifier, rule)
}

/// Put `route` behind http-authz/v1 verification with `rule`.
pub fn protect<S>(
    route: MethodRouter<S>,
    verifier: Arc<Verifier>,
    rule: RouteRule,
) -> MethodRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    route.route_layer(route_layer(verifier, rule))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_authz::tests::{
        BODY, PATH, QUERY, TestKey, cfg, claims, header as signed_header, rule, sent_headers,
    };
    use axum::Router;
    use axum::routing::post;
    use serde_json::json;
    use tower::ServiceExt;

    fn verifier(k: &TestKey) -> Arc<Verifier> {
        Arc::new(Verifier::new(cfg(), Arc::new(k.keys())).unwrap())
    }

    fn app(k: &TestKey) -> Router {
        let route = protect(
            post(
                |axum::Extension(Verified(c)): axum::Extension<Verified>| async move {
                    format!("{} {}", c.iss, c.sub)
                },
            ),
            verifier(k),
            rule(),
        );
        Router::new().route(PATH, route)
    }

    fn token(k: &TestKey) -> String {
        let now = unix_now();
        let mut c = claims();
        c["iat"] = json!(now);
        c["exp"] = json!(now + 60);
        k.sign(&signed_header(k), &c)
    }

    fn request(jwt: &str, headers: &HeaderMap) -> Request {
        let mut b = axum::http::Request::builder()
            .method("POST")
            .uri(format!("{PATH}?{QUERY}"))
            .header(header::AUTHORIZATION, format!("Bearer {jwt}"));
        for (name, value) in headers {
            b = b.header(name, value);
        }
        b.body(Body::from(BODY)).unwrap()
    }

    async fn status(app: &Router, jwt: &str, headers: &HeaderMap) -> StatusCode {
        app.clone()
            .oneshot(request(jwt, headers))
            .await
            .unwrap()
            .status()
    }

    async fn error_body(resp: Response) -> serde_json::Value {
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn unsigned_request() -> Request {
        axum::http::Request::builder()
            .method("POST")
            .uri(PATH)
            .body(Body::empty())
            .unwrap()
    }

    fn layered(layer: AuthzLayer) -> Router {
        Router::new()
            .route(PATH, post(|| async { "ok" }))
            .layer(layer)
    }

    #[tokio::test]
    async fn repeats_are_accepted_and_the_handler_gets_iss_and_sub() {
        let k = TestKey::new();
        let app = app(&k);
        let jwt = token(&k);
        for _ in 0..3 {
            let resp = app
                .clone()
                .oneshot(request(&jwt, &sent_headers()))
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
            assert_eq!(
                body,
                "urn:turnkey:org:sub_org_1 urn:turnkey:user:user_1".as_bytes()
            );
        }
    }

    #[tokio::test]
    async fn bound_headers_are_checked_by_the_layer() {
        let k = TestKey::new();
        let app = app(&k);
        let jwt = token(&k);
        let mut h = sent_headers();
        h.insert("x-api-version", "2026-03-10".parse().unwrap());
        assert_eq!(status(&app, &jwt, &h).await, StatusCode::UNAUTHORIZED);
        let mut h = sent_headers();
        h.append("accept", "text/plain".parse().unwrap());
        assert_eq!(status(&app, &jwt, &h).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn nested_route_binds_the_full_request_path() {
        let k = TestKey::new();
        let (prefix, rest) = ("/v1", "/resources/12345/actions");
        assert_eq!(format!("{prefix}{rest}"), PATH);
        let route = protect(post(|| async { "ok" }), verifier(&k), rule());
        let app = Router::new().nest(prefix, Router::new().route(rest, route));
        assert_eq!(
            status(&app, &token(&k), &sent_headers()).await,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn oversized_body_is_rejected_with_413_payload_too_large() {
        let k = TestKey::new();
        let app = layered(route_layer(verifier(&k), rule()).with_max_body_bytes(8));
        let resp = app
            .oneshot(request(&token(&k), &sent_headers()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            error_body(resp).await,
            json!({"error": "payload_too_large"})
        );
    }

    #[tokio::test]
    async fn default_errors_do_not_name_the_failed_check() {
        let k = TestKey::new();
        let app = layered(route_layer(verifier(&k), rule()));
        let other = TestKey::new();
        let mut h = sent_headers();
        h.remove("accept");
        for req in [
            unsigned_request(),
            request(&token(&other), &sent_headers()),
            request(&token(&k), &h),
        ] {
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(resp.headers()[header::WWW_AUTHENTICATE], "Bearer");
            assert_eq!(error_body(resp).await, json!({"error": "unauthorized"}));
        }
    }

    #[tokio::test]
    async fn detailed_errors_name_the_failed_check() {
        let k = TestKey::new();
        let layer = route_layer(verifier(&k), rule()).with_detailed_errors(true);
        let app = layered(layer.clone());

        let resp = app.clone().oneshot(unsigned_request()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(resp.headers()[header::WWW_AUTHENTICATE], "Bearer");
        let body = error_body(resp).await;
        assert_eq!(body["error"], "missing_authorization");
        assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()));

        let mut h = sent_headers();
        h.remove("accept");
        let resp = app.clone().oneshot(request(&token(&k), &h)).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(resp.headers()[header::WWW_AUTHENTICATE], "Bearer");
        let body = error_body(resp).await;
        assert_eq!(body["error"], "request_mismatch");
        assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()));

        let small = layered(layer.with_max_body_bytes(8));
        let resp = small
            .oneshot(request(&token(&k), &sent_headers()))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            error_body(resp).await,
            json!({"error": "body_too_large", "message": "body exceeds 8 bytes"})
        );
    }

    /// A body that yields `BODY` only after `release` fires.
    struct DelayedBody(Option<tokio::sync::oneshot::Receiver<()>>);

    impl http_body::Body for DelayedBody {
        type Data = axum::body::Bytes;
        type Error = Infallible;

        fn poll_frame(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            let Some(rx) = self.0.as_mut() else {
                return Poll::Ready(None);
            };
            match Pin::new(rx).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => {
                    self.0 = None;
                    Poll::Ready(Some(Ok(http_body::Frame::data(BODY.into()))))
                }
            }
        }
    }

    #[tokio::test]
    async fn token_that_expires_while_the_body_arrives_is_rejected() {
        let k = TestKey::new();
        let app = layered(route_layer(verifier(&k), rule()).with_detailed_errors(true));
        let now = unix_now();
        let mut c = claims();
        c["iat"] = json!(now);
        c["exp"] = json!(now + 1);
        let jwt = k.sign(&signed_header(&k), &c);
        let (release, rx) = tokio::sync::oneshot::channel();
        let (parts, _) = request(&jwt, &sent_headers()).into_parts();
        let req = Request::from_parts(parts, Body::new(DelayedBody(Some(rx))));
        let pending = tokio::spawn(app.oneshot(req));
        while unix_now() <= now + 1 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        release.send(()).unwrap();
        let resp = pending.await.unwrap().unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error_body(resp).await["error"], "expired");
    }

    #[tokio::test]
    async fn body_limit_can_be_raised() {
        let k = TestKey::new();
        let big = vec![b'x'; DEFAULT_MAX_BODY_BYTES + 1];
        let mut c = claims();
        let now = unix_now();
        c["iat"] = json!(now);
        c["exp"] = json!(now + 60);
        c["tk_body_digest"] = json!(crate::http_authz::digest::sha256_digest(&big));
        let jwt = k.sign(&signed_header(&k), &c);
        let req = || {
            let (parts, _) = request(&jwt, &sent_headers()).into_parts();
            Request::from_parts(parts, Body::from(big.clone()))
        };
        let default = layered(route_layer(verifier(&k), rule()));
        assert_eq!(
            default.oneshot(req()).await.unwrap().status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );
        let raised = layered(
            route_layer(verifier(&k), rule()).with_max_body_bytes(DEFAULT_MAX_BODY_BYTES + 1),
        );
        assert_eq!(
            raised.oneshot(req()).await.unwrap().status(),
            StatusCode::OK
        );
    }

    #[test]
    fn bearer_token_accepts_one_nonempty_bearer_field() {
        let mut h = HeaderMap::new();
        assert_eq!(bearer_token(&h), Err(AuthError::MissingAuthorization));
        h.insert(header::AUTHORIZATION, "Bearer abc".parse().unwrap());
        assert_eq!(bearer_token(&h), Ok("abc"));
        h.insert(header::AUTHORIZATION, "bearer abc".parse().unwrap());
        assert_eq!(bearer_token(&h), Ok("abc"));
        h.insert(header::AUTHORIZATION, "Basic abc".parse().unwrap());
        assert_eq!(bearer_token(&h), Err(AuthError::MissingAuthorization));
        h.insert(header::AUTHORIZATION, "Bearer ".parse().unwrap());
        assert_eq!(bearer_token(&h), Err(AuthError::MissingAuthorization));
        h.insert(header::AUTHORIZATION, "Bearer a".parse().unwrap());
        h.append(header::AUTHORIZATION, "Bearer b".parse().unwrap());
        assert_eq!(bearer_token(&h), Err(AuthError::MissingAuthorization));
    }
}
