//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use crate::generated::external::activity::v1 as external_activity;
use crate::generated::immutable::activity::v1 as immutable_activity;
use crate::generated::services::coordinator::public::v1 as coordinator;
use crate::{TurnkeyClient, TurnkeyClientError};
impl TurnkeyClient {
    /// Who am I?
    ///
    /// Get basic information about your current API or WebAuthN user and their organization. Affords Sub-Organization look ups via Parent Organization for WebAuthN or API key users.
    pub async fn get_whoami(
        &self,
        request: coordinator::GetWhoamiRequest,
    ) -> Result<coordinator::GetWhoamiResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/whoami".to_string())
            .await
    }
    /// Get Suborgs
    ///
    /// Get all suborg IDs associated given a parent org ID and an optional filter.
    pub async fn get_sub_org_ids(
        &self,
        request: coordinator::GetSubOrgIdsRequest,
    ) -> Result<coordinator::GetSubOrgIdsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_suborgs".to_string())
            .await
    }
    /// Get Verified Suborgs
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
    /// Get Activity
    ///
    /// Get details about an Activity
    pub async fn get_activity(
        &self,
        request: coordinator::GetActivityRequest,
    ) -> Result<coordinator::ActivityResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_activity".to_string())
            .await
    }
    /// List Activities
    ///
    /// List all Activities within an Organization
    pub async fn get_activities(
        &self,
        request: coordinator::GetActivitiesRequest,
    ) -> Result<coordinator::GetActivitiesResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_activities".to_string())
            .await
    }
    /// Approve Activity
    ///
    /// Approve an Activity
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
        };
        self.process_activity(&request, "/public/v1/submit/approve_activity".to_string())
            .await
    }
    /// Reject Activity
    ///
    /// Reject an Activity
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
        };
        self.process_activity(&request, "/public/v1/submit/reject_activity".to_string())
            .await
    }
    /// Get User
    ///
    /// Get details about a User
    pub async fn get_user(
        &self,
        request: coordinator::GetUserRequest,
    ) -> Result<coordinator::GetUserResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_user".to_string())
            .await
    }
    /// List Users
    ///
    /// List all Users within an Organization
    pub async fn get_users(
        &self,
        request: coordinator::GetUsersRequest,
    ) -> Result<coordinator::GetUsersResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_users".to_string())
            .await
    }
    /// Delete Users
    ///
    /// Delete Users within an Organization
    pub async fn delete_users(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteUsersIntent,
    ) -> Result<immutable_activity::DeleteUsersResult, TurnkeyClientError> {
        let request = external_activity::DeleteUsersRequest {
            r#type: "ACTIVITY_TYPE_DELETE_USERS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_users".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::DeleteUsersResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Policy
    ///
    /// Create a new Policy
    pub async fn create_policy(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePolicyIntentV3,
    ) -> Result<immutable_activity::CreatePolicyResult, TurnkeyClientError> {
        let request = external_activity::CreatePolicyRequest {
            r#type: "ACTIVITY_TYPE_CREATE_POLICY_V3".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_policy".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreatePolicyResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Policies
    ///
    /// Create new Policies
    pub async fn create_policies(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePoliciesIntent,
    ) -> Result<immutable_activity::CreatePoliciesResult, TurnkeyClientError> {
        let request = external_activity::CreatePoliciesRequest {
            r#type: "ACTIVITY_TYPE_CREATE_POLICIES".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_policies".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreatePoliciesResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Update Policy
    ///
    /// Update an existing Policy
    pub async fn update_policy(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdatePolicyIntentV2,
    ) -> Result<immutable_activity::UpdatePolicyResultV2, TurnkeyClientError> {
        let request = external_activity::UpdatePolicyRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_POLICY_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_policy".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::UpdatePolicyResultV2(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete Policy
    ///
    /// Delete an existing Policy
    pub async fn delete_policy(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePolicyIntent,
    ) -> Result<immutable_activity::DeletePolicyResult, TurnkeyClientError> {
        let request = external_activity::DeletePolicyRequest {
            r#type: "ACTIVITY_TYPE_DELETE_POLICY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_policy".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::DeletePolicyResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// List Policies
    ///
    /// List all Policies within an Organization
    pub async fn get_policies(
        &self,
        request: coordinator::GetPoliciesRequest,
    ) -> Result<coordinator::GetPoliciesResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_policies".to_string())
            .await
    }
    /// Get Policy
    ///
    /// Get details about a Policy
    pub async fn get_policy(
        &self,
        request: coordinator::GetPolicyRequest,
    ) -> Result<coordinator::GetPolicyResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_policy".to_string())
            .await
    }
    /// Create Read Only Session
    ///
    /// Create a read only session for a user (valid for 1 hour)
    pub async fn create_read_only_session(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateReadOnlySessionIntent,
    ) -> Result<immutable_activity::CreateReadOnlySessionResult, TurnkeyClientError> {
        let request = external_activity::CreateReadOnlySessionRequest {
            r#type: "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreateReadOnlySessionResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Read Write Session
    ///
    /// Create a read write session for a user
    pub async fn create_read_write_session(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateReadWriteSessionIntentV2,
    ) -> Result<immutable_activity::CreateReadWriteSessionResultV2, TurnkeyClientError> {
        let request = external_activity::CreateReadWriteSessionRequest {
            r#type: "ACTIVITY_TYPE_CREATE_READ_WRITE_SESSION_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreateReadWriteSessionResultV2(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Private Keys
    ///
    /// Create new Private Keys
    pub async fn create_private_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePrivateKeysIntentV2,
    ) -> Result<immutable_activity::CreatePrivateKeysResultV2, TurnkeyClientError> {
        let request = external_activity::CreatePrivateKeysRequest {
            r#type: "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreatePrivateKeysResultV2(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Get Private Key
    ///
    /// Get details about a Private Key
    pub async fn get_private_key(
        &self,
        request: coordinator::GetPrivateKeyRequest,
    ) -> Result<coordinator::GetPrivateKeyResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_private_key".to_string())
            .await
    }
    /// List Private Keys
    ///
    /// List all Private Keys within an Organization
    pub async fn get_private_keys(
        &self,
        request: coordinator::GetPrivateKeysRequest,
    ) -> Result<coordinator::GetPrivateKeysResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_private_keys".to_string())
            .await
    }
    /// Create API Keys
    ///
    /// Add api keys to an existing User
    pub async fn create_api_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateApiKeysIntentV2,
    ) -> Result<immutable_activity::CreateApiKeysResult, TurnkeyClientError> {
        let request = external_activity::CreateApiKeysRequest {
            r#type: "ACTIVITY_TYPE_CREATE_API_KEYS_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_api_keys".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreateApiKeysResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete API Keys
    ///
    /// Remove api keys from a User
    pub async fn delete_api_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteApiKeysIntent,
    ) -> Result<immutable_activity::DeleteApiKeysResult, TurnkeyClientError> {
        let request = external_activity::DeleteApiKeysRequest {
            r#type: "ACTIVITY_TYPE_DELETE_API_KEYS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_api_keys".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::DeleteApiKeysResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Get Oauth providers
    ///
    /// Get details about Oauth providers for a user
    pub async fn get_oauth_providers(
        &self,
        request: coordinator::GetOauthProvidersRequest,
    ) -> Result<coordinator::GetOauthProvidersResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_oauth_providers".to_string())
            .await
    }
    /// Get API key
    ///
    /// Get details about API keys for a user
    pub async fn get_api_keys(
        &self,
        request: coordinator::GetApiKeysRequest,
    ) -> Result<coordinator::GetApiKeysResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_api_keys".to_string())
            .await
    }
    /// Get API key
    ///
    /// Get details about an API key
    pub async fn get_api_key(
        &self,
        request: coordinator::GetApiKeyRequest,
    ) -> Result<coordinator::GetApiKeyResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_api_key".to_string())
            .await
    }
    /// Create Authenticators
    ///
    /// Create Authenticators to authenticate requests to Turnkey
    pub async fn create_authenticators(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateAuthenticatorsIntentV2,
    ) -> Result<immutable_activity::CreateAuthenticatorsResult, TurnkeyClientError> {
        let request = external_activity::CreateAuthenticatorsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreateAuthenticatorsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete Authenticators
    ///
    /// Remove authenticators from a User
    pub async fn delete_authenticators(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteAuthenticatorsIntent,
    ) -> Result<immutable_activity::DeleteAuthenticatorsResult, TurnkeyClientError> {
        let request = external_activity::DeleteAuthenticatorsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_AUTHENTICATORS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::DeleteAuthenticatorsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Get Authenticators
    ///
    /// Get details about authenticators for a user
    pub async fn get_authenticators(
        &self,
        request: coordinator::GetAuthenticatorsRequest,
    ) -> Result<coordinator::GetAuthenticatorsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_authenticators".to_string())
            .await
    }
    /// Get Authenticator
    ///
    /// Get details about an authenticator
    pub async fn get_authenticator(
        &self,
        request: coordinator::GetAuthenticatorRequest,
    ) -> Result<coordinator::GetAuthenticatorResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_authenticator".to_string())
            .await
    }
    /// Create Invitations
    ///
    /// Create Invitations to join an existing Organization
    pub async fn create_invitations(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateInvitationsIntent,
    ) -> Result<immutable_activity::CreateInvitationsResult, TurnkeyClientError> {
        let request = external_activity::CreateInvitationsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_INVITATIONS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_invitations".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreateInvitationsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete Invitation
    ///
    /// Delete an existing Invitation
    pub async fn delete_invitation(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteInvitationIntent,
    ) -> Result<immutable_activity::DeleteInvitationResult, TurnkeyClientError> {
        let request = external_activity::DeleteInvitationRequest {
            r#type: "ACTIVITY_TYPE_DELETE_INVITATION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_invitation".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::DeleteInvitationResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Users
    ///
    /// Create Users in an existing Organization
    pub async fn create_users(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateUsersIntentV3,
    ) -> Result<immutable_activity::CreateUsersResult, TurnkeyClientError> {
        let request = external_activity::CreateUsersRequest {
            r#type: "ACTIVITY_TYPE_CREATE_USERS_V3".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_users".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreateUsersResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Update User
    ///
    /// Update a User in an existing Organization
    pub async fn update_user(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserIntent,
    ) -> Result<immutable_activity::UpdateUserResult, TurnkeyClientError> {
        let request = external_activity::UpdateUserRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_user".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::UpdateUserResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create User Tag
    ///
    /// Create a user tag and add it to users.
    pub async fn create_user_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateUserTagIntent,
    ) -> Result<immutable_activity::CreateUserTagResult, TurnkeyClientError> {
        let request = external_activity::CreateUserTagRequest {
            r#type: "ACTIVITY_TYPE_CREATE_USER_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_user_tag".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreateUserTagResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Private Key Tag
    ///
    /// Create a private key tag and add it to private keys.
    pub async fn create_private_key_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreatePrivateKeyTagIntent,
    ) -> Result<immutable_activity::CreatePrivateKeyTagResult, TurnkeyClientError> {
        let request = external_activity::CreatePrivateKeyTagRequest {
            r#type: "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreatePrivateKeyTagResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Update User Tag
    ///
    /// Update human-readable name or associated users. Note that this activity is atomic: all of the updates will succeed at once, or all of them will fail.
    pub async fn update_user_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateUserTagIntent,
    ) -> Result<immutable_activity::UpdateUserTagResult, TurnkeyClientError> {
        let request = external_activity::UpdateUserTagRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_USER_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_user_tag".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::UpdateUserTagResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// List User Tags
    ///
    /// List all User Tags within an Organization
    pub async fn list_user_tags(
        &self,
        request: coordinator::ListUserTagsRequest,
    ) -> Result<coordinator::ListUserTagsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_user_tags".to_string())
            .await
    }
    /// Delete User Tags
    ///
    /// Delete User Tags within an Organization
    pub async fn delete_user_tags(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteUserTagsIntent,
    ) -> Result<immutable_activity::DeleteUserTagsResult, TurnkeyClientError> {
        let request = external_activity::DeleteUserTagsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_USER_TAGS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_user_tags".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::DeleteUserTagsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Update Private Key Tag
    ///
    /// Update human-readable name or associated private keys. Note that this activity is atomic: all of the updates will succeed at once, or all of them will fail.
    pub async fn update_private_key_tag(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdatePrivateKeyTagIntent,
    ) -> Result<immutable_activity::UpdatePrivateKeyTagResult, TurnkeyClientError> {
        let request = external_activity::UpdatePrivateKeyTagRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::UpdatePrivateKeyTagResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// List Private Key Tags
    ///
    /// List all Private Key Tags within an Organization
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
    /// Delete Private Key Tags
    ///
    /// Delete Private Key Tags within an Organization
    pub async fn delete_private_key_tags(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePrivateKeyTagsIntent,
    ) -> Result<immutable_activity::DeletePrivateKeyTagsResult, TurnkeyClientError> {
        let request = external_activity::DeletePrivateKeyTagsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::DeletePrivateKeyTagsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Sign Raw Payload
    ///
    /// Sign a raw payload
    pub async fn sign_raw_payload(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SignRawPayloadIntentV2,
    ) -> Result<immutable_activity::SignRawPayloadResult, TurnkeyClientError> {
        let request = external_activity::SignRawPayloadRequest {
            r#type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/sign_raw_payload".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::SignRawPayloadResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Sign Raw Payloads
    ///
    /// Sign multiple raw payloads with the same signing parameters
    pub async fn sign_raw_payloads(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SignRawPayloadsIntent,
    ) -> Result<immutable_activity::SignRawPayloadsResult, TurnkeyClientError> {
        let request = external_activity::SignRawPayloadsRequest {
            r#type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/sign_raw_payloads".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::SignRawPayloadsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Sign Transaction
    ///
    /// Sign a transaction
    pub async fn sign_transaction(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SignTransactionIntentV2,
    ) -> Result<immutable_activity::SignTransactionResult, TurnkeyClientError> {
        let request = external_activity::SignTransactionRequest {
            r#type: "ACTIVITY_TYPE_SIGN_TRANSACTION_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/sign_transaction".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::SignTransactionResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Update Root Quorum
    ///
    /// Set the threshold and members of the root quorum. This activity must be approved by the current root quorum.
    pub async fn update_root_quorum(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateRootQuorumIntent,
    ) -> Result<immutable_activity::UpdateRootQuorumResult, TurnkeyClientError> {
        let request = external_activity::UpdateRootQuorumRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_root_quorum".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::UpdateRootQuorumResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Wallet
    ///
    /// Create a Wallet and derive addresses
    pub async fn create_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateWalletIntent,
    ) -> Result<immutable_activity::CreateWalletResult, TurnkeyClientError> {
        let request = external_activity::CreateWalletRequest {
            r#type: "ACTIVITY_TYPE_CREATE_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/create_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::CreateWalletResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// List Wallets
    ///
    /// List all Wallets within an Organization
    pub async fn get_wallets(
        &self,
        request: coordinator::GetWalletsRequest,
    ) -> Result<coordinator::GetWalletsResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/list_wallets".to_string())
            .await
    }
    /// Get Wallet
    ///
    /// Get details about a Wallet
    pub async fn get_wallet(
        &self,
        request: coordinator::GetWalletRequest,
    ) -> Result<coordinator::GetWalletResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_wallet".to_string())
            .await
    }
    /// Create Wallet Accounts
    ///
    /// Derive additional addresses using an existing wallet
    pub async fn create_wallet_accounts(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateWalletAccountsIntent,
    ) -> Result<immutable_activity::CreateWalletAccountsResult, TurnkeyClientError> {
        let request = external_activity::CreateWalletAccountsRequest {
            r#type: "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreateWalletAccountsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// List Wallets Accounts
    ///
    /// List all Accounts within a Wallet
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
    /// Get Wallet Account
    ///
    /// Get a single wallet account
    pub async fn get_wallet_account(
        &self,
        request: coordinator::GetWalletAccountRequest,
    ) -> Result<coordinator::GetWalletAccountResponse, TurnkeyClientError> {
        self.process_request(&request, "/public/v1/query/get_wallet_account".to_string())
            .await
    }
    /// Create Sub-Organization
    ///
    /// Create a new Sub-Organization
    pub async fn create_sub_organization(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateSubOrganizationIntentV7,
    ) -> Result<immutable_activity::CreateSubOrganizationResultV7, TurnkeyClientError> {
        let request = external_activity::CreateSubOrganizationRequest {
            r#type: "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreateSubOrganizationResultV7(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Init Email Recovery
    ///
    /// Initializes a new email recovery
    pub async fn init_user_email_recovery(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitUserEmailRecoveryIntent,
    ) -> Result<immutable_activity::InitUserEmailRecoveryResult, TurnkeyClientError> {
        let request = external_activity::InitUserEmailRecoveryRequest {
            r#type: "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::InitUserEmailRecoveryResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Recover a user
    ///
    /// Completes the process of recovering a user by adding an authenticator
    pub async fn recover_user(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::RecoverUserIntent,
    ) -> Result<immutable_activity::RecoverUserResult, TurnkeyClientError> {
        let request = external_activity::RecoverUserRequest {
            r#type: "ACTIVITY_TYPE_RECOVER_USER".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/recover_user".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::RecoverUserResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Set Organization Feature
    ///
    /// Sets an organization feature. This activity must be approved by the current root quorum.
    pub async fn set_organization_feature(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::SetOrganizationFeatureIntent,
    ) -> Result<immutable_activity::SetOrganizationFeatureResult, TurnkeyClientError> {
        let request = external_activity::SetOrganizationFeatureRequest {
            r#type: "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::SetOrganizationFeatureResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Remove Organization Feature
    ///
    /// Removes an organization feature. This activity must be approved by the current root quorum.
    pub async fn remove_organization_feature(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::RemoveOrganizationFeatureIntent,
    ) -> Result<immutable_activity::RemoveOrganizationFeatureResult, TurnkeyClientError> {
        let request = external_activity::RemoveOrganizationFeatureRequest {
            r#type: "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::RemoveOrganizationFeatureResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Export Private Key
    ///
    /// Exports a Private Key
    pub async fn export_private_key(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ExportPrivateKeyIntent,
    ) -> Result<immutable_activity::ExportPrivateKeyResult, TurnkeyClientError> {
        let request = external_activity::ExportPrivateKeyRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/export_private_key".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::ExportPrivateKeyResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Export Wallet
    ///
    /// Exports a Wallet
    pub async fn export_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ExportWalletIntent,
    ) -> Result<immutable_activity::ExportWalletResult, TurnkeyClientError> {
        let request = external_activity::ExportWalletRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/export_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::ExportWalletResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Perform Email Auth
    ///
    /// Authenticate a user via Email
    pub async fn email_auth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::EmailAuthIntentV2,
    ) -> Result<immutable_activity::EmailAuthResult, TurnkeyClientError> {
        let request = external_activity::EmailAuthRequest {
            r#type: "ACTIVITY_TYPE_EMAIL_AUTH_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/email_auth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::EmailAuthResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Export Wallet Account
    ///
    /// Exports a Wallet Account
    pub async fn export_wallet_account(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ExportWalletAccountIntent,
    ) -> Result<immutable_activity::ExportWalletAccountResult, TurnkeyClientError> {
        let request = external_activity::ExportWalletAccountRequest {
            r#type: "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::ExportWalletAccountResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Init Import Wallet
    ///
    /// Initializes a new wallet import
    pub async fn init_import_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitImportWalletIntent,
    ) -> Result<immutable_activity::InitImportWalletResult, TurnkeyClientError> {
        let request = external_activity::InitImportWalletRequest {
            r#type: "ACTIVITY_TYPE_INIT_IMPORT_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/init_import_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::InitImportWalletResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Import Wallet
    ///
    /// Imports a wallet
    pub async fn import_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ImportWalletIntent,
    ) -> Result<immutable_activity::ImportWalletResult, TurnkeyClientError> {
        let request = external_activity::ImportWalletRequest {
            r#type: "ACTIVITY_TYPE_IMPORT_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/import_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::ImportWalletResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Init Import Private Key
    ///
    /// Initializes a new private key import
    pub async fn init_import_private_key(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitImportPrivateKeyIntent,
    ) -> Result<immutable_activity::InitImportPrivateKeyResult, TurnkeyClientError> {
        let request = external_activity::InitImportPrivateKeyRequest {
            r#type: "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::InitImportPrivateKeyResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Import Private Key
    ///
    /// Imports a private key
    pub async fn import_private_key(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::ImportPrivateKeyIntent,
    ) -> Result<immutable_activity::ImportPrivateKeyResult, TurnkeyClientError> {
        let request = external_activity::ImportPrivateKeyRequest {
            r#type: "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/import_private_key".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::ImportPrivateKeyResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Oauth
    ///
    /// Authenticate a user with an Oidc token (Oauth) - BETA
    pub async fn oauth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::OauthIntent,
    ) -> Result<immutable_activity::OauthResult, TurnkeyClientError> {
        let request = external_activity::OauthRequest {
            r#type: "ACTIVITY_TYPE_OAUTH".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/oauth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::OauthResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Init OTP auth
    ///
    /// Initiate an OTP auth activity
    pub async fn init_otp_auth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::InitOtpAuthIntentV2,
    ) -> Result<immutable_activity::InitOtpAuthResultV2, TurnkeyClientError> {
        let request = external_activity::InitOtpAuthRequest {
            r#type: "ACTIVITY_TYPE_INIT_OTP_AUTH_V2".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/init_otp_auth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::InitOtpAuthResultV2(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// OTP auth
    ///
    /// Authenticate a user with an OTP code sent via email or SMS
    pub async fn otp_auth(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::OtpAuthIntent,
    ) -> Result<immutable_activity::OtpAuthResult, TurnkeyClientError> {
        let request = external_activity::OtpAuthRequest {
            r#type: "ACTIVITY_TYPE_OTP_AUTH".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/otp_auth".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::OtpAuthResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Create Oauth Providers
    ///
    /// Creates Oauth providers for a specified user - BETA
    pub async fn create_oauth_providers(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::CreateOauthProvidersIntent,
    ) -> Result<immutable_activity::CreateOauthProvidersResult, TurnkeyClientError> {
        let request = external_activity::CreateOauthProvidersRequest {
            r#type: "ACTIVITY_TYPE_CREATE_OAUTH_PROVIDERS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::CreateOauthProvidersResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete Oauth Providers
    ///
    /// Removes Oauth providers for a specified user - BETA
    pub async fn delete_oauth_providers(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteOauthProvidersIntent,
    ) -> Result<immutable_activity::DeleteOauthProvidersResult, TurnkeyClientError> {
        let request = external_activity::DeleteOauthProvidersRequest {
            r#type: "ACTIVITY_TYPE_DELETE_OAUTH_PROVIDERS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::DeleteOauthProvidersResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Get Configs
    ///
    /// Get quorum settings and features for an organization
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
    /// Delete Private Keys
    ///
    /// Deletes private keys for an organization
    pub async fn delete_private_keys(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeletePrivateKeysIntent,
    ) -> Result<immutable_activity::DeletePrivateKeysResult, TurnkeyClientError> {
        let request = external_activity::DeletePrivateKeysRequest {
            r#type: "ACTIVITY_TYPE_DELETE_PRIVATE_KEYS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::DeletePrivateKeysResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Update Wallet
    ///
    /// Update a wallet for an organization
    pub async fn update_wallet(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::UpdateWalletIntent,
    ) -> Result<immutable_activity::UpdateWalletResult, TurnkeyClientError> {
        let request = external_activity::UpdateWalletRequest {
            r#type: "ACTIVITY_TYPE_UPDATE_WALLET".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/update_wallet".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::UpdateWalletResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete Wallets
    ///
    /// Deletes wallets for an organization
    pub async fn delete_wallets(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteWalletsIntent,
    ) -> Result<immutable_activity::DeleteWalletsResult, TurnkeyClientError> {
        let request = external_activity::DeleteWalletsRequest {
            r#type: "ACTIVITY_TYPE_DELETE_WALLETS".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
        };
        let activity: external_activity::Activity = self
            .process_activity(&request, "/public/v1/submit/delete_wallets".to_string())
            .await?;
        let inner = activity
            .result
            .ok_or_else(|| TurnkeyClientError::MissingResult)?
            .inner
            .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;
        match inner {
            immutable_activity::result::Inner::DeleteWalletsResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
    /// Delete Sub Organization
    ///
    /// Deletes a sub organization
    pub async fn delete_sub_organization(
        &self,
        organization_id: String,
        timestamp_ms: u128,
        params: immutable_activity::DeleteSubOrganizationIntent,
    ) -> Result<immutable_activity::DeleteSubOrganizationResult, TurnkeyClientError> {
        let request = external_activity::DeleteSubOrganizationRequest {
            r#type: "ACTIVITY_TYPE_DELETE_SUB_ORGANIZATION".to_string(),
            timestamp_ms: timestamp_ms.to_string(),
            parameters: Some(params),
            organization_id,
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
        match inner {
            immutable_activity::result::Inner::DeleteSubOrganizationResult(res) => Ok(res),
            other => Err(TurnkeyClientError::UnexpectedInnerActivityResult(
                serde_json::to_string(&other)?,
            )),
        }
    }
}
