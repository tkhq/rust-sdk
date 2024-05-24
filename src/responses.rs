use serde::{Deserialize, Serialize};

use crate::models::{AddressFormat, CreatedAt, Curve, UpdatedAt};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetWalletResponse {
    wallet: Wallet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub wallet_id: String,
    pub wallet_name: String,
    pub created_at: CreatedAt,
    pub updated_at: UpdatedAt,
    pub exported: bool,
    pub imported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletsAccountsResponse {
    accounts: Vec<WalletAccount>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletAccount {
    pub organzation_id: String,
    pub wallet_id: String,
    pub curve: Curve,
    pub path_format: String,
    pub path: String,
    pub address_format: AddressFormat,
    pub address: String,
    pub created_at: CreatedAt,
    pub updated_at: UpdatedAt,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletsResponse {
    wallets: Vec<Wallet>,
}
