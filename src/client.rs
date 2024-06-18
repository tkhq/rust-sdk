use crate::gen::external::activity::v1 as activity;
use crate::gen::services::coordinator::public::v1 as api;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::FieldBytes;
use reqwest::Client;
use reqwest::Error as ReqwestError;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub type TurnkeyResult<T> = std::result::Result<T, TurnkeyError>;

#[derive(Error, Debug)]
pub enum TurnkeyError {
    #[error("failed to construct stamper: {0}")]
    StampError(StampError),
    #[error("failed to make http request: {0}")]
    HttpError(ReqwestError),
    #[error("failed: {0}")]
    OtherError(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum StampError {
    #[error("cannot decode private key: invalid hex")]
    InvalidPrivateKeyString(#[from] hex::FromHexError),
    #[error("cannot load private key: invalid bytes")]
    InvalidPrivateKeyBytes,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiStamp {
    public_key: String,
    signature: String,
    scheme: String,
}

pub struct Stamper {
    pub public_key_hex: String,
    pub signing_key: SigningKey,
}

impl Stamper {
    pub fn new(public_key_hex: String, private_key_hex: String) -> TurnkeyResult<Self> {
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| TurnkeyError::StampError(StampError::InvalidPrivateKeyString(e)))?;

        let signing_key: SigningKey =
            SigningKey::from_bytes(FieldBytes::from_slice(&private_key_bytes))
                .map_err(|_| TurnkeyError::StampError(StampError::InvalidPrivateKeyBytes))?;

        Ok(Self {
            public_key_hex,
            signing_key,
        })
    }

    pub fn stamp_raw_body(&self, body: &[u8]) -> TurnkeyResult<String> {
        let sig: Signature = self.signing_key.sign(body);
        let stamp = ApiStamp {
            public_key: self.public_key_hex.clone(),
            signature: hex::encode(sig.to_der()),
            scheme: "SIGNATURE_SCHEME_TK_API_P256".to_string(),
        };

        let json_stamp = serde_json::to_string(&stamp).unwrap();

        Ok(BASE64_URL_SAFE_NO_PAD.encode(json_stamp.as_bytes()))
    }
}

pub struct TurnkeyClient {
    stamper: Stamper,
    base_url: String,
    client: Client,
}

impl TurnkeyClient {
    pub fn new(base_url: String, stamper: Stamper) -> Self {
        Self {
            base_url,
            stamper,
            client: Client::new(),
        }
    }

    pub fn new_from_env() -> TurnkeyResult<Self> {
        let public_key_hex = env::var("TURNKEY_API_PUBLIC_KEY")
            .map_err(|e| TurnkeyError::OtherError(e.to_string()))?;
        let private_key_hex = env::var("TURNKEY_API_PRIVATE_KEY")
            .map_err(|e| TurnkeyError::OtherError(e.to_string()))?;
        let base_url =
            env::var("TURNKEY_BASE_URL").map_err(|e| TurnkeyError::OtherError(e.to_string()))?;

        let stamper = Stamper::new(public_key_hex, private_key_hex)?;

        Ok(Self {
            base_url,
            stamper,
            client: Client::new(),
        })
    }

    pub async fn request<RPC>(&self, request_input: RPC::Request) -> TurnkeyResult<RPC::Response>
    where
        RPC: TurnkeyRpc,
        RPC::Request: Serialize,
        RPC::Response: DeserializeOwned,
    {
        let body = serde_json::to_value(request_input).expect("serilization to succeed");
        let resp = self.raw_request(RPC::uri(), body).await?;
        Ok(resp)
    }

    pub fn request_timestamp_ms(&self) -> String {
        let start = SystemTime::now();
        let ts = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let ts = ts.as_millis();
        return ts.to_string();
    }

    async fn raw_request<O>(&self, uri: String, body: Value) -> TurnkeyResult<O>
    where
        O: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, uri);

        let body_str =
            serde_json::to_string(&body).map_err(|e| TurnkeyError::OtherError(e.to_string()))?;
        let stamp = self.stamper.stamp_raw_body(body_str.as_bytes())?;

        let request = self
            .client
            .post(&url)
            .header("X-Stamp", stamp)
            .body(body_str.clone())
            .build()
            .map_err(TurnkeyError::HttpError)?;

        log::debug!(
            "sending turnkey post request, url: {}, body: {}",
            request.url(),
            body_str
        );

        let response = self
            .client
            .execute(request)
            .await
            .map_err(TurnkeyError::HttpError)?;

        let status_response = response.error_for_status_ref().map(|_| ());
        match status_response {
            Ok(_) => match response.json::<O>().await {
                Ok(parsed) => Ok(parsed),
                Err(e) => Err(TurnkeyError::OtherError(format!(
                    "failed to parse response: {}",
                    e.to_string()
                ))),
            },
            Err(e) => {
                let body = response.text().await.map_err(TurnkeyError::HttpError)?;
                log::error!("request failed: {}, body: {}", e.status().unwrap(), body);
                Err(TurnkeyError::HttpError(e))
            }
        }
    }
}

pub trait TurnkeyRpc {
    fn uri() -> String;
    type Request;
    type Response;
}

macro_rules! declare_rpc {
    ($name:ident, $path:literal, $req:ty, $resp:ty) => {
        pub struct $name {}

        impl TurnkeyRpc for $name {
            fn uri() -> String {
                $path.to_owned()
            }

            type Request = $req;
            type Response = $resp;
        }
    };
}

declare_rpc!(
    GetWhoami,
    "/public/v1/query/whoami",
    api::GetWhoamiRequest,
    api::GetWhoamiResponse
);

declare_rpc!(
    GetSubOrgIds,
    "/public/v1/query/list_suborgs",
    api::GetSubOrgIdsRequest,
    api::GetSubOrgIdsResponse
);

declare_rpc!(
    GetOrganization,
    "/public/v1/query/get_organization",
    api::GetOrganizationRequest,
    api::GetOrganizationResponse
);

declare_rpc!(
    GetActivity,
    "/public/v1/query/get_activity",
    api::GetActivityRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetActivities,
    "/public/v1/query/list_activities",
    api::GetActivitiesRequest,
    api::GetActivitiesResponse
);

declare_rpc!(
    ApproveActivity,
    "/public/v1/submit/approve_activity",
    activity::ApproveActivityRequest,
    api::ActivityResponse
);

declare_rpc!(
    RejectActivity,
    "/public/v1/submit/reject_activity",
    activity::RejectActivityRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetUser,
    "/public/v1/query/get_user",
    api::GetUserRequest,
    api::GetUserResponse
);

declare_rpc!(
    GetUsers,
    "/public/v1/query/list_users",
    api::GetUsersRequest,
    api::GetUsersResponse
);

declare_rpc!(
    DeleteUsers,
    "/public/v1/submit/delete_users",
    activity::DeleteUsersRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreatePolicy,
    "/public/v1/submit/create_policy",
    activity::CreatePolicyRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreatePolicies,
    "/public/v1/submit/create_policies",
    activity::CreatePoliciesRequest,
    api::ActivityResponse
);

declare_rpc!(
    UpdatePolicy,
    "/public/v1/submit/update_policy",
    activity::UpdatePolicyRequest,
    api::ActivityResponse
);

declare_rpc!(
    DeletePolicy,
    "/public/v1/submit/delete_policy",
    activity::DeletePolicyRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetPolicies,
    "/public/v1/query/list_policies",
    api::GetPoliciesRequest,
    api::GetPoliciesResponse
);

declare_rpc!(
    GetPolicy,
    "/public/v1/query/get_policy",
    api::GetPolicyRequest,
    api::GetPolicyResponse
);

declare_rpc!(
    CreateReadOnlySession,
    "/public/v1/submit/create_read_only_session",
    activity::CreateReadOnlySessionRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreatePrivateKeys,
    "/public/v1/submit/create_private_keys",
    activity::CreatePrivateKeysRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetPrivateKey,
    "/public/v1/query/get_private_key",
    api::GetPrivateKeyRequest,
    api::GetPrivateKeyResponse
);

declare_rpc!(
    GetPrivateKeys,
    "/public/v1/query/list_private_keys",
    api::GetPrivateKeysRequest,
    api::GetPrivateKeysResponse
);

declare_rpc!(
    CreateApiKeys,
    "/public/v1/submit/create_api_keys",
    activity::CreateApiKeysRequest,
    api::ActivityResponse
);

declare_rpc!(
    DeleteApiKeys,
    "/public/v1/submit/delete_api_keys",
    activity::DeleteApiKeysRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetApiKeys,
    "/public/v1/query/get_api_keys",
    api::GetApiKeysRequest,
    api::GetApiKeysResponse
);

declare_rpc!(
    GetApiKey,
    "/public/v1/query/get_api_key",
    api::GetApiKeyRequest,
    api::GetApiKeyResponse
);

declare_rpc!(
    CreateAuthenticators,
    "/public/v1/submit/create_authenticators",
    activity::CreateAuthenticatorsRequest,
    api::ActivityResponse
);

declare_rpc!(
    DeleteAuthenticators,
    "/public/v1/submit/delete_authenticators",
    activity::DeleteAuthenticatorsRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetAuthenticators,
    "/public/v1/query/get_authenticators",
    api::GetAuthenticatorsRequest,
    api::GetAuthenticatorsResponse
);

declare_rpc!(
    GetAuthenticator,
    "/public/v1/query/get_authenticator",
    api::GetAuthenticatorRequest,
    api::GetAuthenticatorResponse
);

declare_rpc!(
    CreateInvitations,
    "/public/v1/submit/create_invitations",
    activity::CreateInvitationsRequest,
    api::ActivityResponse
);

declare_rpc!(
    DeleteInvitation,
    "/public/v1/submit/delete_invitation",
    activity::DeleteInvitationRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreateUsers,
    "/public/v1/submit/create_users",
    activity::CreateUsersRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreateApiOnlyUsers,
    "/public/v1/submit/create_api_only_users",
    activity::CreateApiOnlyUsersRequest,
    api::ActivityResponse
);

declare_rpc!(
    UpdateUser,
    "/public/v1/submit/update_user",
    activity::UpdateUserRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreateUserTag,
    "/public/v1/submit/create_user_tag",
    activity::CreateUserTagRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreatePrivateKeyTag,
    "/public/v1/submit/create_private_key_tag",
    activity::CreatePrivateKeyTagRequest,
    api::ActivityResponse
);

declare_rpc!(
    UpdateUserTag,
    "/public/v1/submit/update_user_tag",
    activity::UpdateUserTagRequest,
    api::ActivityResponse
);

declare_rpc!(
    ListUserTags,
    "/public/v1/query/list_user_tags",
    api::ListUserTagsRequest,
    api::ListUserTagsResponse
);

declare_rpc!(
    DeleteUserTags,
    "/public/v1/submit/delete_user_tags",
    activity::DeleteUserTagsRequest,
    api::ActivityResponse
);

declare_rpc!(
    UpdatePrivateKeyTag,
    "/public/v1/submit/update_private_key_tag",
    activity::UpdatePrivateKeyTagRequest,
    api::ActivityResponse
);

declare_rpc!(
    ListPrivateKeyTags,
    "/public/v1/query/list_private_key_tags",
    api::ListPrivateKeyTagsRequest,
    api::ListPrivateKeyTagsResponse
);

declare_rpc!(
    DeletePrivateKeyTags,
    "/public/v1/submit/delete_private_key_tags",
    activity::DeletePrivateKeyTagsRequest,
    api::ActivityResponse
);

declare_rpc!(
    SignRawPayload,
    "/public/v1/submit/sign_raw_payload",
    activity::SignRawPayloadRequest,
    api::ActivityResponse
);

declare_rpc!(
    SignRawPayloads,
    "/public/v1/submit/sign_raw_payloads",
    activity::SignRawPayloadsRequest,
    api::ActivityResponse
);

declare_rpc!(
    SignTransaction,
    "/public/v1/submit/sign_transaction",
    activity::SignTransactionRequest,
    api::ActivityResponse
);

declare_rpc!(
    UpdateRootQuorum,
    "/public/v1/submit/update_root_quorum",
    activity::UpdateRootQuorumRequest,
    api::ActivityResponse
);

declare_rpc!(
    CreateWallet,
    "/public/v1/submit/create_wallet",
    activity::CreateWalletRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetWallets,
    "/public/v1/query/list_wallets",
    api::GetWalletsRequest,
    api::GetWalletsResponse
);

declare_rpc!(
    GetWallet,
    "/public/v1/query/get_wallet",
    api::GetWalletRequest,
    api::GetWalletResponse
);

declare_rpc!(
    CreateWalletAccounts,
    "/public/v1/submit/create_wallet_accounts",
    activity::CreateWalletAccountsRequest,
    api::ActivityResponse
);

declare_rpc!(
    GetWalletAccounts,
    "/public/v1/query/list_wallet_accounts",
    api::GetWalletAccountsRequest,
    api::GetWalletAccountsResponse
);

declare_rpc!(
    CreateSubOrganization,
    "/public/v1/submit/create_sub_organization",
    activity::CreateSubOrganizationRequest,
    api::ActivityResponse
);

declare_rpc!(
    InitUserEmailRecovery,
    "/public/v1/submit/init_user_email_recovery",
    activity::InitUserEmailRecoveryRequest,
    api::ActivityResponse
);

declare_rpc!(
    RecoverUser,
    "/public/v1/submit/recover_user",
    activity::RecoverUserRequest,
    api::ActivityResponse
);

declare_rpc!(
    SetOrganizationFeature,
    "/public/v1/submit/set_organization_feature",
    activity::SetOrganizationFeatureRequest,
    api::ActivityResponse
);

declare_rpc!(
    RemoveOrganizationFeature,
    "/public/v1/submit/remove_organization_feature",
    activity::RemoveOrganizationFeatureRequest,
    api::ActivityResponse
);

declare_rpc!(
    ExportPrivateKey,
    "/public/v1/submit/export_private_key",
    activity::ExportPrivateKeyRequest,
    api::ActivityResponse
);

declare_rpc!(
    ExportWallet,
    "/public/v1/submit/export_wallet",
    activity::ExportWalletRequest,
    api::ActivityResponse
);

declare_rpc!(
    EmailAuth,
    "/public/v1/submit/email_auth",
    activity::EmailAuthRequest,
    api::ActivityResponse
);

declare_rpc!(
    ExportWalletAccount,
    "/public/v1/submit/export_wallet_account",
    activity::ExportWalletAccountRequest,
    api::ActivityResponse
);

declare_rpc!(
    InitImportWallet,
    "/public/v1/submit/init_import_wallet",
    activity::InitImportWalletRequest,
    api::ActivityResponse
);

declare_rpc!(
    ImportWallet,
    "/public/v1/submit/import_wallet",
    activity::ImportWalletRequest,
    api::ActivityResponse
);

declare_rpc!(
    InitImportPrivateKey,
    "/public/v1/submit/init_import_private_key",
    activity::InitImportPrivateKeyRequest,
    api::ActivityResponse
);

declare_rpc!(
    ImportPrivateKey,
    "/public/v1/submit/import_private_key",
    activity::ImportPrivateKeyRequest,
    api::ActivityResponse
);
