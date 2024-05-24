use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::models::{AddressFormat, Curve};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetWalletRequest {
    pub organization_id: String,
    pub wallet_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletAccountsRequest {
    pub organization_id: String,
    pub wallet_id: String,
    pub pagination_options: Option<PaginationOptions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    pub limit: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListWalletsRequest {
    pub organization_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: CreateWalletIntent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletIntent {
    pub wallet_name: String,
    pub accounts: Vec<WalletAccountParams>,
    #[serde(default = "default_mnemonic_length")]
    pub mnemonic_length: Option<MnemonicLength>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum MnemonicLength {
    Twelve = 12,
    Fifteen = 15,
    Eighteen = 18,
    TwentyOne = 21,
    TwentyFour = 24,
}

fn default_mnemonic_length() -> Option<MnemonicLength> {
    Some(MnemonicLength::Twelve)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletAccountParams {
    pub curve: Curve,
    pub path_format: String,
    pub path: String,
    pub address_format: AddressFormat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletAccountsRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: CreateWalletAccountsIntent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletAccountsIntent {
    pub wallet_id: String,
    pub accounts: Vec<WalletAccountParams>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportWalletRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: ExportWalletIntent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportWalletIntent {
    pub wallet_id: String,
    pub target_public_key: String,
    pub language: Option<Language>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Language {
    #[serde(rename = "MNEMONIC_LANGUAGE_ENGLISH")]
    MnemonicLanguageEnglish,
    #[serde(rename = "MNEMONIC_LANGUAGE_SIMPLIFIED_CHINESE")]
    MnemonicLanguageSimplifiedChinese,
    #[serde(rename = "MNEMONIC_LANGUAGE_TRADITIONAL_CHINESE")]
    MnemonicLanguageTraditionalChinese,
    #[serde(rename = "MNEMONIC_LANGUAGE_CZECH")]
    MnemonicLanguageCzech,
    #[serde(rename = "MNEMONIC_LANGUAGE_FRENCH")]
    MnemonicLanguageFrench,
    #[serde(rename = "MNEMONIC_LANGUAGE_ITALIAN")]
    MnemonicLanguageItalian,
    #[serde(rename = "MNEMONIC_LANGUAGE_JAPANESE")]
    MnemonicLanguageJapanese,
    #[serde(rename = "MNEMONIC_LANGUAGE_KOREAN")]
    MnemonicLanguageKorean,
    #[serde(rename = "MNEMONIC_LANGUAGE_SPANISH")]
    MnemonicLanguageSpanish,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportWalletAccountRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: ExportWalletAccountIntent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportWalletAccountIntent {
    pub address: String,
    pub target_public_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportWalletRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: ImportWalletIntent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportWalletIntent {
    pub user_id: String,
    pub wallet_name: String,
    pub encrypted_bundle: String,
    pub accounts: Vec<WalletAccountParams>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitImportWalletRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: InitImportWalletIntent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitImportWalletIntent {
    pub user_id: String,
}
