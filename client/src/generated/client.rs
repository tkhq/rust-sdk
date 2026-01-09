//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use crate::generated::external::activity::v1 as external_activity;
use crate::generated::immutable::activity::v1 as immutable_activity;
use crate::generated::services::coordinator::public::v1 as coordinator;
use crate::{ActivityResult, Stamp, TurnkeyClient, TurnkeyClientError};
impl<S: Stamp> TurnkeyClient<S> {
    /// Who am I?
    ///
    /// Get basic information about your current API or WebAuthN user and their organization. Affords sub-organization look ups via parent organization for WebAuthN or API key users.
    pub async fn get_whoami(
        &self,
        request: coordinator::GetWhoamiRequest,
    ) -> Result<coordinator::GetWhoamiResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/whoami".to_string())
            .await
    }
    /// Get sub-organizations
    ///
    /// Get all suborg IDs associated given a parent org ID and an optional filter.
    pub async fn get_sub_org_ids(
        &self,
        request: coordinator::GetSubOrgIdsRequest,
    ) -> Result<coordinator::GetSubOrgIdsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_suborgs".to_string())
            .await
    }
    /// Get verified sub-organizations
    ///
    /// Get all email or phone verified suborg IDs associated given a parent org ID.
    pub async fn get_verified_sub_org_ids(
        &self,
        request: coordinator::GetVerifiedSubOrgIdsRequest,
    ) -> Result<coordinator::GetVerifiedSubOrgIdsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/list_verified_suborgs".to_string(),
        )
        .await
    }
    /// Get activity
    ///
    /// Get details about an activity.
    pub async fn get_activity(
        &self,
        request: coordinator::GetActivityRequest,
    ) -> Result<coordinator::ActivityResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_activity".to_string())
            .await
    }
    /// List activities
    ///
    /// List all activities within an organization.
    pub async fn get_activities(
        &self,
        request: coordinator::GetActivitiesRequest,
    ) -> Result<coordinator::GetActivitiesResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_activities".to_string())
            .await
    }
    /// Approve activity
    ///
    /// Approve an activity.
    pub async fn approve_activity(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ApproveActivityIntent,
    ) -> Result<external_activity::Activity, TurnkeyClientError> {
        let request = external_activity::ApproveActivityRequest {
            r#type: "ACTIVITY_TYPE_APPROVE_ACTIVITY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        self.process_activity(&request, "/public/v1/submit/approve_activity".to_string())
            .await
    }
    /// Reject activity
    ///
    /// Reject an activity.
    pub async fn reject_activity(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::RejectActivityIntent,
    ) -> Result<external_activity::Activity, TurnkeyClientError> {
        let request = external_activity::RejectActivityRequest {
            r#type: "ACTIVITY_TYPE_REJECT_ACTIVITY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        self.process_activity(&request, "/public/v1/submit/reject_activity".to_string())
            .await
    }
    /// Get user
    ///
    /// Get details about a user.
    pub async fn get_user(
        &self,
        request: coordinator::GetUserRequest,
    ) -> Result<coordinator::GetUserResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_user".to_string())
            .await
    }
    /// List users
    ///
    /// List all users within an organization.
    pub async fn get_users(
        &self,
        request: coordinator::GetUsersRequest,
    ) -> Result<coordinator::GetUsersResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_users".to_string())
            .await
    }
    /// Delete users
    ///
    /// Delete users within an organization.
    pub async fn delete_users(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteUsersIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteUsersResult>, TurnkeyClientError> {
        let request = external_activity::DeleteUsersRequest {
            r#type: "ACTIVITY_TYPE_DELETE_USERS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_users".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteUsersResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create policy
    ///
    /// Create a new policy.
    pub async fn create_policy(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePolicyIntentV3,
    ) -> Result<ActivityResult<immutable_activity::CreatePolicyResult>, TurnkeyClientError> {
        let request = external_activity::CreatePolicyRequest {
            r#type: "ACTIVITY_TYPE_CREATE_POLICY_V3".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_policy".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreatePolicyResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create policies
    ///
    /// Create new policies.
    pub async fn create_policies(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePoliciesIntent,
    ) -> Result<ActivityResult<immutable_activity::CreatePoliciesResult>, TurnkeyClientError> {
        let request = external_activity::CreatePoliciesRequest {
            r#type: "ACTIVITY_TYPE_CREATE_POLICIES".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_policies".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreatePoliciesResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update policy
    ///
    /// Update an existing policy.
    pub async fn update_policy(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdatePolicyIntentV2,
    ) -> Result<ActivityResult<immutable_activity::UpdatePolicyResultV2>, TurnkeyClientError> {
        let request = external_activity::UpdatePolicyRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_POLICY_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_policy".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdatePolicyResultV2(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete policy
    ///
    /// Delete an existing policy.
    pub async fn delete_policy(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePolicyIntent,
    ) -> Result<ActivityResult<immutable_activity::DeletePolicyResult>, TurnkeyClientError> {
        let request = external_activity::DeletePolicyRequest {
            r#type: "ACTIVITY_TYPE_DELETE_POLICY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_policy".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeletePolicyResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete policies
    ///
    /// Delete existing policies.
    pub async fn delete_policies(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePoliciesIntent,
    ) -> Result<ActivityResult<immutable_activity::DeletePoliciesResult>, TurnkeyClientError> {
        let request = external_activity::DeletePoliciesRequest {
            r#type: "ACTIVITY_TYPE_DELETE_POLICIES".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_policies".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeletePoliciesResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List policies
    ///
    /// List all policies within an organization.
    pub async fn get_policies(
        &self,
        request: coordinator::GetPoliciesRequest,
    ) -> Result<coordinator::GetPoliciesResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_policies".to_string())
            .await
    }
    /// Get policy
    ///
    /// Get details about a policy.
    pub async fn get_policy(
        &self,
        request: coordinator::GetPolicyRequest,
    ) -> Result<coordinator::GetPolicyResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_policy".to_string())
            .await
    }
    /// Create read only session
    ///
    /// Create a read only session for a user (valid for 1 hour).
    pub async fn create_read_only_session(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateReadOnlySessionIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateReadOnlySessionResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateReadOnlySessionRequest {
            r#type: "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_read_only_session".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateReadOnlySessionResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create read write session
    ///
    /// Create a read write session for a user.
    pub async fn create_read_write_session(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateReadWriteSessionIntentV2,
    ) -> Result<
        ActivityResult<immutable_activity::CreateReadWriteSessionResultV2>,
        TurnkeyClientError,
    > {
        let request = external_activity::CreateReadWriteSessionRequest {
            r#type: "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_read_write_session".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateReadWriteSessionResultV2(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Login with Oauth
    ///
    /// Create an Oauth session for a user.
    pub async fn oauth_login(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::OauthLoginIntent,
    ) -> Result<ActivityResult<immutable_activity::OauthLoginResult>, TurnkeyClientError> {
        let request = external_activity::OauthLoginRequest {
            r#type: "ACTIVITY_TYPE_OAUTH_LOGIN".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/oauth_login".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::OauthLoginResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Login with a stamp
    ///
    /// Create a session for a user through stamping client side (API key, wallet client, or passkey client).
    pub async fn stamp_login(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::StampLoginIntent,
    ) -> Result<ActivityResult<immutable_activity::StampLoginResult>, TurnkeyClientError> {
        let request = external_activity::StampLoginRequest {
            r#type: "ACTIVITY_TYPE_STAMP_LOGIN".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/stamp_login".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::StampLoginResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Login with OTP
    ///
    /// Create an OTP session for a user.
    pub async fn otp_login(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::OtpLoginIntent,
    ) -> Result<ActivityResult<immutable_activity::OtpLoginResult>, TurnkeyClientError> {
        let request = external_activity::OtpLoginRequest {
            r#type: "ACTIVITY_TYPE_OTP_LOGIN".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/otp_login".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::OtpLoginResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create private keys
    ///
    /// Create new private keys.
    pub async fn create_private_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePrivateKeysIntentV2,
    ) -> Result<ActivityResult<immutable_activity::CreatePrivateKeysResultV2>, TurnkeyClientError>
    {
        let request = external_activity::CreatePrivateKeysRequest {
            r#type: "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_private_keys".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreatePrivateKeysResultV2(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get private key
    ///
    /// Get details about a private key.
    pub async fn get_private_key(
        &self,
        request: coordinator::GetPrivateKeyRequest,
    ) -> Result<coordinator::GetPrivateKeyResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_private_key".to_string())
            .await
    }
    /// List private keys
    ///
    /// List all private keys within an organization.
    pub async fn get_private_keys(
        &self,
        request: coordinator::GetPrivateKeysRequest,
    ) -> Result<coordinator::GetPrivateKeysResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_private_keys".to_string())
            .await
    }
    /// Create API keys
    ///
    /// Add API keys to an existing user.
    pub async fn create_api_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateApiKeysIntentV2,
    ) -> Result<ActivityResult<immutable_activity::CreateApiKeysResult>, TurnkeyClientError> {
        let request = external_activity::CreateApiKeysRequest {
            r#type: "ACTIVITY_TYPE_CREATE_API_KEYS_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_api_keys".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateApiKeysResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete API keys
    ///
    /// Remove api keys from a user.
    pub async fn delete_api_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteApiKeysIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteApiKeysResult>, TurnkeyClientError> {
        let request = external_activity::DeleteApiKeysRequest {
            r#type: "ACTIVITY_TYPE_DELETE_API_KEYS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_api_keys".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteApiKeysResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get Oauth providers
    ///
    /// Get details about Oauth providers for a user.
    pub async fn get_oauth_providers(
        &self,
        request: coordinator::GetOauthProvidersRequest,
    ) -> Result<coordinator::GetOauthProvidersResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_oauth_providers".to_string())
            .await
    }
    /// Get API keys
    ///
    /// Get details about API keys for a user.
    pub async fn get_api_keys(
        &self,
        request: coordinator::GetApiKeysRequest,
    ) -> Result<coordinator::GetApiKeysResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_api_keys".to_string())
            .await
    }
    /// Get On Ramp transaction status
    ///
    /// Get the status of an on ramp transaction.
    pub async fn get_on_ramp_transaction_status(
        &self,
        request: coordinator::GetOnRampTransactionStatusRequest,
    ) -> Result<coordinator::GetOnRampTransactionStatusResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_onramp_transaction_status".to_string(),
        )
        .await
    }
    /// Get send transaction status
    ///
    /// Get the status of a send transaction request.
    pub async fn get_send_transaction_status(
        &self,
        request: coordinator::GetSendTransactionStatusRequest,
    ) -> Result<coordinator::GetSendTransactionStatusResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_send_transaction_status".to_string(),
        )
        .await
    }
    /// Get API key
    ///
    /// Get details about an API key.
    pub async fn get_api_key(
        &self,
        request: coordinator::GetApiKeyRequest,
    ) -> Result<coordinator::GetApiKeyResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_api_key".to_string())
            .await
    }
    /// Create authenticators
    ///
    /// Create authenticators to authenticate requests to Turnkey.
    pub async fn create_authenticators(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateAuthenticatorsIntentV2,
    ) -> Result<ActivityResult<immutable_activity::CreateAuthenticatorsResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateAuthenticatorsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_authenticators".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateAuthenticatorsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete authenticators
    ///
    /// Remove authenticators from a user.
    pub async fn delete_authenticators(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteAuthenticatorsIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteAuthenticatorsResult>, TurnkeyClientError>
    {
        let request = external_activity::DeleteAuthenticatorsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_AUTHENTICATORS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_authenticators".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteAuthenticatorsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get authenticators
    ///
    /// Get details about authenticators for a user.
    pub async fn get_authenticators(
        &self,
        request: coordinator::GetAuthenticatorsRequest,
    ) -> Result<coordinator::GetAuthenticatorsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_authenticators".to_string())
            .await
    }
    /// Get authenticator
    ///
    /// Get details about an authenticator.
    pub async fn get_authenticator(
        &self,
        request: coordinator::GetAuthenticatorRequest,
    ) -> Result<coordinator::GetAuthenticatorResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_authenticator".to_string())
            .await
    }
    /// Create invitations
    ///
    /// Create invitations to join an existing organization.
    pub async fn create_invitations(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateInvitationsIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateInvitationsResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateInvitationsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_INVITATIONS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_invitations".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateInvitationsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete invitation
    ///
    /// Delete an existing invitation.
    pub async fn delete_invitation(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteInvitationIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteInvitationResult>, TurnkeyClientError>
    {
        let request = external_activity::DeleteInvitationRequest {
            r#type: "ACTIVITY_TYPE_DELETE_INVITATION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_invitation".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteInvitationResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create users
    ///
    /// Create users in an existing organization.
    pub async fn create_users(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateUsersIntentV3,
    ) -> Result<ActivityResult<immutable_activity::CreateUsersResult>, TurnkeyClientError> {
        let request = external_activity::CreateUsersRequest {
            r#type: "ACTIVITY_TYPE_CREATE_USERS_V3".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_users".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateUsersResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update user
    ///
    /// Update a user in an existing organization.
    pub async fn update_user(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateUserResult>, TurnkeyClientError> {
        let request = external_activity::UpdateUserRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_user".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateUserResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update user's name
    ///
    /// Update a user's name in an existing organization.
    pub async fn update_user_name(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserNameIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateUserNameResult>, TurnkeyClientError> {
        let request = external_activity::UpdateUserNameRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER_NAME".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_user_name".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateUserNameResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update user's email
    ///
    /// Update a user's email in an existing organization.
    pub async fn update_user_email(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserEmailIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateUserEmailResult>, TurnkeyClientError> {
        let request = external_activity::UpdateUserEmailRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER_EMAIL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_user_email".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateUserEmailResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update user's phone number
    ///
    /// Update a user's phone number in an existing organization.
    pub async fn update_user_phone_number(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserPhoneNumberIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateUserPhoneNumberResult>, TurnkeyClientError>
    {
        let request = external_activity::UpdateUserPhoneNumberRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER_PHONE_NUMBER".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/update_user_phone_number".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateUserPhoneNumberResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create user tag
    ///
    /// Create a user tag and add it to users.
    pub async fn create_user_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateUserTagIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateUserTagResult>, TurnkeyClientError> {
        let request = external_activity::CreateUserTagRequest {
            r#type: "ACTIVITY_TYPE_CREATE_USER_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_user_tag".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateUserTagResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create private key tag
    ///
    /// Create a private key tag and add it to private keys.
    pub async fn create_private_key_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePrivateKeyTagIntent,
    ) -> Result<ActivityResult<immutable_activity::CreatePrivateKeyTagResult>, TurnkeyClientError>
    {
        let request = external_activity::CreatePrivateKeyTagRequest {
            r#type: "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_private_key_tag".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreatePrivateKeyTagResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update user tag
    ///
    /// Update human-readable name or associated users. Note that this activity is atomic: all of the updates will succeed at once, or all of them will fail.
    pub async fn update_user_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserTagIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateUserTagResult>, TurnkeyClientError> {
        let request = external_activity::UpdateUserTagRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_user_tag".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateUserTagResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List user tags
    ///
    /// List all user tags within an organization.
    pub async fn list_user_tags(
        &self,
        request: coordinator::ListUserTagsRequest,
    ) -> Result<coordinator::ListUserTagsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_user_tags".to_string())
            .await
    }
    /// Delete user tags
    ///
    /// Delete user tags within an organization.
    pub async fn delete_user_tags(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteUserTagsIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteUserTagsResult>, TurnkeyClientError> {
        let request = external_activity::DeleteUserTagsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_USER_TAGS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_user_tags".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteUserTagsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update private key tag
    ///
    /// Update human-readable name or associated private keys. Note that this activity is atomic: all of the updates will succeed at once, or all of them will fail.
    pub async fn update_private_key_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdatePrivateKeyTagIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdatePrivateKeyTagResult>, TurnkeyClientError>
    {
        let request = external_activity::UpdatePrivateKeyTagRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/update_private_key_tag".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdatePrivateKeyTagResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List private key tags
    ///
    /// List all private key tags within an organization.
    pub async fn list_private_key_tags(
        &self,
        request: coordinator::ListPrivateKeyTagsRequest,
    ) -> Result<coordinator::ListPrivateKeyTagsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/list_private_key_tags".to_string(),
        )
        .await
    }
    /// Delete private key tags
    ///
    /// Delete private key tags within an organization.
    pub async fn delete_private_key_tags(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePrivateKeyTagsIntent,
    ) -> Result<ActivityResult<immutable_activity::DeletePrivateKeyTagsResult>, TurnkeyClientError>
    {
        let request = external_activity::DeletePrivateKeyTagsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_private_key_tags".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeletePrivateKeyTagsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Sign raw payload
    ///
    /// Sign a raw payload.
    pub async fn sign_raw_payload(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SignRawPayloadIntentV2,
    ) -> Result<ActivityResult<immutable_activity::SignRawPayloadResult>, TurnkeyClientError> {
        let request = external_activity::SignRawPayloadRequest {
            r#type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/sign_raw_payload".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::SignRawPayloadResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Sign raw payloads
    ///
    /// Sign multiple raw payloads with the same signing parameters.
    pub async fn sign_raw_payloads(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SignRawPayloadsIntent,
    ) -> Result<ActivityResult<immutable_activity::SignRawPayloadsResult>, TurnkeyClientError> {
        let request = external_activity::SignRawPayloadsRequest {
            r#type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/sign_raw_payloads".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::SignRawPayloadsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Sign transaction
    ///
    /// Sign a transaction.
    pub async fn sign_transaction(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SignTransactionIntentV2,
    ) -> Result<ActivityResult<immutable_activity::SignTransactionResult>, TurnkeyClientError> {
        let request = external_activity::SignTransactionRequest {
            r#type: "ACTIVITY_TYPE_SIGN_TRANSACTION_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/sign_transaction".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::SignTransactionResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create smart contract interface
    ///
    /// Create an ABI/IDL in JSON.
    pub async fn create_smart_contract_interface(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateSmartContractInterfaceIntent,
    ) -> Result<
        ActivityResult<immutable_activity::CreateSmartContractInterfaceResult>,
        TurnkeyClientError,
    > {
        let request = external_activity::CreateSmartContractInterfaceRequest {
            r#type: "ACTIVITY_TYPE_CREATE_SMART_CONTRACT_INTERFACE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_smart_contract_interface".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateSmartContractInterfaceResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete smart contract interface
    ///
    /// Delete a smart contract interface.
    pub async fn delete_smart_contract_interface(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteSmartContractInterfaceIntent,
    ) -> Result<
        ActivityResult<immutable_activity::DeleteSmartContractInterfaceResult>,
        TurnkeyClientError,
    > {
        let request = external_activity::DeleteSmartContractInterfaceRequest {
            r#type: "ACTIVITY_TYPE_DELETE_SMART_CONTRACT_INTERFACE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_smart_contract_interface".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteSmartContractInterfaceResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List smart contract interfaces
    ///
    /// List all smart contract interfaces within an organization.
    pub async fn get_smart_contract_interfaces(
        &self,
        request: coordinator::GetSmartContractInterfacesRequest,
    ) -> Result<coordinator::GetSmartContractInterfacesResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/list_smart_contract_interfaces".to_string(),
        )
        .await
    }
    /// Get smart contract interface
    ///
    /// Get details about a smart contract interface.
    pub async fn get_smart_contract_interface(
        &self,
        request: coordinator::GetSmartContractInterfaceRequest,
    ) -> Result<coordinator::GetSmartContractInterfaceResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_smart_contract_interface".to_string(),
        )
        .await
    }
    /// Update root quorum
    ///
    /// Set the threshold and members of the root quorum. This activity must be approved by the current root quorum.
    pub async fn update_root_quorum(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateRootQuorumIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateRootQuorumResult>, TurnkeyClientError>
    {
        let request = external_activity::UpdateRootQuorumRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_root_quorum".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateRootQuorumResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create wallet
    ///
    /// Create a wallet and derive addresses.
    pub async fn create_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateWalletIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateWalletResult>, TurnkeyClientError> {
        let request = external_activity::CreateWalletRequest {
            r#type: "ACTIVITY_TYPE_CREATE_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateWalletResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List wallets
    ///
    /// List all wallets within an organization.
    pub async fn get_wallets(
        &self,
        request: coordinator::GetWalletsRequest,
    ) -> Result<coordinator::GetWalletsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_wallets".to_string())
            .await
    }
    /// Get wallet
    ///
    /// Get details about a wallet.
    pub async fn get_wallet(
        &self,
        request: coordinator::GetWalletRequest,
    ) -> Result<coordinator::GetWalletResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_wallet".to_string())
            .await
    }
    /// Create wallet accounts
    ///
    /// Derive additional addresses using an existing wallet.
    pub async fn create_wallet_accounts(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateWalletAccountsIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateWalletAccountsResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateWalletAccountsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_wallet_accounts".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateWalletAccountsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List wallets accounts
    ///
    /// List all accounts within a wallet.
    pub async fn get_wallet_accounts(
        &self,
        request: coordinator::GetWalletAccountsRequest,
    ) -> Result<coordinator::GetWalletAccountsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/list_wallet_accounts".to_string(),
        )
        .await
    }
    /// Get wallet account
    ///
    /// Get a single wallet account.
    pub async fn get_wallet_account(
        &self,
        request: coordinator::GetWalletAccountRequest,
    ) -> Result<coordinator::GetWalletAccountResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_wallet_account".to_string())
            .await
    }
    /// Create sub-organization
    ///
    /// Create a new sub-organization.
    pub async fn create_sub_organization(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateSubOrganizationIntentV7,
    ) -> Result<ActivityResult<immutable_activity::CreateSubOrganizationResultV7>, TurnkeyClientError>
    {
        let request = external_activity::CreateSubOrganizationRequest {
            r#type: "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_sub_organization".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateSubOrganizationResultV7(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Init email recovery
    ///
    /// Initialize a new email recovery.
    pub async fn init_user_email_recovery(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitUserEmailRecoveryIntentV2,
    ) -> Result<ActivityResult<immutable_activity::InitUserEmailRecoveryResult>, TurnkeyClientError>
    {
        let request = external_activity::InitUserEmailRecoveryRequest {
            r#type: "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/init_user_email_recovery".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::InitUserEmailRecoveryResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Recover a user
    ///
    /// Complete the process of recovering a user by adding an authenticator.
    pub async fn recover_user(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::RecoverUserIntent,
    ) -> Result<ActivityResult<immutable_activity::RecoverUserResult>, TurnkeyClientError> {
        let request = external_activity::RecoverUserRequest {
            r#type: "ACTIVITY_TYPE_RECOVER_USER".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/recover_user".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::RecoverUserResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Set organization feature
    ///
    /// Set an organization feature. This activity must be approved by the current root quorum.
    pub async fn set_organization_feature(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SetOrganizationFeatureIntent,
    ) -> Result<ActivityResult<immutable_activity::SetOrganizationFeatureResult>, TurnkeyClientError>
    {
        let request = external_activity::SetOrganizationFeatureRequest {
            r#type: "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/set_organization_feature".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::SetOrganizationFeatureResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Remove organization feature
    ///
    /// Remove an organization feature. This activity must be approved by the current root quorum.
    pub async fn remove_organization_feature(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::RemoveOrganizationFeatureIntent,
    ) -> Result<
        ActivityResult<immutable_activity::RemoveOrganizationFeatureResult>,
        TurnkeyClientError,
    > {
        let request = external_activity::RemoveOrganizationFeatureRequest {
            r#type: "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/remove_organization_feature".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::RemoveOrganizationFeatureResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Export private key
    ///
    /// Export a private key.
    pub async fn export_private_key(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ExportPrivateKeyIntent,
    ) -> Result<ActivityResult<immutable_activity::ExportPrivateKeyResult>, TurnkeyClientError>
    {
        let request = external_activity::ExportPrivateKeyRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/export_private_key".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::ExportPrivateKeyResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Export wallet
    ///
    /// Export a wallet.
    pub async fn export_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ExportWalletIntent,
    ) -> Result<ActivityResult<immutable_activity::ExportWalletResult>, TurnkeyClientError> {
        let request = external_activity::ExportWalletRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/export_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::ExportWalletResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Perform email auth
    ///
    /// Authenticate a user via email.
    pub async fn email_auth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::EmailAuthIntentV3,
    ) -> Result<ActivityResult<immutable_activity::EmailAuthResult>, TurnkeyClientError> {
        let request = external_activity::EmailAuthRequest {
            r#type: "ACTIVITY_TYPE_EMAIL_AUTH_V3".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/email_auth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::EmailAuthResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Export wallet account
    ///
    /// Export a wallet account.
    pub async fn export_wallet_account(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ExportWalletAccountIntent,
    ) -> Result<ActivityResult<immutable_activity::ExportWalletAccountResult>, TurnkeyClientError>
    {
        let request = external_activity::ExportWalletAccountRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/export_wallet_account".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::ExportWalletAccountResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Init fiat on ramp
    ///
    /// Initiate a fiat on ramp flow.
    pub async fn init_fiat_on_ramp(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitFiatOnRampIntent,
    ) -> Result<ActivityResult<immutable_activity::InitFiatOnRampResult>, TurnkeyClientError> {
        let request = external_activity::InitFiatOnRampRequest {
            r#type: "ACTIVITY_TYPE_INIT_FIAT_ON_RAMP".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/init_fiat_on_ramp".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::InitFiatOnRampResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Init import wallet
    ///
    /// Initialize a new wallet import.
    pub async fn init_import_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitImportWalletIntent,
    ) -> Result<ActivityResult<immutable_activity::InitImportWalletResult>, TurnkeyClientError>
    {
        let request = external_activity::InitImportWalletRequest {
            r#type: "ACTIVITY_TYPE_INIT_IMPORT_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/init_import_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::InitImportWalletResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Import wallet
    ///
    /// Import a wallet.
    pub async fn import_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ImportWalletIntent,
    ) -> Result<ActivityResult<immutable_activity::ImportWalletResult>, TurnkeyClientError> {
        let request = external_activity::ImportWalletRequest {
            r#type: "ACTIVITY_TYPE_IMPORT_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/import_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::ImportWalletResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Init import private key
    ///
    /// Initialize a new private key import.
    pub async fn init_import_private_key(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitImportPrivateKeyIntent,
    ) -> Result<ActivityResult<immutable_activity::InitImportPrivateKeyResult>, TurnkeyClientError>
    {
        let request = external_activity::InitImportPrivateKeyRequest {
            r#type: "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/init_import_private_key".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::InitImportPrivateKeyResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Import private key
    ///
    /// Import a private key.
    pub async fn import_private_key(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ImportPrivateKeyIntent,
    ) -> Result<ActivityResult<immutable_activity::ImportPrivateKeyResult>, TurnkeyClientError>
    {
        let request = external_activity::ImportPrivateKeyRequest {
            r#type: "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/import_private_key".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::ImportPrivateKeyResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Oauth
    ///
    /// Authenticate a user with an OIDC token (Oauth).
    pub async fn oauth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::OauthIntent,
    ) -> Result<ActivityResult<immutable_activity::OauthResult>, TurnkeyClientError> {
        let request = external_activity::OauthRequest {
            r#type: "ACTIVITY_TYPE_OAUTH".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/oauth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::OauthResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Init generic OTP
    ///
    /// Initiate a generic OTP activity.
    pub async fn init_otp(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitOtpIntentV2,
    ) -> Result<ActivityResult<immutable_activity::InitOtpResult>, TurnkeyClientError> {
        let request = external_activity::InitOtpRequest {
            r#type: "ACTIVITY_TYPE_INIT_OTP_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/init_otp".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::InitOtpResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Verify generic OTP
    ///
    /// Verify a generic OTP.
    pub async fn verify_otp(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::VerifyOtpIntent,
    ) -> Result<ActivityResult<immutable_activity::VerifyOtpResult>, TurnkeyClientError> {
        let request = external_activity::VerifyOtpRequest {
            r#type: "ACTIVITY_TYPE_VERIFY_OTP".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/verify_otp".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::VerifyOtpResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Init OTP auth
    ///
    /// Initiate an OTP auth activity.
    pub async fn init_otp_auth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitOtpAuthIntentV3,
    ) -> Result<ActivityResult<immutable_activity::InitOtpAuthResultV2>, TurnkeyClientError> {
        let request = external_activity::InitOtpAuthRequest {
            r#type: "ACTIVITY_TYPE_INIT_OTP_AUTH_V3".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/init_otp_auth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::InitOtpAuthResultV2(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// OTP auth
    ///
    /// Authenticate a user with an OTP code sent via email or SMS.
    pub async fn otp_auth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::OtpAuthIntent,
    ) -> Result<ActivityResult<immutable_activity::OtpAuthResult>, TurnkeyClientError> {
        let request = external_activity::OtpAuthRequest {
            r#type: "ACTIVITY_TYPE_OTP_AUTH".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/otp_auth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::OtpAuthResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create Oauth providers
    ///
    /// Create Oauth providers for a specified user.
    pub async fn create_oauth_providers(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateOauthProvidersIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateOauthProvidersResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateOauthProvidersRequest {
            r#type: "ACTIVITY_TYPE_CREATE_OAUTH_PROVIDERS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_oauth_providers".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateOauthProvidersResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete Oauth providers
    ///
    /// Remove Oauth providers for a specified user.
    pub async fn delete_oauth_providers(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteOauthProvidersIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteOauthProvidersResult>, TurnkeyClientError>
    {
        let request = external_activity::DeleteOauthProvidersRequest {
            r#type: "ACTIVITY_TYPE_DELETE_OAUTH_PROVIDERS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_oauth_providers".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteOauthProvidersResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get configs
    ///
    /// Get quorum settings and features for an organization.
    pub async fn get_organization_configs(
        &self,
        request: coordinator::GetOrganizationConfigsRequest,
    ) -> Result<coordinator::GetOrganizationConfigsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_organization_configs".to_string(),
        )
        .await
    }
    /// Delete private keys
    ///
    /// Delete private keys for an organization.
    pub async fn delete_private_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePrivateKeysIntent,
    ) -> Result<ActivityResult<immutable_activity::DeletePrivateKeysResult>, TurnkeyClientError>
    {
        let request = external_activity::DeletePrivateKeysRequest {
            r#type: "ACTIVITY_TYPE_DELETE_PRIVATE_KEYS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_private_keys".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeletePrivateKeysResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update wallet
    ///
    /// Update a wallet for an organization.
    pub async fn update_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateWalletIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateWalletResult>, TurnkeyClientError> {
        let request = external_activity::UpdateWalletRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateWalletResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete wallets
    ///
    /// Delete wallets for an organization.
    pub async fn delete_wallets(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteWalletsIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteWalletsResult>, TurnkeyClientError> {
        let request = external_activity::DeleteWalletsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_WALLETS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_wallets".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteWalletsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete sub-organization
    ///
    /// Delete a sub-organization.
    pub async fn delete_sub_organization(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteSubOrganizationIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteSubOrganizationResult>, TurnkeyClientError>
    {
        let request = external_activity::DeleteSubOrganizationRequest {
            r#type: "ACTIVITY_TYPE_DELETE_SUB_ORGANIZATION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_sub_organization".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteSubOrganizationResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get policy evaluations
    ///
    /// Get the policy evaluations for an activity.
    pub async fn get_policy_evaluations(
        &self,
        request: coordinator::GetPolicyEvaluationsRequest,
    ) -> Result<coordinator::GetPolicyEvaluationsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_policy_evaluations".to_string(),
        )
        .await
    }
    /// Create an OAuth 2.0 Credential
    ///
    /// Enable authentication for end users with an OAuth 2.0 provider
    pub async fn create_oauth2_credential(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateOauth2CredentialIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateOauth2CredentialResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateOauth2CredentialRequest {
            r#type: "ACTIVITY_TYPE_CREATE_OAUTH2_CREDENTIAL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_oauth2_credential".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateOauth2CredentialResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update an OAuth 2.0 Credential
    ///
    /// Update an OAuth 2.0 provider credential
    pub async fn update_oauth2_credential(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateOauth2CredentialIntent,
    ) -> Result<ActivityResult<immutable_activity::UpdateOauth2CredentialResult>, TurnkeyClientError>
    {
        let request = external_activity::UpdateOauth2CredentialRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_OAUTH2_CREDENTIAL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/update_oauth2_credential".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateOauth2CredentialResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete an OAuth 2.0 Credential
    ///
    /// Disable authentication for end users with an OAuth 2.0 provider
    pub async fn delete_oauth2_credential(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteOauth2CredentialIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteOauth2CredentialResult>, TurnkeyClientError>
    {
        let request = external_activity::DeleteOauth2CredentialRequest {
            r#type: "ACTIVITY_TYPE_DELETE_OAUTH2_CREDENTIAL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_oauth2_credential".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteOauth2CredentialResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get a specific boot proof
    ///
    /// Get the boot proof for a given ephemeral key.
    pub async fn get_boot_proof(
        &self,
        request: coordinator::GetBootProofRequest,
    ) -> Result<coordinator::BootProofResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_boot_proof".to_string())
            .await
    }
    /// Get the latest boot proof for an app
    ///
    /// Get the latest boot proof for a given enclave app name.
    pub async fn get_latest_boot_proof(
        &self,
        request: coordinator::GetLatestBootProofRequest,
    ) -> Result<coordinator::BootProofResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_latest_boot_proof".to_string(),
        )
        .await
    }
    /// List App Proofs for an activity
    ///
    /// List the App Proofs for the given activity.
    pub async fn get_app_proofs(
        &self,
        request: coordinator::GetAppProofsRequest,
    ) -> Result<coordinator::GetAppProofsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_app_proofs".to_string())
            .await
    }
    /// List OAuth 2.0 Credentials
    ///
    /// List all OAuth 2.0 credentials within an organization.
    pub async fn list_oauth2_credentials(
        &self,
        request: coordinator::ListOauth2CredentialsRequest,
    ) -> Result<coordinator::ListOauth2CredentialsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/list_oauth2_credentials".to_string(),
        )
        .await
    }
    /// Get OAuth 2.0 credential
    ///
    /// Get details about an OAuth 2.0 credential.
    pub async fn get_oauth2_credential(
        &self,
        request: coordinator::GetOauth2CredentialRequest,
    ) -> Result<coordinator::GetOauth2CredentialResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/get_oauth2_credential".to_string(),
        )
        .await
    }
    /// OAuth 2.0 authentication
    ///
    /// Authenticate a user with an OAuth 2.0 provider and receive an OIDC token to use with the LoginWithOAuth or CreateSubOrganization activities
    pub async fn oauth2_authenticate(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::Oauth2AuthenticateIntent,
    ) -> Result<ActivityResult<immutable_activity::Oauth2AuthenticateResult>, TurnkeyClientError>
    {
        let request = external_activity::Oauth2AuthenticateRequest {
            r#type: "ACTIVITY_TYPE_OAUTH2_AUTHENTICATE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/oauth2_authenticate".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::Oauth2AuthenticateResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete wallet accounts
    ///
    /// Delete wallet accounts for an organization.
    pub async fn delete_wallet_accounts(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteWalletAccountsIntent,
    ) -> Result<ActivityResult<immutable_activity::DeleteWalletAccountsResult>, TurnkeyClientError>
    {
        let request = external_activity::DeleteWalletAccountsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_WALLET_ACCOUNTS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_wallet_accounts".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteWalletAccountsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Submit a raw transaction for broadcasting.
    ///
    /// Submit a raw transaction (serialized and signed) for broadcasting to the network.
    pub async fn eth_send_raw_transaction(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::EthSendRawTransactionIntent,
    ) -> Result<ActivityResult<immutable_activity::EthSendRawTransactionResult>, TurnkeyClientError>
    {
        let request = external_activity::EthSendRawTransactionRequest {
            r#type: "ACTIVITY_TYPE_ETH_SEND_RAW_TRANSACTION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/eth_send_raw_transaction".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::EthSendRawTransactionResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create a Fiat On Ramp Credential
    ///
    /// Create a fiat on ramp provider credential
    pub async fn create_fiat_on_ramp_credential(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateFiatOnRampCredentialIntent,
    ) -> Result<
        ActivityResult<immutable_activity::CreateFiatOnRampCredentialResult>,
        TurnkeyClientError,
    > {
        let request = external_activity::CreateFiatOnRampCredentialRequest {
            r#type: "ACTIVITY_TYPE_CREATE_FIAT_ON_RAMP_CREDENTIAL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_fiat_on_ramp_credential".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateFiatOnRampCredentialResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Update a Fiat On Ramp Credential
    ///
    /// Update a fiat on ramp provider credential
    pub async fn update_fiat_on_ramp_credential(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateFiatOnRampCredentialIntent,
    ) -> Result<
        ActivityResult<immutable_activity::UpdateFiatOnRampCredentialResult>,
        TurnkeyClientError,
    > {
        let request = external_activity::UpdateFiatOnRampCredentialRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_FIAT_ON_RAMP_CREDENTIAL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/update_fiat_on_ramp_credential".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::UpdateFiatOnRampCredentialResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Delete a Fiat On Ramp Credential
    ///
    /// Delete a fiat on ramp provider credential
    pub async fn delete_fiat_on_ramp_credential(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteFiatOnRampCredentialIntent,
    ) -> Result<
        ActivityResult<immutable_activity::DeleteFiatOnRampCredentialResult>,
        TurnkeyClientError,
    > {
        let request = external_activity::DeleteFiatOnRampCredentialRequest {
            r#type: "ACTIVITY_TYPE_DELETE_FIAT_ON_RAMP_CREDENTIAL".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/delete_fiat_on_ramp_credential".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::DeleteFiatOnRampCredentialResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// List Fiat On Ramp Credentials
    ///
    /// List all fiat on ramp provider credentials within an organization.
    pub async fn list_fiat_on_ramp_credentials(
        &self,
        request: coordinator::ListFiatOnRampCredentialsRequest,
    ) -> Result<coordinator::ListFiatOnRampCredentialsResponse, TurnkeyClientError> {
        self.process_request(
            &request,
            "/public/v1/query/list_fiat_on_ramp_credentials".to_string(),
        )
        .await
    }
    /// Submit a transaction intent for broadcasting.
    ///
    /// Submit a transaction intent describing a transaction you would like to broadcast.
    pub async fn eth_send_transaction(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::EthSendTransactionIntent,
    ) -> Result<ActivityResult<immutable_activity::EthSendTransactionResult>, TurnkeyClientError>
    {
        let request = external_activity::EthSendTransactionRequest {
            r#type: "ACTIVITY_TYPE_ETH_SEND_TRANSACTION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
            generate_app_proofs: self.generate_app_proofs(),
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/eth_send_transaction".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::EthSendTransactionResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Get gas usage and limits.
    ///
    /// Get gas usage and gas limits for either the parent organization or a sub-organization.
    pub async fn get_gas_usage(
        &self,
        request: coordinator::GetGasUsageRequest,
    ) -> Result<coordinator::GetGasUsageResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_gas_usage".to_string())
            .await
    }
    /// Get nonces for an address.
    ///
    /// Get nonce values for an address on a given network. Can fetch the standard on-chain nonce and/or the gas station nonce used for sponsored transactions.
    pub async fn get_nonces(
        &self,
        request: coordinator::GetNoncesRequest,
    ) -> Result<coordinator::GetNoncesResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_nonces".to_string())
            .await
    }
    /// Create TVC app
    ///
    /// Create a new TVC application.
    pub async fn create_tvc_app(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateTvcAppIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateTvcAppResult>, TurnkeyClientError> {
        let request = external_activity::CreateTvcAppRequest {
            r#type: "ACTIVITY_TYPE_CREATE_TVC_APP".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_tvc_app".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateTvcAppResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create TVC deployment
    ///
    /// Create a new TVC deployment.
    pub async fn create_tvc_deployment(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateTvcDeploymentIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateTvcDeploymentResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateTvcDeploymentRequest {
            r#type: "ACTIVITY_TYPE_CREATE_TVC_DEPLOYMENT".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_tvc_deployment".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateTvcDeploymentResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
    /// Create TVC manifest approvals
    ///
    /// Post manifest approvals to Turnkey.
    pub async fn create_tvc_manifest_approvals(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateTvcManifestApprovalsIntent,
    ) -> Result<ActivityResult<immutable_activity::CreateTvcManifestApprovalsResult>, TurnkeyClientError>
    {
        let request = external_activity::CreateTvcManifestApprovalsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_TVC_MANIFEST_APPROVALS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(
                &request,
                "/public/v1/submit/create_tvc_manifest_approvals".to_string(),
            )
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        let result = match inner {
            immutable_activity::result::Inner::CreateTvcManifestApprovalsResult(res) => res,
            other => {
                return Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                    serde_json::to_string(&other)?,
                ));
            }
        };
        Ok(ActivityResult {
            result,
            activity_id: activity.id,
            status: activity.status,
            app_proofs: activity.app_proofs,
        })
    }
}
