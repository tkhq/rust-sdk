use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAt {
    pub seconds: String,
    pub nanos: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedAt {
    pub seconds: String,
    pub nanos: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AddressFormat {
    #[serde(rename = "ADDRESS_FORMAT_UNCOMPRESSED")]
    AddressFormatUncompressed,
    #[serde(rename = "ADDRESS_FORMAT_COMPRESSED")]
    AddressFormatCompressed,
    #[serde(rename = "ADDRESS_FORMAT_ETHEREUM")]
    AddressFormatEthereum,
    #[serde(rename = "ADDRESS_FORMAT_SOLANA")]
    AddressFormatSolana,
    #[serde(rename = "ADDRESS_FORMAT_COSMOS")]
    AddressFormatCosmos,
    #[serde(rename = "ADDRESS_FORMAT_TRON")]
    AddressFormatTron,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Curve {
    #[serde(rename = "CURVE_SECP256K1")]
    CurveSecp256k1,
    #[serde(rename = "CURVE_ED25519")]
    CurveEd25519,
}
