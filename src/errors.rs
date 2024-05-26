use reqwest::Error as ReqwestError;
use thiserror::Error;

#[derive(Debug)]
pub enum TurnkeyError {
    MethodError(TurnkeyResponseError),
    StampError(StampError),
    HttpError(ReqwestError),
    OtherError(String),
}
#[derive(Debug, Clone)]
pub struct TurnkeyResponseError {
    pub code: u32,
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Clone)]
pub struct ErrorDetail {
    pub type_field: String,
    pub field_violations: Vec<FieldViolation>,
}

#[derive(Debug, Clone)]
pub struct FieldViolation {
    pub field: String,
    pub description: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum StampError {
    #[error("cannot decode private key: invalid hex")]
    InvalidPrivateKeyString(#[from] hex::FromHexError),
    #[error("cannot load private key: invalid bytes")]
    InvalidPrivateKeyBytes,
}
