#![allow(missing_docs, clippy::expect_used, clippy::panic)]

use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use http_body_util::BodyExt;
use serde::Serialize;
use tvc_axum::QosJson;

#[derive(Serialize)]
struct Sample {
    message: String,
    #[serde(with = "qos_json::string_or_numeric")]
    count: u64,
}

#[tokio::test]
async fn qos_json_body_matches_qos_json_to_vec() {
    let value = Sample {
        message: "hello".to_owned(),
        count: 42,
    };
    let expected = qos_json::to_vec(&value).expect("qos_json should serialize");

    let response = QosJson(value).into_response();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE),
        Some(&header::HeaderValue::from_static("application/json"))
    );
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes()
        .to_vec();
    assert_eq!(body, expected);
}
