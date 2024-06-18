impl serde::Serialize for AcceptInvitationIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.AcceptInvitationIntent", len)?;
        if true {
            struct_ser.serialize_field("invitationId", &self.invitation_id)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if let Some(v) = self.authenticator.as_ref() {
            struct_ser.serialize_field("authenticator", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AcceptInvitationIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_id",
            "invitationId",
            "user_id",
            "userId",
            "authenticator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationId,
            UserId,
            Authenticator,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitationId" | "invitation_id" => Ok(GeneratedField::InvitationId),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "authenticator" => Ok(GeneratedField::Authenticator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AcceptInvitationIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.AcceptInvitationIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AcceptInvitationIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_id__ = None;
                let mut user_id__ = None;
                let mut authenticator__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationId => {
                            if invitation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationId"));
                            }
                            invitation_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Authenticator => {
                            if authenticator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticator"));
                            }
                            authenticator__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AcceptInvitationIntent {
                    invitation_id: invitation_id__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                    authenticator: authenticator__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.AcceptInvitationIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AcceptInvitationIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.AcceptInvitationIntentV2", len)?;
        if true {
            struct_ser.serialize_field("invitationId", &self.invitation_id)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if let Some(v) = self.authenticator.as_ref() {
            struct_ser.serialize_field("authenticator", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AcceptInvitationIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_id",
            "invitationId",
            "user_id",
            "userId",
            "authenticator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationId,
            UserId,
            Authenticator,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitationId" | "invitation_id" => Ok(GeneratedField::InvitationId),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "authenticator" => Ok(GeneratedField::Authenticator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AcceptInvitationIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.AcceptInvitationIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AcceptInvitationIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_id__ = None;
                let mut user_id__ = None;
                let mut authenticator__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationId => {
                            if invitation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationId"));
                            }
                            invitation_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Authenticator => {
                            if authenticator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticator"));
                            }
                            authenticator__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AcceptInvitationIntentV2 {
                    invitation_id: invitation_id__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                    authenticator: authenticator__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.AcceptInvitationIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AcceptInvitationResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.AcceptInvitationResult", len)?;
        if true {
            struct_ser.serialize_field("invitationId", &self.invitation_id)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AcceptInvitationResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_id",
            "invitationId",
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationId,
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitationId" | "invitation_id" => Ok(GeneratedField::InvitationId),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AcceptInvitationResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.AcceptInvitationResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AcceptInvitationResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_id__ = None;
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationId => {
                            if invitation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationId"));
                            }
                            invitation_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AcceptInvitationResult {
                    invitation_id: invitation_id__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.AcceptInvitationResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActivateBillingTierIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ActivateBillingTierIntent", len)?;
        if true {
            struct_ser.serialize_field("productId", &self.product_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActivateBillingTierIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "product_id",
            "productId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProductId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "productId" | "product_id" => Ok(GeneratedField::ProductId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActivateBillingTierIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ActivateBillingTierIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActivateBillingTierIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut product_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ProductId => {
                            if product_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("productId"));
                            }
                            product_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ActivateBillingTierIntent {
                    product_id: product_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ActivateBillingTierIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActivateBillingTierResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ActivateBillingTierResult", len)?;
        if true {
            struct_ser.serialize_field("productId", &self.product_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActivateBillingTierResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "product_id",
            "productId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProductId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "productId" | "product_id" => Ok(GeneratedField::ProductId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActivateBillingTierResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ActivateBillingTierResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActivateBillingTierResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut product_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ProductId => {
                            if product_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("productId"));
                            }
                            product_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ActivateBillingTierResult {
                    product_id: product_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ActivateBillingTierResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActivityStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ACTIVITY_STATUS_UNSPECIFIED",
            Self::Created => "ACTIVITY_STATUS_CREATED",
            Self::Pending => "ACTIVITY_STATUS_PENDING",
            Self::Completed => "ACTIVITY_STATUS_COMPLETED",
            Self::Failed => "ACTIVITY_STATUS_FAILED",
            Self::ConsensusNeeded => "ACTIVITY_STATUS_CONSENSUS_NEEDED",
            Self::Rejected => "ACTIVITY_STATUS_REJECTED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ActivityStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ACTIVITY_STATUS_UNSPECIFIED",
            "ACTIVITY_STATUS_CREATED",
            "ACTIVITY_STATUS_PENDING",
            "ACTIVITY_STATUS_COMPLETED",
            "ACTIVITY_STATUS_FAILED",
            "ACTIVITY_STATUS_CONSENSUS_NEEDED",
            "ACTIVITY_STATUS_REJECTED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActivityStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ACTIVITY_STATUS_UNSPECIFIED" => Ok(ActivityStatus::Unspecified),
                    "ACTIVITY_STATUS_CREATED" => Ok(ActivityStatus::Created),
                    "ACTIVITY_STATUS_PENDING" => Ok(ActivityStatus::Pending),
                    "ACTIVITY_STATUS_COMPLETED" => Ok(ActivityStatus::Completed),
                    "ACTIVITY_STATUS_FAILED" => Ok(ActivityStatus::Failed),
                    "ACTIVITY_STATUS_CONSENSUS_NEEDED" => Ok(ActivityStatus::ConsensusNeeded),
                    "ACTIVITY_STATUS_REJECTED" => Ok(ActivityStatus::Rejected),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ActivityType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ACTIVITY_TYPE_UNSPECIFIED",
            Self::CreateApiKeys => "ACTIVITY_TYPE_CREATE_API_KEYS",
            Self::CreateUsers => "ACTIVITY_TYPE_CREATE_USERS",
            Self::CreatePrivateKeys => "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS",
            Self::SignRawPayload => "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD",
            Self::CreateInvitations => "ACTIVITY_TYPE_CREATE_INVITATIONS",
            Self::AcceptInvitation => "ACTIVITY_TYPE_ACCEPT_INVITATION",
            Self::CreatePolicy => "ACTIVITY_TYPE_CREATE_POLICY",
            Self::DisablePrivateKey => "ACTIVITY_TYPE_DISABLE_PRIVATE_KEY",
            Self::DeleteUsers => "ACTIVITY_TYPE_DELETE_USERS",
            Self::DeleteApiKeys => "ACTIVITY_TYPE_DELETE_API_KEYS",
            Self::DeleteInvitation => "ACTIVITY_TYPE_DELETE_INVITATION",
            Self::DeleteOrganization => "ACTIVITY_TYPE_DELETE_ORGANIZATION",
            Self::DeletePolicy => "ACTIVITY_TYPE_DELETE_POLICY",
            Self::CreateUserTag => "ACTIVITY_TYPE_CREATE_USER_TAG",
            Self::DeleteUserTags => "ACTIVITY_TYPE_DELETE_USER_TAGS",
            Self::CreateOrganization => "ACTIVITY_TYPE_CREATE_ORGANIZATION",
            Self::SignTransaction => "ACTIVITY_TYPE_SIGN_TRANSACTION",
            Self::ApproveActivity => "ACTIVITY_TYPE_APPROVE_ACTIVITY",
            Self::RejectActivity => "ACTIVITY_TYPE_REJECT_ACTIVITY",
            Self::DeleteAuthenticators => "ACTIVITY_TYPE_DELETE_AUTHENTICATORS",
            Self::CreateAuthenticators => "ACTIVITY_TYPE_CREATE_AUTHENTICATORS",
            Self::CreatePrivateKeyTag => "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG",
            Self::DeletePrivateKeyTags => "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS",
            Self::SetPaymentMethod => "ACTIVITY_TYPE_SET_PAYMENT_METHOD",
            Self::ActivateBillingTier => "ACTIVITY_TYPE_ACTIVATE_BILLING_TIER",
            Self::DeletePaymentMethod => "ACTIVITY_TYPE_DELETE_PAYMENT_METHOD",
            Self::CreatePolicyV2 => "ACTIVITY_TYPE_CREATE_POLICY_V2",
            Self::CreatePolicyV3 => "ACTIVITY_TYPE_CREATE_POLICY_V3",
            Self::CreateApiOnlyUsers => "ACTIVITY_TYPE_CREATE_API_ONLY_USERS",
            Self::UpdateRootQuorum => "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM",
            Self::UpdateUserTag => "ACTIVITY_TYPE_UPDATE_USER_TAG",
            Self::UpdatePrivateKeyTag => "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG",
            Self::CreateAuthenticatorsV2 => "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2",
            Self::CreateOrganizationV2 => "ACTIVITY_TYPE_CREATE_ORGANIZATION_V2",
            Self::CreateUsersV2 => "ACTIVITY_TYPE_CREATE_USERS_V2",
            Self::AcceptInvitationV2 => "ACTIVITY_TYPE_ACCEPT_INVITATION_V2",
            Self::CreateSubOrganization => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION",
            Self::CreateSubOrganizationV2 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V2",
            Self::UpdateAllowedOrigins => "ACTIVITY_TYPE_UPDATE_ALLOWED_ORIGINS",
            Self::CreatePrivateKeysV2 => "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2",
            Self::UpdateUser => "ACTIVITY_TYPE_UPDATE_USER",
            Self::UpdatePolicy => "ACTIVITY_TYPE_UPDATE_POLICY",
            Self::SetPaymentMethodV2 => "ACTIVITY_TYPE_SET_PAYMENT_METHOD_V2",
            Self::CreateSubOrganizationV3 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V3",
            Self::CreateWallet => "ACTIVITY_TYPE_CREATE_WALLET",
            Self::CreateWalletAccounts => "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS",
            Self::InitUserEmailRecovery => "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY",
            Self::RecoverUser => "ACTIVITY_TYPE_RECOVER_USER",
            Self::SetOrganizationFeature => "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE",
            Self::RemoveOrganizationFeature => "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE",
            Self::SignRawPayloadV2 => "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
            Self::SignTransactionV2 => "ACTIVITY_TYPE_SIGN_TRANSACTION_V2",
            Self::ExportPrivateKey => "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY",
            Self::ExportWallet => "ACTIVITY_TYPE_EXPORT_WALLET",
            Self::CreateSubOrganizationV4 => "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V4",
            Self::EmailAuth => "ACTIVITY_TYPE_EMAIL_AUTH",
            Self::ExportWalletAccount => "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT",
            Self::InitImportWallet => "ACTIVITY_TYPE_INIT_IMPORT_WALLET",
            Self::ImportWallet => "ACTIVITY_TYPE_IMPORT_WALLET",
            Self::InitImportPrivateKey => "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY",
            Self::ImportPrivateKey => "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY",
            Self::CreatePolicies => "ACTIVITY_TYPE_CREATE_POLICIES",
            Self::SignRawPayloads => "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS",
            Self::CreateReadOnlySession => "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ActivityType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ACTIVITY_TYPE_UNSPECIFIED",
            "ACTIVITY_TYPE_CREATE_API_KEYS",
            "ACTIVITY_TYPE_CREATE_USERS",
            "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS",
            "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD",
            "ACTIVITY_TYPE_CREATE_INVITATIONS",
            "ACTIVITY_TYPE_ACCEPT_INVITATION",
            "ACTIVITY_TYPE_CREATE_POLICY",
            "ACTIVITY_TYPE_DISABLE_PRIVATE_KEY",
            "ACTIVITY_TYPE_DELETE_USERS",
            "ACTIVITY_TYPE_DELETE_API_KEYS",
            "ACTIVITY_TYPE_DELETE_INVITATION",
            "ACTIVITY_TYPE_DELETE_ORGANIZATION",
            "ACTIVITY_TYPE_DELETE_POLICY",
            "ACTIVITY_TYPE_CREATE_USER_TAG",
            "ACTIVITY_TYPE_DELETE_USER_TAGS",
            "ACTIVITY_TYPE_CREATE_ORGANIZATION",
            "ACTIVITY_TYPE_SIGN_TRANSACTION",
            "ACTIVITY_TYPE_APPROVE_ACTIVITY",
            "ACTIVITY_TYPE_REJECT_ACTIVITY",
            "ACTIVITY_TYPE_DELETE_AUTHENTICATORS",
            "ACTIVITY_TYPE_CREATE_AUTHENTICATORS",
            "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG",
            "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS",
            "ACTIVITY_TYPE_SET_PAYMENT_METHOD",
            "ACTIVITY_TYPE_ACTIVATE_BILLING_TIER",
            "ACTIVITY_TYPE_DELETE_PAYMENT_METHOD",
            "ACTIVITY_TYPE_CREATE_POLICY_V2",
            "ACTIVITY_TYPE_CREATE_POLICY_V3",
            "ACTIVITY_TYPE_CREATE_API_ONLY_USERS",
            "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM",
            "ACTIVITY_TYPE_UPDATE_USER_TAG",
            "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG",
            "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2",
            "ACTIVITY_TYPE_CREATE_ORGANIZATION_V2",
            "ACTIVITY_TYPE_CREATE_USERS_V2",
            "ACTIVITY_TYPE_ACCEPT_INVITATION_V2",
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION",
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V2",
            "ACTIVITY_TYPE_UPDATE_ALLOWED_ORIGINS",
            "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2",
            "ACTIVITY_TYPE_UPDATE_USER",
            "ACTIVITY_TYPE_UPDATE_POLICY",
            "ACTIVITY_TYPE_SET_PAYMENT_METHOD_V2",
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V3",
            "ACTIVITY_TYPE_CREATE_WALLET",
            "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS",
            "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY",
            "ACTIVITY_TYPE_RECOVER_USER",
            "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE",
            "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE",
            "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
            "ACTIVITY_TYPE_SIGN_TRANSACTION_V2",
            "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY",
            "ACTIVITY_TYPE_EXPORT_WALLET",
            "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V4",
            "ACTIVITY_TYPE_EMAIL_AUTH",
            "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT",
            "ACTIVITY_TYPE_INIT_IMPORT_WALLET",
            "ACTIVITY_TYPE_IMPORT_WALLET",
            "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY",
            "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY",
            "ACTIVITY_TYPE_CREATE_POLICIES",
            "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS",
            "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActivityType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ACTIVITY_TYPE_UNSPECIFIED" => Ok(ActivityType::Unspecified),
                    "ACTIVITY_TYPE_CREATE_API_KEYS" => Ok(ActivityType::CreateApiKeys),
                    "ACTIVITY_TYPE_CREATE_USERS" => Ok(ActivityType::CreateUsers),
                    "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS" => Ok(ActivityType::CreatePrivateKeys),
                    "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD" => Ok(ActivityType::SignRawPayload),
                    "ACTIVITY_TYPE_CREATE_INVITATIONS" => Ok(ActivityType::CreateInvitations),
                    "ACTIVITY_TYPE_ACCEPT_INVITATION" => Ok(ActivityType::AcceptInvitation),
                    "ACTIVITY_TYPE_CREATE_POLICY" => Ok(ActivityType::CreatePolicy),
                    "ACTIVITY_TYPE_DISABLE_PRIVATE_KEY" => Ok(ActivityType::DisablePrivateKey),
                    "ACTIVITY_TYPE_DELETE_USERS" => Ok(ActivityType::DeleteUsers),
                    "ACTIVITY_TYPE_DELETE_API_KEYS" => Ok(ActivityType::DeleteApiKeys),
                    "ACTIVITY_TYPE_DELETE_INVITATION" => Ok(ActivityType::DeleteInvitation),
                    "ACTIVITY_TYPE_DELETE_ORGANIZATION" => Ok(ActivityType::DeleteOrganization),
                    "ACTIVITY_TYPE_DELETE_POLICY" => Ok(ActivityType::DeletePolicy),
                    "ACTIVITY_TYPE_CREATE_USER_TAG" => Ok(ActivityType::CreateUserTag),
                    "ACTIVITY_TYPE_DELETE_USER_TAGS" => Ok(ActivityType::DeleteUserTags),
                    "ACTIVITY_TYPE_CREATE_ORGANIZATION" => Ok(ActivityType::CreateOrganization),
                    "ACTIVITY_TYPE_SIGN_TRANSACTION" => Ok(ActivityType::SignTransaction),
                    "ACTIVITY_TYPE_APPROVE_ACTIVITY" => Ok(ActivityType::ApproveActivity),
                    "ACTIVITY_TYPE_REJECT_ACTIVITY" => Ok(ActivityType::RejectActivity),
                    "ACTIVITY_TYPE_DELETE_AUTHENTICATORS" => Ok(ActivityType::DeleteAuthenticators),
                    "ACTIVITY_TYPE_CREATE_AUTHENTICATORS" => Ok(ActivityType::CreateAuthenticators),
                    "ACTIVITY_TYPE_CREATE_PRIVATE_KEY_TAG" => Ok(ActivityType::CreatePrivateKeyTag),
                    "ACTIVITY_TYPE_DELETE_PRIVATE_KEY_TAGS" => Ok(ActivityType::DeletePrivateKeyTags),
                    "ACTIVITY_TYPE_SET_PAYMENT_METHOD" => Ok(ActivityType::SetPaymentMethod),
                    "ACTIVITY_TYPE_ACTIVATE_BILLING_TIER" => Ok(ActivityType::ActivateBillingTier),
                    "ACTIVITY_TYPE_DELETE_PAYMENT_METHOD" => Ok(ActivityType::DeletePaymentMethod),
                    "ACTIVITY_TYPE_CREATE_POLICY_V2" => Ok(ActivityType::CreatePolicyV2),
                    "ACTIVITY_TYPE_CREATE_POLICY_V3" => Ok(ActivityType::CreatePolicyV3),
                    "ACTIVITY_TYPE_CREATE_API_ONLY_USERS" => Ok(ActivityType::CreateApiOnlyUsers),
                    "ACTIVITY_TYPE_UPDATE_ROOT_QUORUM" => Ok(ActivityType::UpdateRootQuorum),
                    "ACTIVITY_TYPE_UPDATE_USER_TAG" => Ok(ActivityType::UpdateUserTag),
                    "ACTIVITY_TYPE_UPDATE_PRIVATE_KEY_TAG" => Ok(ActivityType::UpdatePrivateKeyTag),
                    "ACTIVITY_TYPE_CREATE_AUTHENTICATORS_V2" => Ok(ActivityType::CreateAuthenticatorsV2),
                    "ACTIVITY_TYPE_CREATE_ORGANIZATION_V2" => Ok(ActivityType::CreateOrganizationV2),
                    "ACTIVITY_TYPE_CREATE_USERS_V2" => Ok(ActivityType::CreateUsersV2),
                    "ACTIVITY_TYPE_ACCEPT_INVITATION_V2" => Ok(ActivityType::AcceptInvitationV2),
                    "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION" => Ok(ActivityType::CreateSubOrganization),
                    "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V2" => Ok(ActivityType::CreateSubOrganizationV2),
                    "ACTIVITY_TYPE_UPDATE_ALLOWED_ORIGINS" => Ok(ActivityType::UpdateAllowedOrigins),
                    "ACTIVITY_TYPE_CREATE_PRIVATE_KEYS_V2" => Ok(ActivityType::CreatePrivateKeysV2),
                    "ACTIVITY_TYPE_UPDATE_USER" => Ok(ActivityType::UpdateUser),
                    "ACTIVITY_TYPE_UPDATE_POLICY" => Ok(ActivityType::UpdatePolicy),
                    "ACTIVITY_TYPE_SET_PAYMENT_METHOD_V2" => Ok(ActivityType::SetPaymentMethodV2),
                    "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V3" => Ok(ActivityType::CreateSubOrganizationV3),
                    "ACTIVITY_TYPE_CREATE_WALLET" => Ok(ActivityType::CreateWallet),
                    "ACTIVITY_TYPE_CREATE_WALLET_ACCOUNTS" => Ok(ActivityType::CreateWalletAccounts),
                    "ACTIVITY_TYPE_INIT_USER_EMAIL_RECOVERY" => Ok(ActivityType::InitUserEmailRecovery),
                    "ACTIVITY_TYPE_RECOVER_USER" => Ok(ActivityType::RecoverUser),
                    "ACTIVITY_TYPE_SET_ORGANIZATION_FEATURE" => Ok(ActivityType::SetOrganizationFeature),
                    "ACTIVITY_TYPE_REMOVE_ORGANIZATION_FEATURE" => Ok(ActivityType::RemoveOrganizationFeature),
                    "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2" => Ok(ActivityType::SignRawPayloadV2),
                    "ACTIVITY_TYPE_SIGN_TRANSACTION_V2" => Ok(ActivityType::SignTransactionV2),
                    "ACTIVITY_TYPE_EXPORT_PRIVATE_KEY" => Ok(ActivityType::ExportPrivateKey),
                    "ACTIVITY_TYPE_EXPORT_WALLET" => Ok(ActivityType::ExportWallet),
                    "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V4" => Ok(ActivityType::CreateSubOrganizationV4),
                    "ACTIVITY_TYPE_EMAIL_AUTH" => Ok(ActivityType::EmailAuth),
                    "ACTIVITY_TYPE_EXPORT_WALLET_ACCOUNT" => Ok(ActivityType::ExportWalletAccount),
                    "ACTIVITY_TYPE_INIT_IMPORT_WALLET" => Ok(ActivityType::InitImportWallet),
                    "ACTIVITY_TYPE_IMPORT_WALLET" => Ok(ActivityType::ImportWallet),
                    "ACTIVITY_TYPE_INIT_IMPORT_PRIVATE_KEY" => Ok(ActivityType::InitImportPrivateKey),
                    "ACTIVITY_TYPE_IMPORT_PRIVATE_KEY" => Ok(ActivityType::ImportPrivateKey),
                    "ACTIVITY_TYPE_CREATE_POLICIES" => Ok(ActivityType::CreatePolicies),
                    "ACTIVITY_TYPE_SIGN_RAW_PAYLOADS" => Ok(ActivityType::SignRawPayloads),
                    "ACTIVITY_TYPE_CREATE_READ_ONLY_SESSION" => Ok(ActivityType::CreateReadOnlySession),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Address {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.Address", len)?;
        if true {
            let v = super::super::common::v1::AddressFormat::try_from(self.format)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.format)))?;
            struct_ser.serialize_field("format", &v)?;
        }
        if true {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Address {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "format",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Format,
            Address,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "format" => Ok(GeneratedField::Format),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Address;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.Address")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Address, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut format__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = Some(map_.next_value::<super::super::common::v1::AddressFormat>()? as i32);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Address {
                    format: format__.unwrap_or_default(),
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.Address", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ApiKeyParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ApiKeyParams", len)?;
        if true {
            struct_ser.serialize_field("apiKeyName", &self.api_key_name)?;
        }
        if true {
            struct_ser.serialize_field("publicKey", &self.public_key)?;
        }
        if let Some(v) = self.expiration_seconds.as_ref() {
            struct_ser.serialize_field("expirationSeconds", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ApiKeyParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_key_name",
            "apiKeyName",
            "public_key",
            "publicKey",
            "expiration_seconds",
            "expirationSeconds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiKeyName,
            PublicKey,
            ExpirationSeconds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiKeyName" | "api_key_name" => Ok(GeneratedField::ApiKeyName),
                            "publicKey" | "public_key" => Ok(GeneratedField::PublicKey),
                            "expirationSeconds" | "expiration_seconds" => Ok(GeneratedField::ExpirationSeconds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ApiKeyParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ApiKeyParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ApiKeyParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_key_name__ = None;
                let mut public_key__ = None;
                let mut expiration_seconds__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiKeyName => {
                            if api_key_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyName"));
                            }
                            api_key_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PublicKey => {
                            if public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("publicKey"));
                            }
                            public_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExpirationSeconds => {
                            if expiration_seconds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expirationSeconds"));
                            }
                            expiration_seconds__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ApiKeyParams {
                    api_key_name: api_key_name__.unwrap_or_default(),
                    public_key: public_key__.unwrap_or_default(),
                    expiration_seconds: expiration_seconds__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ApiKeyParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ApiOnlyUserParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ApiOnlyUserParams", len)?;
        if true {
            struct_ser.serialize_field("userName", &self.user_name)?;
        }
        if let Some(v) = self.user_email.as_ref() {
            struct_ser.serialize_field("userEmail", v)?;
        }
        if true {
            struct_ser.serialize_field("userTags", &self.user_tags)?;
        }
        if true {
            struct_ser.serialize_field("apiKeys", &self.api_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ApiOnlyUserParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_name",
            "userName",
            "user_email",
            "userEmail",
            "user_tags",
            "userTags",
            "api_keys",
            "apiKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserName,
            UserEmail,
            UserTags,
            ApiKeys,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userName" | "user_name" => Ok(GeneratedField::UserName),
                            "userEmail" | "user_email" => Ok(GeneratedField::UserEmail),
                            "userTags" | "user_tags" => Ok(GeneratedField::UserTags),
                            "apiKeys" | "api_keys" => Ok(GeneratedField::ApiKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ApiOnlyUserParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ApiOnlyUserParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ApiOnlyUserParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_name__ = None;
                let mut user_email__ = None;
                let mut user_tags__ = None;
                let mut api_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserName => {
                            if user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userName"));
                            }
                            user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserEmail => {
                            if user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userEmail"));
                            }
                            user_email__ = map_.next_value()?;
                        }
                        GeneratedField::UserTags => {
                            if user_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTags"));
                            }
                            user_tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApiKeys => {
                            if api_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeys"));
                            }
                            api_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ApiOnlyUserParams {
                    user_name: user_name__.unwrap_or_default(),
                    user_email: user_email__,
                    user_tags: user_tags__.unwrap_or_default(),
                    api_keys: api_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ApiOnlyUserParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ApproveActivityIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ApproveActivityIntent", len)?;
        if true {
            struct_ser.serialize_field("fingerprint", &self.fingerprint)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ApproveActivityIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fingerprint",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fingerprint,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "fingerprint" => Ok(GeneratedField::Fingerprint),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ApproveActivityIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ApproveActivityIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ApproveActivityIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fingerprint__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Fingerprint => {
                            if fingerprint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fingerprint"));
                            }
                            fingerprint__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ApproveActivityIntent {
                    fingerprint: fingerprint__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ApproveActivityIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Attestation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.Attestation", len)?;
        if true {
            struct_ser.serialize_field("credentialId", &self.credential_id)?;
        }
        if true {
            struct_ser.serialize_field("clientDataJson", &self.client_data_json)?;
        }
        if true {
            struct_ser.serialize_field("attestationObject", &self.attestation_object)?;
        }
        if true {
            let v = self.transports.iter().cloned().map(|v| {
                super::super::webauthn::v1::AuthenticatorTransport::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("transports", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Attestation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "credential_id",
            "credentialId",
            "client_data_json",
            "clientDataJson",
            "attestation_object",
            "attestationObject",
            "transports",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CredentialId,
            ClientDataJson,
            AttestationObject,
            Transports,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "credentialId" | "credential_id" => Ok(GeneratedField::CredentialId),
                            "clientDataJson" | "client_data_json" => Ok(GeneratedField::ClientDataJson),
                            "attestationObject" | "attestation_object" => Ok(GeneratedField::AttestationObject),
                            "transports" => Ok(GeneratedField::Transports),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Attestation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.Attestation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Attestation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut credential_id__ = None;
                let mut client_data_json__ = None;
                let mut attestation_object__ = None;
                let mut transports__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CredentialId => {
                            if credential_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentialId"));
                            }
                            credential_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ClientDataJson => {
                            if client_data_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientDataJson"));
                            }
                            client_data_json__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AttestationObject => {
                            if attestation_object__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attestationObject"));
                            }
                            attestation_object__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Transports => {
                            if transports__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transports"));
                            }
                            transports__ = Some(map_.next_value::<Vec<super::super::webauthn::v1::AuthenticatorTransport>>()?.into_iter().map(|x| x as i32).collect());
                        }
                    }
                }
                Ok(Attestation {
                    credential_id: credential_id__.unwrap_or_default(),
                    client_data_json: client_data_json__.unwrap_or_default(),
                    attestation_object: attestation_object__.unwrap_or_default(),
                    transports: transports__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.Attestation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthenticatorParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.AuthenticatorParams", len)?;
        if true {
            struct_ser.serialize_field("authenticatorName", &self.authenticator_name)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if let Some(v) = self.attestation.as_ref() {
            struct_ser.serialize_field("attestation", v)?;
        }
        if true {
            struct_ser.serialize_field("challenge", &self.challenge)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthenticatorParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticator_name",
            "authenticatorName",
            "user_id",
            "userId",
            "attestation",
            "challenge",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuthenticatorName,
            UserId,
            Attestation,
            Challenge,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticatorName" | "authenticator_name" => Ok(GeneratedField::AuthenticatorName),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "attestation" => Ok(GeneratedField::Attestation),
                            "challenge" => Ok(GeneratedField::Challenge),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthenticatorParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.AuthenticatorParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthenticatorParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticator_name__ = None;
                let mut user_id__ = None;
                let mut attestation__ = None;
                let mut challenge__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuthenticatorName => {
                            if authenticator_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorName"));
                            }
                            authenticator_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Attestation => {
                            if attestation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attestation"));
                            }
                            attestation__ = map_.next_value()?;
                        }
                        GeneratedField::Challenge => {
                            if challenge__.is_some() {
                                return Err(serde::de::Error::duplicate_field("challenge"));
                            }
                            challenge__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(AuthenticatorParams {
                    authenticator_name: authenticator_name__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                    attestation: attestation__,
                    challenge: challenge__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.AuthenticatorParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthenticatorParamsV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.AuthenticatorParamsV2", len)?;
        if true {
            struct_ser.serialize_field("authenticatorName", &self.authenticator_name)?;
        }
        if true {
            struct_ser.serialize_field("challenge", &self.challenge)?;
        }
        if let Some(v) = self.attestation.as_ref() {
            struct_ser.serialize_field("attestation", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthenticatorParamsV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticator_name",
            "authenticatorName",
            "challenge",
            "attestation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuthenticatorName,
            Challenge,
            Attestation,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticatorName" | "authenticator_name" => Ok(GeneratedField::AuthenticatorName),
                            "challenge" => Ok(GeneratedField::Challenge),
                            "attestation" => Ok(GeneratedField::Attestation),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthenticatorParamsV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.AuthenticatorParamsV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthenticatorParamsV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticator_name__ = None;
                let mut challenge__ = None;
                let mut attestation__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuthenticatorName => {
                            if authenticator_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorName"));
                            }
                            authenticator_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Challenge => {
                            if challenge__.is_some() {
                                return Err(serde::de::Error::duplicate_field("challenge"));
                            }
                            challenge__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Attestation => {
                            if attestation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attestation"));
                            }
                            attestation__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AuthenticatorParamsV2 {
                    authenticator_name: authenticator_name__.unwrap_or_default(),
                    challenge: challenge__.unwrap_or_default(),
                    attestation: attestation__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.AuthenticatorParamsV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateApiKeysIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateApiKeysIntent", len)?;
        if true {
            struct_ser.serialize_field("apiKeys", &self.api_keys)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateApiKeysIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_keys",
            "apiKeys",
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiKeys,
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiKeys" | "api_keys" => Ok(GeneratedField::ApiKeys),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateApiKeysIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateApiKeysIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateApiKeysIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_keys__ = None;
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiKeys => {
                            if api_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeys"));
                            }
                            api_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateApiKeysIntent {
                    api_keys: api_keys__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateApiKeysIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateApiKeysResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateApiKeysResult", len)?;
        if true {
            struct_ser.serialize_field("apiKeyIds", &self.api_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateApiKeysResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_key_ids",
            "apiKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiKeyIds" | "api_key_ids" => Ok(GeneratedField::ApiKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateApiKeysResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateApiKeysResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateApiKeysResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiKeyIds => {
                            if api_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyIds"));
                            }
                            api_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateApiKeysResult {
                    api_key_ids: api_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateApiKeysResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateApiOnlyUsersIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateApiOnlyUsersIntent", len)?;
        if true {
            struct_ser.serialize_field("apiOnlyUsers", &self.api_only_users)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateApiOnlyUsersIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_only_users",
            "apiOnlyUsers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiOnlyUsers,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiOnlyUsers" | "api_only_users" => Ok(GeneratedField::ApiOnlyUsers),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateApiOnlyUsersIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateApiOnlyUsersIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateApiOnlyUsersIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_only_users__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiOnlyUsers => {
                            if api_only_users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiOnlyUsers"));
                            }
                            api_only_users__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateApiOnlyUsersIntent {
                    api_only_users: api_only_users__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateApiOnlyUsersIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateApiOnlyUsersResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateApiOnlyUsersResult", len)?;
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateApiOnlyUsersResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateApiOnlyUsersResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateApiOnlyUsersResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateApiOnlyUsersResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateApiOnlyUsersResult {
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateApiOnlyUsersResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateAuthenticatorsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateAuthenticatorsIntent", len)?;
        if true {
            struct_ser.serialize_field("authenticators", &self.authenticators)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateAuthenticatorsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticators",
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authenticators,
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticators" => Ok(GeneratedField::Authenticators),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateAuthenticatorsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateAuthenticatorsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateAuthenticatorsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticators__ = None;
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Authenticators => {
                            if authenticators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticators"));
                            }
                            authenticators__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateAuthenticatorsIntent {
                    authenticators: authenticators__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateAuthenticatorsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateAuthenticatorsIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateAuthenticatorsIntentV2", len)?;
        if true {
            struct_ser.serialize_field("authenticators", &self.authenticators)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateAuthenticatorsIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticators",
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authenticators,
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticators" => Ok(GeneratedField::Authenticators),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateAuthenticatorsIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateAuthenticatorsIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateAuthenticatorsIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticators__ = None;
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Authenticators => {
                            if authenticators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticators"));
                            }
                            authenticators__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateAuthenticatorsIntentV2 {
                    authenticators: authenticators__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateAuthenticatorsIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateAuthenticatorsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateAuthenticatorsResult", len)?;
        if true {
            struct_ser.serialize_field("authenticatorIds", &self.authenticator_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateAuthenticatorsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticator_ids",
            "authenticatorIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuthenticatorIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticatorIds" | "authenticator_ids" => Ok(GeneratedField::AuthenticatorIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateAuthenticatorsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateAuthenticatorsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateAuthenticatorsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticator_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuthenticatorIds => {
                            if authenticator_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorIds"));
                            }
                            authenticator_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateAuthenticatorsResult {
                    authenticator_ids: authenticator_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateAuthenticatorsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateInvitationsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateInvitationsIntent", len)?;
        if true {
            struct_ser.serialize_field("invitations", &self.invitations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateInvitationsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Invitations,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitations" => Ok(GeneratedField::Invitations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateInvitationsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateInvitationsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateInvitationsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitations__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Invitations => {
                            if invitations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitations"));
                            }
                            invitations__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateInvitationsIntent {
                    invitations: invitations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateInvitationsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateInvitationsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateInvitationsResult", len)?;
        if true {
            struct_ser.serialize_field("invitationIds", &self.invitation_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateInvitationsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_ids",
            "invitationIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitationIds" | "invitation_ids" => Ok(GeneratedField::InvitationIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateInvitationsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateInvitationsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateInvitationsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationIds => {
                            if invitation_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationIds"));
                            }
                            invitation_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateInvitationsResult {
                    invitation_ids: invitation_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateInvitationsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateOrganizationIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateOrganizationIntent", len)?;
        if true {
            struct_ser.serialize_field("organizationName", &self.organization_name)?;
        }
        if true {
            struct_ser.serialize_field("rootEmail", &self.root_email)?;
        }
        if let Some(v) = self.root_authenticator.as_ref() {
            struct_ser.serialize_field("rootAuthenticator", v)?;
        }
        if let Some(v) = self.root_user_id.as_ref() {
            struct_ser.serialize_field("rootUserId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateOrganizationIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_name",
            "organizationName",
            "root_email",
            "rootEmail",
            "root_authenticator",
            "rootAuthenticator",
            "root_user_id",
            "rootUserId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationName,
            RootEmail,
            RootAuthenticator,
            RootUserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "organizationName" | "organization_name" => Ok(GeneratedField::OrganizationName),
                            "rootEmail" | "root_email" => Ok(GeneratedField::RootEmail),
                            "rootAuthenticator" | "root_authenticator" => Ok(GeneratedField::RootAuthenticator),
                            "rootUserId" | "root_user_id" => Ok(GeneratedField::RootUserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateOrganizationIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateOrganizationIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateOrganizationIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_name__ = None;
                let mut root_email__ = None;
                let mut root_authenticator__ = None;
                let mut root_user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationName => {
                            if organization_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationName"));
                            }
                            organization_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootEmail => {
                            if root_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootEmail"));
                            }
                            root_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootAuthenticator => {
                            if root_authenticator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootAuthenticator"));
                            }
                            root_authenticator__ = map_.next_value()?;
                        }
                        GeneratedField::RootUserId => {
                            if root_user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUserId"));
                            }
                            root_user_id__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CreateOrganizationIntent {
                    organization_name: organization_name__.unwrap_or_default(),
                    root_email: root_email__.unwrap_or_default(),
                    root_authenticator: root_authenticator__,
                    root_user_id: root_user_id__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateOrganizationIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateOrganizationIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateOrganizationIntentV2", len)?;
        if true {
            struct_ser.serialize_field("organizationName", &self.organization_name)?;
        }
        if true {
            struct_ser.serialize_field("rootEmail", &self.root_email)?;
        }
        if let Some(v) = self.root_authenticator.as_ref() {
            struct_ser.serialize_field("rootAuthenticator", v)?;
        }
        if let Some(v) = self.root_user_id.as_ref() {
            struct_ser.serialize_field("rootUserId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateOrganizationIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_name",
            "organizationName",
            "root_email",
            "rootEmail",
            "root_authenticator",
            "rootAuthenticator",
            "root_user_id",
            "rootUserId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationName,
            RootEmail,
            RootAuthenticator,
            RootUserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "organizationName" | "organization_name" => Ok(GeneratedField::OrganizationName),
                            "rootEmail" | "root_email" => Ok(GeneratedField::RootEmail),
                            "rootAuthenticator" | "root_authenticator" => Ok(GeneratedField::RootAuthenticator),
                            "rootUserId" | "root_user_id" => Ok(GeneratedField::RootUserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateOrganizationIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateOrganizationIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateOrganizationIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_name__ = None;
                let mut root_email__ = None;
                let mut root_authenticator__ = None;
                let mut root_user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationName => {
                            if organization_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationName"));
                            }
                            organization_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootEmail => {
                            if root_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootEmail"));
                            }
                            root_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootAuthenticator => {
                            if root_authenticator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootAuthenticator"));
                            }
                            root_authenticator__ = map_.next_value()?;
                        }
                        GeneratedField::RootUserId => {
                            if root_user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUserId"));
                            }
                            root_user_id__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CreateOrganizationIntentV2 {
                    organization_name: organization_name__.unwrap_or_default(),
                    root_email: root_email__.unwrap_or_default(),
                    root_authenticator: root_authenticator__,
                    root_user_id: root_user_id__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateOrganizationIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateOrganizationResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateOrganizationResult", len)?;
        if true {
            struct_ser.serialize_field("organizationId", &self.organization_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateOrganizationResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_id",
            "organizationId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "organizationId" | "organization_id" => Ok(GeneratedField::OrganizationId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateOrganizationResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateOrganizationResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateOrganizationResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationId => {
                            if organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationId"));
                            }
                            organization_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateOrganizationResult {
                    organization_id: organization_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateOrganizationResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePoliciesIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePoliciesIntent", len)?;
        if true {
            struct_ser.serialize_field("policies", &self.policies)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePoliciesIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policies",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Policies,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policies" => Ok(GeneratedField::Policies),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePoliciesIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePoliciesIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePoliciesIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policies__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Policies => {
                            if policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policies"));
                            }
                            policies__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePoliciesIntent {
                    policies: policies__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePoliciesIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePoliciesResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePoliciesResult", len)?;
        if true {
            struct_ser.serialize_field("policyIds", &self.policy_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePoliciesResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_ids",
            "policyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyIds" | "policy_ids" => Ok(GeneratedField::PolicyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePoliciesResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePoliciesResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePoliciesResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyIds => {
                            if policy_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyIds"));
                            }
                            policy_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePoliciesResult {
                    policy_ids: policy_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePoliciesResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePolicyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePolicyIntent", len)?;
        if true {
            struct_ser.serialize_field("policyName", &self.policy_name)?;
        }
        if true {
            struct_ser.serialize_field("selectors", &self.selectors)?;
        }
        if true {
            let v = super::super::common::v1::Effect::try_from(self.effect)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.effect)))?;
            struct_ser.serialize_field("effect", &v)?;
        }
        if true {
            struct_ser.serialize_field("notes", &self.notes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePolicyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_name",
            "policyName",
            "selectors",
            "effect",
            "notes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyName,
            Selectors,
            Effect,
            Notes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyName" | "policy_name" => Ok(GeneratedField::PolicyName),
                            "selectors" => Ok(GeneratedField::Selectors),
                            "effect" => Ok(GeneratedField::Effect),
                            "notes" => Ok(GeneratedField::Notes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePolicyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePolicyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePolicyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_name__ = None;
                let mut selectors__ = None;
                let mut effect__ = None;
                let mut notes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyName => {
                            if policy_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyName"));
                            }
                            policy_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Selectors => {
                            if selectors__.is_some() {
                                return Err(serde::de::Error::duplicate_field("selectors"));
                            }
                            selectors__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Effect => {
                            if effect__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effect"));
                            }
                            effect__ = Some(map_.next_value::<super::super::common::v1::Effect>()? as i32);
                        }
                        GeneratedField::Notes => {
                            if notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notes"));
                            }
                            notes__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePolicyIntent {
                    policy_name: policy_name__.unwrap_or_default(),
                    selectors: selectors__.unwrap_or_default(),
                    effect: effect__.unwrap_or_default(),
                    notes: notes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePolicyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePolicyIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePolicyIntentV2", len)?;
        if true {
            struct_ser.serialize_field("policyName", &self.policy_name)?;
        }
        if true {
            struct_ser.serialize_field("selectors", &self.selectors)?;
        }
        if true {
            let v = super::super::common::v1::Effect::try_from(self.effect)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.effect)))?;
            struct_ser.serialize_field("effect", &v)?;
        }
        if true {
            struct_ser.serialize_field("notes", &self.notes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePolicyIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_name",
            "policyName",
            "selectors",
            "effect",
            "notes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyName,
            Selectors,
            Effect,
            Notes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyName" | "policy_name" => Ok(GeneratedField::PolicyName),
                            "selectors" => Ok(GeneratedField::Selectors),
                            "effect" => Ok(GeneratedField::Effect),
                            "notes" => Ok(GeneratedField::Notes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePolicyIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePolicyIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePolicyIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_name__ = None;
                let mut selectors__ = None;
                let mut effect__ = None;
                let mut notes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyName => {
                            if policy_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyName"));
                            }
                            policy_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Selectors => {
                            if selectors__.is_some() {
                                return Err(serde::de::Error::duplicate_field("selectors"));
                            }
                            selectors__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Effect => {
                            if effect__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effect"));
                            }
                            effect__ = Some(map_.next_value::<super::super::common::v1::Effect>()? as i32);
                        }
                        GeneratedField::Notes => {
                            if notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notes"));
                            }
                            notes__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePolicyIntentV2 {
                    policy_name: policy_name__.unwrap_or_default(),
                    selectors: selectors__.unwrap_or_default(),
                    effect: effect__.unwrap_or_default(),
                    notes: notes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePolicyIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePolicyIntentV3 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePolicyIntentV3", len)?;
        if true {
            struct_ser.serialize_field("policyName", &self.policy_name)?;
        }
        if true {
            let v = super::super::common::v1::Effect::try_from(self.effect)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.effect)))?;
            struct_ser.serialize_field("effect", &v)?;
        }
        if let Some(v) = self.condition.as_ref() {
            struct_ser.serialize_field("condition", v)?;
        }
        if let Some(v) = self.consensus.as_ref() {
            struct_ser.serialize_field("consensus", v)?;
        }
        if true {
            struct_ser.serialize_field("notes", &self.notes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePolicyIntentV3 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_name",
            "policyName",
            "effect",
            "condition",
            "consensus",
            "notes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyName,
            Effect,
            Condition,
            Consensus,
            Notes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyName" | "policy_name" => Ok(GeneratedField::PolicyName),
                            "effect" => Ok(GeneratedField::Effect),
                            "condition" => Ok(GeneratedField::Condition),
                            "consensus" => Ok(GeneratedField::Consensus),
                            "notes" => Ok(GeneratedField::Notes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePolicyIntentV3;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePolicyIntentV3")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePolicyIntentV3, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_name__ = None;
                let mut effect__ = None;
                let mut condition__ = None;
                let mut consensus__ = None;
                let mut notes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyName => {
                            if policy_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyName"));
                            }
                            policy_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Effect => {
                            if effect__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effect"));
                            }
                            effect__ = Some(map_.next_value::<super::super::common::v1::Effect>()? as i32);
                        }
                        GeneratedField::Condition => {
                            if condition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("condition"));
                            }
                            condition__ = map_.next_value()?;
                        }
                        GeneratedField::Consensus => {
                            if consensus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensus"));
                            }
                            consensus__ = map_.next_value()?;
                        }
                        GeneratedField::Notes => {
                            if notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notes"));
                            }
                            notes__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePolicyIntentV3 {
                    policy_name: policy_name__.unwrap_or_default(),
                    effect: effect__.unwrap_or_default(),
                    condition: condition__,
                    consensus: consensus__,
                    notes: notes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePolicyIntentV3", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePolicyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePolicyResult", len)?;
        if true {
            struct_ser.serialize_field("policyId", &self.policy_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePolicyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_id",
            "policyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyId" | "policy_id" => Ok(GeneratedField::PolicyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePolicyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePolicyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePolicyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyId => {
                            if policy_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyId"));
                            }
                            policy_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePolicyResult {
                    policy_id: policy_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePolicyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePrivateKeyTagIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePrivateKeyTagIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyTagName", &self.private_key_tag_name)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyIds", &self.private_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePrivateKeyTagIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_tag_name",
            "privateKeyTagName",
            "private_key_ids",
            "privateKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyTagName,
            PrivateKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyTagName" | "private_key_tag_name" => Ok(GeneratedField::PrivateKeyTagName),
                            "privateKeyIds" | "private_key_ids" => Ok(GeneratedField::PrivateKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePrivateKeyTagIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePrivateKeyTagIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePrivateKeyTagIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_tag_name__ = None;
                let mut private_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyTagName => {
                            if private_key_tag_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTagName"));
                            }
                            private_key_tag_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeyIds => {
                            if private_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyIds"));
                            }
                            private_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePrivateKeyTagIntent {
                    private_key_tag_name: private_key_tag_name__.unwrap_or_default(),
                    private_key_ids: private_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePrivateKeyTagIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePrivateKeyTagResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePrivateKeyTagResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyTagId", &self.private_key_tag_id)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyIds", &self.private_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePrivateKeyTagResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_tag_id",
            "privateKeyTagId",
            "private_key_ids",
            "privateKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyTagId,
            PrivateKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyTagId" | "private_key_tag_id" => Ok(GeneratedField::PrivateKeyTagId),
                            "privateKeyIds" | "private_key_ids" => Ok(GeneratedField::PrivateKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePrivateKeyTagResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePrivateKeyTagResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePrivateKeyTagResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_tag_id__ = None;
                let mut private_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyTagId => {
                            if private_key_tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTagId"));
                            }
                            private_key_tag_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeyIds => {
                            if private_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyIds"));
                            }
                            private_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePrivateKeyTagResult {
                    private_key_tag_id: private_key_tag_id__.unwrap_or_default(),
                    private_key_ids: private_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePrivateKeyTagResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePrivateKeysIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePrivateKeysIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeys", &self.private_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePrivateKeysIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_keys",
            "privateKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeys,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeys" | "private_keys" => Ok(GeneratedField::PrivateKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePrivateKeysIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePrivateKeysIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePrivateKeysIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeys => {
                            if private_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeys"));
                            }
                            private_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePrivateKeysIntent {
                    private_keys: private_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePrivateKeysIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePrivateKeysIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePrivateKeysIntentV2", len)?;
        if true {
            struct_ser.serialize_field("privateKeys", &self.private_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePrivateKeysIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_keys",
            "privateKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeys,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeys" | "private_keys" => Ok(GeneratedField::PrivateKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePrivateKeysIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePrivateKeysIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePrivateKeysIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeys => {
                            if private_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeys"));
                            }
                            private_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePrivateKeysIntentV2 {
                    private_keys: private_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePrivateKeysIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePrivateKeysResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePrivateKeysResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyIds", &self.private_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePrivateKeysResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_ids",
            "privateKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyIds" | "private_key_ids" => Ok(GeneratedField::PrivateKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePrivateKeysResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePrivateKeysResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePrivateKeysResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyIds => {
                            if private_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyIds"));
                            }
                            private_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePrivateKeysResult {
                    private_key_ids: private_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePrivateKeysResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreatePrivateKeysResultV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreatePrivateKeysResultV2", len)?;
        if true {
            struct_ser.serialize_field("privateKeys", &self.private_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePrivateKeysResultV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_keys",
            "privateKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeys,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeys" | "private_keys" => Ok(GeneratedField::PrivateKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePrivateKeysResultV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreatePrivateKeysResultV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePrivateKeysResultV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeys => {
                            if private_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeys"));
                            }
                            private_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreatePrivateKeysResultV2 {
                    private_keys: private_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreatePrivateKeysResultV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateReadOnlySessionIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateReadOnlySessionIntent", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateReadOnlySessionIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateReadOnlySessionIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateReadOnlySessionIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateReadOnlySessionIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(CreateReadOnlySessionIntent {
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateReadOnlySessionIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateReadOnlySessionResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateReadOnlySessionResult", len)?;
        if true {
            struct_ser.serialize_field("organizationId", &self.organization_id)?;
        }
        if true {
            struct_ser.serialize_field("organizationName", &self.organization_name)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("username", &self.username)?;
        }
        if true {
            struct_ser.serialize_field("session", &self.session)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("sessionExpiry", ToString::to_string(&self.session_expiry).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateReadOnlySessionResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_id",
            "organizationId",
            "organization_name",
            "organizationName",
            "user_id",
            "userId",
            "username",
            "session",
            "session_expiry",
            "sessionExpiry",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationId,
            OrganizationName,
            UserId,
            Username,
            Session,
            SessionExpiry,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "organizationId" | "organization_id" => Ok(GeneratedField::OrganizationId),
                            "organizationName" | "organization_name" => Ok(GeneratedField::OrganizationName),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "username" => Ok(GeneratedField::Username),
                            "session" => Ok(GeneratedField::Session),
                            "sessionExpiry" | "session_expiry" => Ok(GeneratedField::SessionExpiry),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateReadOnlySessionResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateReadOnlySessionResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateReadOnlySessionResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_id__ = None;
                let mut organization_name__ = None;
                let mut user_id__ = None;
                let mut username__ = None;
                let mut session__ = None;
                let mut session_expiry__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationId => {
                            if organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationId"));
                            }
                            organization_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OrganizationName => {
                            if organization_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationName"));
                            }
                            organization_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Username => {
                            if username__.is_some() {
                                return Err(serde::de::Error::duplicate_field("username"));
                            }
                            username__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Session => {
                            if session__.is_some() {
                                return Err(serde::de::Error::duplicate_field("session"));
                            }
                            session__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SessionExpiry => {
                            if session_expiry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionExpiry"));
                            }
                            session_expiry__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CreateReadOnlySessionResult {
                    organization_id: organization_id__.unwrap_or_default(),
                    organization_name: organization_name__.unwrap_or_default(),
                    user_id: user_id__.unwrap_or_default(),
                    username: username__.unwrap_or_default(),
                    session: session__.unwrap_or_default(),
                    session_expiry: session_expiry__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateReadOnlySessionResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationIntent", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.root_authenticator.as_ref() {
            struct_ser.serialize_field("rootAuthenticator", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "root_authenticator",
            "rootAuthenticator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            RootAuthenticator,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "rootAuthenticator" | "root_authenticator" => Ok(GeneratedField::RootAuthenticator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut root_authenticator__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootAuthenticator => {
                            if root_authenticator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootAuthenticator"));
                            }
                            root_authenticator__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CreateSubOrganizationIntent {
                    name: name__.unwrap_or_default(),
                    root_authenticator: root_authenticator__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationIntentV2", len)?;
        if true {
            struct_ser.serialize_field("subOrganizationName", &self.sub_organization_name)?;
        }
        if true {
            struct_ser.serialize_field("rootUsers", &self.root_users)?;
        }
        if true {
            struct_ser.serialize_field("rootQuorumThreshold", &self.root_quorum_threshold)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sub_organization_name",
            "subOrganizationName",
            "root_users",
            "rootUsers",
            "root_quorum_threshold",
            "rootQuorumThreshold",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SubOrganizationName,
            RootUsers,
            RootQuorumThreshold,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subOrganizationName" | "sub_organization_name" => Ok(GeneratedField::SubOrganizationName),
                            "rootUsers" | "root_users" => Ok(GeneratedField::RootUsers),
                            "rootQuorumThreshold" | "root_quorum_threshold" => Ok(GeneratedField::RootQuorumThreshold),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sub_organization_name__ = None;
                let mut root_users__ = None;
                let mut root_quorum_threshold__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SubOrganizationName => {
                            if sub_organization_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subOrganizationName"));
                            }
                            sub_organization_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootUsers => {
                            if root_users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUsers"));
                            }
                            root_users__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootQuorumThreshold => {
                            if root_quorum_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootQuorumThreshold"));
                            }
                            root_quorum_threshold__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CreateSubOrganizationIntentV2 {
                    sub_organization_name: sub_organization_name__.unwrap_or_default(),
                    root_users: root_users__.unwrap_or_default(),
                    root_quorum_threshold: root_quorum_threshold__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationIntentV3 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationIntentV3", len)?;
        if true {
            struct_ser.serialize_field("subOrganizationName", &self.sub_organization_name)?;
        }
        if true {
            struct_ser.serialize_field("rootUsers", &self.root_users)?;
        }
        if true {
            struct_ser.serialize_field("rootQuorumThreshold", &self.root_quorum_threshold)?;
        }
        if true {
            struct_ser.serialize_field("privateKeys", &self.private_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationIntentV3 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sub_organization_name",
            "subOrganizationName",
            "root_users",
            "rootUsers",
            "root_quorum_threshold",
            "rootQuorumThreshold",
            "private_keys",
            "privateKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SubOrganizationName,
            RootUsers,
            RootQuorumThreshold,
            PrivateKeys,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subOrganizationName" | "sub_organization_name" => Ok(GeneratedField::SubOrganizationName),
                            "rootUsers" | "root_users" => Ok(GeneratedField::RootUsers),
                            "rootQuorumThreshold" | "root_quorum_threshold" => Ok(GeneratedField::RootQuorumThreshold),
                            "privateKeys" | "private_keys" => Ok(GeneratedField::PrivateKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationIntentV3;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationIntentV3")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationIntentV3, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sub_organization_name__ = None;
                let mut root_users__ = None;
                let mut root_quorum_threshold__ = None;
                let mut private_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SubOrganizationName => {
                            if sub_organization_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subOrganizationName"));
                            }
                            sub_organization_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootUsers => {
                            if root_users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUsers"));
                            }
                            root_users__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootQuorumThreshold => {
                            if root_quorum_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootQuorumThreshold"));
                            }
                            root_quorum_threshold__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PrivateKeys => {
                            if private_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeys"));
                            }
                            private_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateSubOrganizationIntentV3 {
                    sub_organization_name: sub_organization_name__.unwrap_or_default(),
                    root_users: root_users__.unwrap_or_default(),
                    root_quorum_threshold: root_quorum_threshold__.unwrap_or_default(),
                    private_keys: private_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationIntentV3", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationIntentV4 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationIntentV4", len)?;
        if true {
            struct_ser.serialize_field("subOrganizationName", &self.sub_organization_name)?;
        }
        if true {
            struct_ser.serialize_field("rootUsers", &self.root_users)?;
        }
        if true {
            struct_ser.serialize_field("rootQuorumThreshold", &self.root_quorum_threshold)?;
        }
        if let Some(v) = self.wallet.as_ref() {
            struct_ser.serialize_field("wallet", v)?;
        }
        if let Some(v) = self.disable_email_recovery.as_ref() {
            struct_ser.serialize_field("disableEmailRecovery", v)?;
        }
        if let Some(v) = self.disable_email_auth.as_ref() {
            struct_ser.serialize_field("disableEmailAuth", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationIntentV4 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sub_organization_name",
            "subOrganizationName",
            "root_users",
            "rootUsers",
            "root_quorum_threshold",
            "rootQuorumThreshold",
            "wallet",
            "disable_email_recovery",
            "disableEmailRecovery",
            "disable_email_auth",
            "disableEmailAuth",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SubOrganizationName,
            RootUsers,
            RootQuorumThreshold,
            Wallet,
            DisableEmailRecovery,
            DisableEmailAuth,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subOrganizationName" | "sub_organization_name" => Ok(GeneratedField::SubOrganizationName),
                            "rootUsers" | "root_users" => Ok(GeneratedField::RootUsers),
                            "rootQuorumThreshold" | "root_quorum_threshold" => Ok(GeneratedField::RootQuorumThreshold),
                            "wallet" => Ok(GeneratedField::Wallet),
                            "disableEmailRecovery" | "disable_email_recovery" => Ok(GeneratedField::DisableEmailRecovery),
                            "disableEmailAuth" | "disable_email_auth" => Ok(GeneratedField::DisableEmailAuth),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationIntentV4;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationIntentV4")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationIntentV4, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sub_organization_name__ = None;
                let mut root_users__ = None;
                let mut root_quorum_threshold__ = None;
                let mut wallet__ = None;
                let mut disable_email_recovery__ = None;
                let mut disable_email_auth__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SubOrganizationName => {
                            if sub_organization_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subOrganizationName"));
                            }
                            sub_organization_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootUsers => {
                            if root_users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUsers"));
                            }
                            root_users__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootQuorumThreshold => {
                            if root_quorum_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootQuorumThreshold"));
                            }
                            root_quorum_threshold__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Wallet => {
                            if wallet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wallet"));
                            }
                            wallet__ = map_.next_value()?;
                        }
                        GeneratedField::DisableEmailRecovery => {
                            if disable_email_recovery__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disableEmailRecovery"));
                            }
                            disable_email_recovery__ = map_.next_value()?;
                        }
                        GeneratedField::DisableEmailAuth => {
                            if disable_email_auth__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disableEmailAuth"));
                            }
                            disable_email_auth__ = map_.next_value()?;
                        }
                    }
                }
                Ok(CreateSubOrganizationIntentV4 {
                    sub_organization_name: sub_organization_name__.unwrap_or_default(),
                    root_users: root_users__.unwrap_or_default(),
                    root_quorum_threshold: root_quorum_threshold__.unwrap_or_default(),
                    wallet: wallet__,
                    disable_email_recovery: disable_email_recovery__,
                    disable_email_auth: disable_email_auth__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationIntentV4", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationResult", len)?;
        if true {
            struct_ser.serialize_field("subOrganizationId", &self.sub_organization_id)?;
        }
        if true {
            struct_ser.serialize_field("rootUserIds", &self.root_user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sub_organization_id",
            "subOrganizationId",
            "root_user_ids",
            "rootUserIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SubOrganizationId,
            RootUserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subOrganizationId" | "sub_organization_id" => Ok(GeneratedField::SubOrganizationId),
                            "rootUserIds" | "root_user_ids" => Ok(GeneratedField::RootUserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sub_organization_id__ = None;
                let mut root_user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SubOrganizationId => {
                            if sub_organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subOrganizationId"));
                            }
                            sub_organization_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootUserIds => {
                            if root_user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUserIds"));
                            }
                            root_user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateSubOrganizationResult {
                    sub_organization_id: sub_organization_id__.unwrap_or_default(),
                    root_user_ids: root_user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationResultV3 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationResultV3", len)?;
        if true {
            struct_ser.serialize_field("subOrganizationId", &self.sub_organization_id)?;
        }
        if true {
            struct_ser.serialize_field("privateKeys", &self.private_keys)?;
        }
        if true {
            struct_ser.serialize_field("rootUserIds", &self.root_user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationResultV3 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sub_organization_id",
            "subOrganizationId",
            "private_keys",
            "privateKeys",
            "root_user_ids",
            "rootUserIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SubOrganizationId,
            PrivateKeys,
            RootUserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subOrganizationId" | "sub_organization_id" => Ok(GeneratedField::SubOrganizationId),
                            "privateKeys" | "private_keys" => Ok(GeneratedField::PrivateKeys),
                            "rootUserIds" | "root_user_ids" => Ok(GeneratedField::RootUserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationResultV3;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationResultV3")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationResultV3, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sub_organization_id__ = None;
                let mut private_keys__ = None;
                let mut root_user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SubOrganizationId => {
                            if sub_organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subOrganizationId"));
                            }
                            sub_organization_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeys => {
                            if private_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeys"));
                            }
                            private_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootUserIds => {
                            if root_user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUserIds"));
                            }
                            root_user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateSubOrganizationResultV3 {
                    sub_organization_id: sub_organization_id__.unwrap_or_default(),
                    private_keys: private_keys__.unwrap_or_default(),
                    root_user_ids: root_user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationResultV3", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateSubOrganizationResultV4 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateSubOrganizationResultV4", len)?;
        if true {
            struct_ser.serialize_field("subOrganizationId", &self.sub_organization_id)?;
        }
        if let Some(v) = self.wallet.as_ref() {
            struct_ser.serialize_field("wallet", v)?;
        }
        if true {
            struct_ser.serialize_field("rootUserIds", &self.root_user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateSubOrganizationResultV4 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sub_organization_id",
            "subOrganizationId",
            "wallet",
            "root_user_ids",
            "rootUserIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SubOrganizationId,
            Wallet,
            RootUserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subOrganizationId" | "sub_organization_id" => Ok(GeneratedField::SubOrganizationId),
                            "wallet" => Ok(GeneratedField::Wallet),
                            "rootUserIds" | "root_user_ids" => Ok(GeneratedField::RootUserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateSubOrganizationResultV4;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateSubOrganizationResultV4")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateSubOrganizationResultV4, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sub_organization_id__ = None;
                let mut wallet__ = None;
                let mut root_user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SubOrganizationId => {
                            if sub_organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subOrganizationId"));
                            }
                            sub_organization_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Wallet => {
                            if wallet__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wallet"));
                            }
                            wallet__ = map_.next_value()?;
                        }
                        GeneratedField::RootUserIds => {
                            if root_user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootUserIds"));
                            }
                            root_user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateSubOrganizationResultV4 {
                    sub_organization_id: sub_organization_id__.unwrap_or_default(),
                    wallet: wallet__,
                    root_user_ids: root_user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateSubOrganizationResultV4", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUserTagIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateUserTagIntent", len)?;
        if true {
            struct_ser.serialize_field("userTagName", &self.user_tag_name)?;
        }
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUserTagIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tag_name",
            "userTagName",
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTagName,
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userTagName" | "user_tag_name" => Ok(GeneratedField::UserTagName),
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUserTagIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateUserTagIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateUserTagIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tag_name__ = None;
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTagName => {
                            if user_tag_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagName"));
                            }
                            user_tag_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateUserTagIntent {
                    user_tag_name: user_tag_name__.unwrap_or_default(),
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateUserTagIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUserTagResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateUserTagResult", len)?;
        if true {
            struct_ser.serialize_field("userTagId", &self.user_tag_id)?;
        }
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUserTagResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tag_id",
            "userTagId",
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTagId,
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userTagId" | "user_tag_id" => Ok(GeneratedField::UserTagId),
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUserTagResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateUserTagResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateUserTagResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tag_id__ = None;
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTagId => {
                            if user_tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagId"));
                            }
                            user_tag_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateUserTagResult {
                    user_tag_id: user_tag_id__.unwrap_or_default(),
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateUserTagResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUsersIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateUsersIntent", len)?;
        if true {
            struct_ser.serialize_field("users", &self.users)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUsersIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "users",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Users,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "users" => Ok(GeneratedField::Users),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUsersIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateUsersIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateUsersIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut users__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Users => {
                            if users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("users"));
                            }
                            users__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateUsersIntent {
                    users: users__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateUsersIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUsersIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateUsersIntentV2", len)?;
        if true {
            struct_ser.serialize_field("users", &self.users)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUsersIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "users",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Users,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "users" => Ok(GeneratedField::Users),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUsersIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateUsersIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateUsersIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut users__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Users => {
                            if users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("users"));
                            }
                            users__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateUsersIntentV2 {
                    users: users__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateUsersIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateUsersResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateUsersResult", len)?;
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateUsersResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateUsersResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateUsersResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateUsersResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateUsersResult {
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateUsersResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateWalletAccountsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateWalletAccountsIntent", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("accounts", &self.accounts)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateWalletAccountsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "accounts",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            Accounts,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "accounts" => Ok(GeneratedField::Accounts),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateWalletAccountsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateWalletAccountsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateWalletAccountsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut accounts__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Accounts => {
                            if accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accounts"));
                            }
                            accounts__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateWalletAccountsIntent {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    accounts: accounts__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateWalletAccountsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateWalletAccountsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateWalletAccountsResult", len)?;
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateWalletAccountsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Addresses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateWalletAccountsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateWalletAccountsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateWalletAccountsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut addresses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateWalletAccountsResult {
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateWalletAccountsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateWalletIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateWalletIntent", len)?;
        if true {
            struct_ser.serialize_field("walletName", &self.wallet_name)?;
        }
        if true {
            struct_ser.serialize_field("accounts", &self.accounts)?;
        }
        if let Some(v) = self.mnemonic_length.as_ref() {
            struct_ser.serialize_field("mnemonicLength", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateWalletIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_name",
            "walletName",
            "accounts",
            "mnemonic_length",
            "mnemonicLength",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletName,
            Accounts,
            MnemonicLength,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletName" | "wallet_name" => Ok(GeneratedField::WalletName),
                            "accounts" => Ok(GeneratedField::Accounts),
                            "mnemonicLength" | "mnemonic_length" => Ok(GeneratedField::MnemonicLength),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateWalletIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateWalletIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateWalletIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_name__ = None;
                let mut accounts__ = None;
                let mut mnemonic_length__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletName => {
                            if wallet_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletName"));
                            }
                            wallet_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Accounts => {
                            if accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accounts"));
                            }
                            accounts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MnemonicLength => {
                            if mnemonic_length__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mnemonicLength"));
                            }
                            mnemonic_length__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(CreateWalletIntent {
                    wallet_name: wallet_name__.unwrap_or_default(),
                    accounts: accounts__.unwrap_or_default(),
                    mnemonic_length: mnemonic_length__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateWalletIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateWalletResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.CreateWalletResult", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateWalletResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            Addresses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateWalletResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.CreateWalletResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateWalletResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut addresses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CreateWalletResult {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.CreateWalletResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteApiKeysIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteApiKeysIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("apiKeyIds", &self.api_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteApiKeysIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "api_key_ids",
            "apiKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            ApiKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "apiKeyIds" | "api_key_ids" => Ok(GeneratedField::ApiKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteApiKeysIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteApiKeysIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteApiKeysIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut api_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApiKeyIds => {
                            if api_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyIds"));
                            }
                            api_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteApiKeysIntent {
                    user_id: user_id__.unwrap_or_default(),
                    api_key_ids: api_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteApiKeysIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteApiKeysResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteApiKeysResult", len)?;
        if true {
            struct_ser.serialize_field("apiKeyIds", &self.api_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteApiKeysResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_key_ids",
            "apiKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiKeyIds" | "api_key_ids" => Ok(GeneratedField::ApiKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteApiKeysResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteApiKeysResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteApiKeysResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiKeyIds => {
                            if api_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyIds"));
                            }
                            api_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteApiKeysResult {
                    api_key_ids: api_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteApiKeysResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteAuthenticatorsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteAuthenticatorsIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("authenticatorIds", &self.authenticator_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteAuthenticatorsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "authenticator_ids",
            "authenticatorIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            AuthenticatorIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "authenticatorIds" | "authenticator_ids" => Ok(GeneratedField::AuthenticatorIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteAuthenticatorsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteAuthenticatorsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteAuthenticatorsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut authenticator_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthenticatorIds => {
                            if authenticator_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorIds"));
                            }
                            authenticator_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteAuthenticatorsIntent {
                    user_id: user_id__.unwrap_or_default(),
                    authenticator_ids: authenticator_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteAuthenticatorsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteAuthenticatorsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteAuthenticatorsResult", len)?;
        if true {
            struct_ser.serialize_field("authenticatorIds", &self.authenticator_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteAuthenticatorsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticator_ids",
            "authenticatorIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuthenticatorIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticatorIds" | "authenticator_ids" => Ok(GeneratedField::AuthenticatorIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteAuthenticatorsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteAuthenticatorsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteAuthenticatorsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticator_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuthenticatorIds => {
                            if authenticator_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorIds"));
                            }
                            authenticator_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteAuthenticatorsResult {
                    authenticator_ids: authenticator_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteAuthenticatorsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteInvitationIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteInvitationIntent", len)?;
        if true {
            struct_ser.serialize_field("invitationId", &self.invitation_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteInvitationIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_id",
            "invitationId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitationId" | "invitation_id" => Ok(GeneratedField::InvitationId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteInvitationIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteInvitationIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteInvitationIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationId => {
                            if invitation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationId"));
                            }
                            invitation_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteInvitationIntent {
                    invitation_id: invitation_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteInvitationIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteInvitationResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteInvitationResult", len)?;
        if true {
            struct_ser.serialize_field("invitationId", &self.invitation_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteInvitationResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_id",
            "invitationId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "invitationId" | "invitation_id" => Ok(GeneratedField::InvitationId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteInvitationResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteInvitationResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteInvitationResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationId => {
                            if invitation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationId"));
                            }
                            invitation_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteInvitationResult {
                    invitation_id: invitation_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteInvitationResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteOrganizationIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteOrganizationIntent", len)?;
        if true {
            struct_ser.serialize_field("organizationId", &self.organization_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteOrganizationIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_id",
            "organizationId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "organizationId" | "organization_id" => Ok(GeneratedField::OrganizationId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteOrganizationIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteOrganizationIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteOrganizationIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationId => {
                            if organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationId"));
                            }
                            organization_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteOrganizationIntent {
                    organization_id: organization_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteOrganizationIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteOrganizationResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteOrganizationResult", len)?;
        if true {
            struct_ser.serialize_field("organizationId", &self.organization_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteOrganizationResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_id",
            "organizationId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "organizationId" | "organization_id" => Ok(GeneratedField::OrganizationId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteOrganizationResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteOrganizationResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteOrganizationResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationId => {
                            if organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationId"));
                            }
                            organization_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteOrganizationResult {
                    organization_id: organization_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteOrganizationResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePaymentMethodIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeletePaymentMethodIntent", len)?;
        if let Some(v) = self.payment_method_id.as_ref() {
            struct_ser.serialize_field("paymentMethodId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePaymentMethodIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payment_method_id",
            "paymentMethodId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PaymentMethodId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "paymentMethodId" | "payment_method_id" => Ok(GeneratedField::PaymentMethodId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeletePaymentMethodIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeletePaymentMethodIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePaymentMethodIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payment_method_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PaymentMethodId => {
                            if payment_method_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("paymentMethodId"));
                            }
                            payment_method_id__ = map_.next_value()?;
                        }
                    }
                }
                Ok(DeletePaymentMethodIntent {
                    payment_method_id: payment_method_id__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeletePaymentMethodIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePaymentMethodResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeletePaymentMethodResult", len)?;
        if true {
            struct_ser.serialize_field("paymentMethodId", &self.payment_method_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePaymentMethodResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payment_method_id",
            "paymentMethodId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PaymentMethodId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "paymentMethodId" | "payment_method_id" => Ok(GeneratedField::PaymentMethodId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeletePaymentMethodResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeletePaymentMethodResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePaymentMethodResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payment_method_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PaymentMethodId => {
                            if payment_method_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("paymentMethodId"));
                            }
                            payment_method_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeletePaymentMethodResult {
                    payment_method_id: payment_method_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeletePaymentMethodResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePolicyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeletePolicyIntent", len)?;
        if true {
            struct_ser.serialize_field("policyId", &self.policy_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePolicyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_id",
            "policyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyId" | "policy_id" => Ok(GeneratedField::PolicyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeletePolicyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeletePolicyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePolicyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyId => {
                            if policy_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyId"));
                            }
                            policy_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeletePolicyIntent {
                    policy_id: policy_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeletePolicyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePolicyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeletePolicyResult", len)?;
        if true {
            struct_ser.serialize_field("policyId", &self.policy_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePolicyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_id",
            "policyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyId" | "policy_id" => Ok(GeneratedField::PolicyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeletePolicyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeletePolicyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePolicyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyId => {
                            if policy_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyId"));
                            }
                            policy_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeletePolicyResult {
                    policy_id: policy_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeletePolicyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePrivateKeyTagsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeletePrivateKeyTagsIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyTagIds", &self.private_key_tag_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePrivateKeyTagsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_tag_ids",
            "privateKeyTagIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyTagIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyTagIds" | "private_key_tag_ids" => Ok(GeneratedField::PrivateKeyTagIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeletePrivateKeyTagsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeletePrivateKeyTagsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePrivateKeyTagsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_tag_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyTagIds => {
                            if private_key_tag_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTagIds"));
                            }
                            private_key_tag_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeletePrivateKeyTagsIntent {
                    private_key_tag_ids: private_key_tag_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeletePrivateKeyTagsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePrivateKeyTagsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeletePrivateKeyTagsResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyTagIds", &self.private_key_tag_ids)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyIds", &self.private_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePrivateKeyTagsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_tag_ids",
            "privateKeyTagIds",
            "private_key_ids",
            "privateKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyTagIds,
            PrivateKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyTagIds" | "private_key_tag_ids" => Ok(GeneratedField::PrivateKeyTagIds),
                            "privateKeyIds" | "private_key_ids" => Ok(GeneratedField::PrivateKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeletePrivateKeyTagsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeletePrivateKeyTagsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePrivateKeyTagsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_tag_ids__ = None;
                let mut private_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyTagIds => {
                            if private_key_tag_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTagIds"));
                            }
                            private_key_tag_ids__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeyIds => {
                            if private_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyIds"));
                            }
                            private_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeletePrivateKeyTagsResult {
                    private_key_tag_ids: private_key_tag_ids__.unwrap_or_default(),
                    private_key_ids: private_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeletePrivateKeyTagsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteUserTagsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteUserTagsIntent", len)?;
        if true {
            struct_ser.serialize_field("userTagIds", &self.user_tag_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteUserTagsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tag_ids",
            "userTagIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTagIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userTagIds" | "user_tag_ids" => Ok(GeneratedField::UserTagIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteUserTagsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteUserTagsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteUserTagsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tag_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTagIds => {
                            if user_tag_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagIds"));
                            }
                            user_tag_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteUserTagsIntent {
                    user_tag_ids: user_tag_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteUserTagsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteUserTagsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteUserTagsResult", len)?;
        if true {
            struct_ser.serialize_field("userTagIds", &self.user_tag_ids)?;
        }
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteUserTagsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tag_ids",
            "userTagIds",
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTagIds,
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userTagIds" | "user_tag_ids" => Ok(GeneratedField::UserTagIds),
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteUserTagsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteUserTagsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteUserTagsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tag_ids__ = None;
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTagIds => {
                            if user_tag_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagIds"));
                            }
                            user_tag_ids__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteUserTagsResult {
                    user_tag_ids: user_tag_ids__.unwrap_or_default(),
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteUserTagsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteUsersIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteUsersIntent", len)?;
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteUsersIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteUsersIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteUsersIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteUsersIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteUsersIntent {
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteUsersIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteUsersResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DeleteUsersResult", len)?;
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteUsersResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteUsersResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DeleteUsersResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteUsersResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DeleteUsersResult {
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DeleteUsersResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DisablePrivateKeyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DisablePrivateKeyIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DisablePrivateKeyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DisablePrivateKeyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DisablePrivateKeyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DisablePrivateKeyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DisablePrivateKeyIntent {
                    private_key_id: private_key_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DisablePrivateKeyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DisablePrivateKeyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.DisablePrivateKeyResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DisablePrivateKeyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DisablePrivateKeyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.DisablePrivateKeyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DisablePrivateKeyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(DisablePrivateKeyResult {
                    private_key_id: private_key_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.DisablePrivateKeyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EmailAuthIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.EmailAuthIntent", len)?;
        if true {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if true {
            struct_ser.serialize_field("targetPublicKey", &self.target_public_key)?;
        }
        if let Some(v) = self.api_key_name.as_ref() {
            struct_ser.serialize_field("apiKeyName", v)?;
        }
        if let Some(v) = self.expiration_seconds.as_ref() {
            struct_ser.serialize_field("expirationSeconds", v)?;
        }
        if let Some(v) = self.email_customization.as_ref() {
            struct_ser.serialize_field("emailCustomization", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EmailAuthIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "email",
            "target_public_key",
            "targetPublicKey",
            "api_key_name",
            "apiKeyName",
            "expiration_seconds",
            "expirationSeconds",
            "email_customization",
            "emailCustomization",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Email,
            TargetPublicKey,
            ApiKeyName,
            ExpirationSeconds,
            EmailCustomization,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "email" => Ok(GeneratedField::Email),
                            "targetPublicKey" | "target_public_key" => Ok(GeneratedField::TargetPublicKey),
                            "apiKeyName" | "api_key_name" => Ok(GeneratedField::ApiKeyName),
                            "expirationSeconds" | "expiration_seconds" => Ok(GeneratedField::ExpirationSeconds),
                            "emailCustomization" | "email_customization" => Ok(GeneratedField::EmailCustomization),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EmailAuthIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.EmailAuthIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EmailAuthIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut email__ = None;
                let mut target_public_key__ = None;
                let mut api_key_name__ = None;
                let mut expiration_seconds__ = None;
                let mut email_customization__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TargetPublicKey => {
                            if target_public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetPublicKey"));
                            }
                            target_public_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApiKeyName => {
                            if api_key_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyName"));
                            }
                            api_key_name__ = map_.next_value()?;
                        }
                        GeneratedField::ExpirationSeconds => {
                            if expiration_seconds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expirationSeconds"));
                            }
                            expiration_seconds__ = map_.next_value()?;
                        }
                        GeneratedField::EmailCustomization => {
                            if email_customization__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emailCustomization"));
                            }
                            email_customization__ = map_.next_value()?;
                        }
                    }
                }
                Ok(EmailAuthIntent {
                    email: email__.unwrap_or_default(),
                    target_public_key: target_public_key__.unwrap_or_default(),
                    api_key_name: api_key_name__,
                    expiration_seconds: expiration_seconds__,
                    email_customization: email_customization__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.EmailAuthIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EmailAuthResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.EmailAuthResult", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("apiKeyId", &self.api_key_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EmailAuthResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "api_key_id",
            "apiKeyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            ApiKeyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "apiKeyId" | "api_key_id" => Ok(GeneratedField::ApiKeyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EmailAuthResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.EmailAuthResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EmailAuthResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut api_key_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApiKeyId => {
                            if api_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyId"));
                            }
                            api_key_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(EmailAuthResult {
                    user_id: user_id__.unwrap_or_default(),
                    api_key_id: api_key_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.EmailAuthResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EmailCustomizationParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.EmailCustomizationParams", len)?;
        if let Some(v) = self.app_name.as_ref() {
            struct_ser.serialize_field("appName", v)?;
        }
        if let Some(v) = self.logo_url.as_ref() {
            struct_ser.serialize_field("logoUrl", v)?;
        }
        if let Some(v) = self.magic_link_template.as_ref() {
            struct_ser.serialize_field("magicLinkTemplate", v)?;
        }
        if let Some(v) = self.template_variables.as_ref() {
            struct_ser.serialize_field("templateVariables", v)?;
        }
        if let Some(v) = self.template_id.as_ref() {
            struct_ser.serialize_field("templateId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EmailCustomizationParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "app_name",
            "appName",
            "logo_url",
            "logoUrl",
            "magic_link_template",
            "magicLinkTemplate",
            "template_variables",
            "templateVariables",
            "template_id",
            "templateId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AppName,
            LogoUrl,
            MagicLinkTemplate,
            TemplateVariables,
            TemplateId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "appName" | "app_name" => Ok(GeneratedField::AppName),
                            "logoUrl" | "logo_url" => Ok(GeneratedField::LogoUrl),
                            "magicLinkTemplate" | "magic_link_template" => Ok(GeneratedField::MagicLinkTemplate),
                            "templateVariables" | "template_variables" => Ok(GeneratedField::TemplateVariables),
                            "templateId" | "template_id" => Ok(GeneratedField::TemplateId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EmailCustomizationParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.EmailCustomizationParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EmailCustomizationParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut app_name__ = None;
                let mut logo_url__ = None;
                let mut magic_link_template__ = None;
                let mut template_variables__ = None;
                let mut template_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AppName => {
                            if app_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appName"));
                            }
                            app_name__ = map_.next_value()?;
                        }
                        GeneratedField::LogoUrl => {
                            if logo_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("logoUrl"));
                            }
                            logo_url__ = map_.next_value()?;
                        }
                        GeneratedField::MagicLinkTemplate => {
                            if magic_link_template__.is_some() {
                                return Err(serde::de::Error::duplicate_field("magicLinkTemplate"));
                            }
                            magic_link_template__ = map_.next_value()?;
                        }
                        GeneratedField::TemplateVariables => {
                            if template_variables__.is_some() {
                                return Err(serde::de::Error::duplicate_field("templateVariables"));
                            }
                            template_variables__ = map_.next_value()?;
                        }
                        GeneratedField::TemplateId => {
                            if template_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("templateId"));
                            }
                            template_id__ = map_.next_value()?;
                        }
                    }
                }
                Ok(EmailCustomizationParams {
                    app_name: app_name__,
                    logo_url: logo_url__,
                    magic_link_template: magic_link_template__,
                    template_variables: template_variables__,
                    template_id: template_id__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.EmailCustomizationParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportPrivateKeyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ExportPrivateKeyIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("targetPublicKey", &self.target_public_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportPrivateKeyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "target_public_key",
            "targetPublicKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            TargetPublicKey,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "targetPublicKey" | "target_public_key" => Ok(GeneratedField::TargetPublicKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportPrivateKeyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ExportPrivateKeyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportPrivateKeyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut target_public_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TargetPublicKey => {
                            if target_public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetPublicKey"));
                            }
                            target_public_key__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportPrivateKeyIntent {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    target_public_key: target_public_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ExportPrivateKeyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportPrivateKeyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ExportPrivateKeyResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("exportBundle", &self.export_bundle)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportPrivateKeyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "export_bundle",
            "exportBundle",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            ExportBundle,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "exportBundle" | "export_bundle" => Ok(GeneratedField::ExportBundle),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportPrivateKeyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ExportPrivateKeyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportPrivateKeyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut export_bundle__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExportBundle => {
                            if export_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportBundle"));
                            }
                            export_bundle__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportPrivateKeyResult {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    export_bundle: export_bundle__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ExportPrivateKeyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportWalletAccountIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ExportWalletAccountIntent", len)?;
        if true {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if true {
            struct_ser.serialize_field("targetPublicKey", &self.target_public_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportWalletAccountIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "target_public_key",
            "targetPublicKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            TargetPublicKey,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "address" => Ok(GeneratedField::Address),
                            "targetPublicKey" | "target_public_key" => Ok(GeneratedField::TargetPublicKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportWalletAccountIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ExportWalletAccountIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportWalletAccountIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut target_public_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TargetPublicKey => {
                            if target_public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetPublicKey"));
                            }
                            target_public_key__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportWalletAccountIntent {
                    address: address__.unwrap_or_default(),
                    target_public_key: target_public_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ExportWalletAccountIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportWalletAccountResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ExportWalletAccountResult", len)?;
        if true {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if true {
            struct_ser.serialize_field("exportBundle", &self.export_bundle)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportWalletAccountResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "export_bundle",
            "exportBundle",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            ExportBundle,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "address" => Ok(GeneratedField::Address),
                            "exportBundle" | "export_bundle" => Ok(GeneratedField::ExportBundle),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportWalletAccountResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ExportWalletAccountResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportWalletAccountResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut export_bundle__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExportBundle => {
                            if export_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportBundle"));
                            }
                            export_bundle__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportWalletAccountResult {
                    address: address__.unwrap_or_default(),
                    export_bundle: export_bundle__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ExportWalletAccountResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportWalletIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ExportWalletIntent", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("targetPublicKey", &self.target_public_key)?;
        }
        if let Some(v) = self.language.as_ref() {
            let v = super::super::common::v1::MnemonicLanguage::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("language", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportWalletIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "target_public_key",
            "targetPublicKey",
            "language",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            TargetPublicKey,
            Language,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "targetPublicKey" | "target_public_key" => Ok(GeneratedField::TargetPublicKey),
                            "language" => Ok(GeneratedField::Language),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportWalletIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ExportWalletIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportWalletIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut target_public_key__ = None;
                let mut language__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TargetPublicKey => {
                            if target_public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetPublicKey"));
                            }
                            target_public_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = map_.next_value::<::std::option::Option<super::super::common::v1::MnemonicLanguage>>()?.map(|x| x as i32);
                        }
                    }
                }
                Ok(ExportWalletIntent {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    target_public_key: target_public_key__.unwrap_or_default(),
                    language: language__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ExportWalletIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExportWalletResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ExportWalletResult", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("exportBundle", &self.export_bundle)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExportWalletResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "export_bundle",
            "exportBundle",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            ExportBundle,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "exportBundle" | "export_bundle" => Ok(GeneratedField::ExportBundle),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExportWalletResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ExportWalletResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExportWalletResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut export_bundle__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExportBundle => {
                            if export_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportBundle"));
                            }
                            export_bundle__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExportWalletResult {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    export_bundle: export_bundle__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ExportWalletResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ImportPrivateKeyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ImportPrivateKeyIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyName", &self.private_key_name)?;
        }
        if true {
            struct_ser.serialize_field("encryptedBundle", &self.encrypted_bundle)?;
        }
        if true {
            let v = super::super::common::v1::Curve::try_from(self.curve)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.curve)))?;
            struct_ser.serialize_field("curve", &v)?;
        }
        if true {
            let v = self.address_formats.iter().cloned().map(|v| {
                super::super::common::v1::AddressFormat::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("addressFormats", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ImportPrivateKeyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "private_key_name",
            "privateKeyName",
            "encrypted_bundle",
            "encryptedBundle",
            "curve",
            "address_formats",
            "addressFormats",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            PrivateKeyName,
            EncryptedBundle,
            Curve,
            AddressFormats,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "privateKeyName" | "private_key_name" => Ok(GeneratedField::PrivateKeyName),
                            "encryptedBundle" | "encrypted_bundle" => Ok(GeneratedField::EncryptedBundle),
                            "curve" => Ok(GeneratedField::Curve),
                            "addressFormats" | "address_formats" => Ok(GeneratedField::AddressFormats),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ImportPrivateKeyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ImportPrivateKeyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ImportPrivateKeyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut private_key_name__ = None;
                let mut encrypted_bundle__ = None;
                let mut curve__ = None;
                let mut address_formats__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeyName => {
                            if private_key_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyName"));
                            }
                            private_key_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EncryptedBundle => {
                            if encrypted_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedBundle"));
                            }
                            encrypted_bundle__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Curve => {
                            if curve__.is_some() {
                                return Err(serde::de::Error::duplicate_field("curve"));
                            }
                            curve__ = Some(map_.next_value::<super::super::common::v1::Curve>()? as i32);
                        }
                        GeneratedField::AddressFormats => {
                            if address_formats__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressFormats"));
                            }
                            address_formats__ = Some(map_.next_value::<Vec<super::super::common::v1::AddressFormat>>()?.into_iter().map(|x| x as i32).collect());
                        }
                    }
                }
                Ok(ImportPrivateKeyIntent {
                    user_id: user_id__.unwrap_or_default(),
                    private_key_name: private_key_name__.unwrap_or_default(),
                    encrypted_bundle: encrypted_bundle__.unwrap_or_default(),
                    curve: curve__.unwrap_or_default(),
                    address_formats: address_formats__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ImportPrivateKeyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ImportPrivateKeyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ImportPrivateKeyResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ImportPrivateKeyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            Addresses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ImportPrivateKeyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ImportPrivateKeyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ImportPrivateKeyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut addresses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ImportPrivateKeyResult {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ImportPrivateKeyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ImportWalletIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ImportWalletIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("walletName", &self.wallet_name)?;
        }
        if true {
            struct_ser.serialize_field("encryptedBundle", &self.encrypted_bundle)?;
        }
        if true {
            struct_ser.serialize_field("accounts", &self.accounts)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ImportWalletIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "wallet_name",
            "walletName",
            "encrypted_bundle",
            "encryptedBundle",
            "accounts",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            WalletName,
            EncryptedBundle,
            Accounts,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "walletName" | "wallet_name" => Ok(GeneratedField::WalletName),
                            "encryptedBundle" | "encrypted_bundle" => Ok(GeneratedField::EncryptedBundle),
                            "accounts" => Ok(GeneratedField::Accounts),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ImportWalletIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ImportWalletIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ImportWalletIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut wallet_name__ = None;
                let mut encrypted_bundle__ = None;
                let mut accounts__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::WalletName => {
                            if wallet_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletName"));
                            }
                            wallet_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EncryptedBundle => {
                            if encrypted_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedBundle"));
                            }
                            encrypted_bundle__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Accounts => {
                            if accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accounts"));
                            }
                            accounts__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ImportWalletIntent {
                    user_id: user_id__.unwrap_or_default(),
                    wallet_name: wallet_name__.unwrap_or_default(),
                    encrypted_bundle: encrypted_bundle__.unwrap_or_default(),
                    accounts: accounts__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ImportWalletIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ImportWalletResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.ImportWalletResult", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ImportWalletResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            Addresses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ImportWalletResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.ImportWalletResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ImportWalletResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut addresses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ImportWalletResult {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.ImportWalletResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitImportPrivateKeyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InitImportPrivateKeyIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitImportPrivateKeyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitImportPrivateKeyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InitImportPrivateKeyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitImportPrivateKeyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitImportPrivateKeyIntent {
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InitImportPrivateKeyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitImportPrivateKeyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InitImportPrivateKeyResult", len)?;
        if true {
            struct_ser.serialize_field("importBundle", &self.import_bundle)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitImportPrivateKeyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "import_bundle",
            "importBundle",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ImportBundle,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "importBundle" | "import_bundle" => Ok(GeneratedField::ImportBundle),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitImportPrivateKeyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InitImportPrivateKeyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitImportPrivateKeyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut import_bundle__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ImportBundle => {
                            if import_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importBundle"));
                            }
                            import_bundle__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitImportPrivateKeyResult {
                    import_bundle: import_bundle__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InitImportPrivateKeyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitImportWalletIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InitImportWalletIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitImportWalletIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitImportWalletIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InitImportWalletIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitImportWalletIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitImportWalletIntent {
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InitImportWalletIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitImportWalletResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InitImportWalletResult", len)?;
        if true {
            struct_ser.serialize_field("importBundle", &self.import_bundle)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitImportWalletResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "import_bundle",
            "importBundle",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ImportBundle,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "importBundle" | "import_bundle" => Ok(GeneratedField::ImportBundle),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitImportWalletResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InitImportWalletResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitImportWalletResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut import_bundle__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ImportBundle => {
                            if import_bundle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importBundle"));
                            }
                            import_bundle__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitImportWalletResult {
                    import_bundle: import_bundle__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InitImportWalletResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitUserEmailRecoveryIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InitUserEmailRecoveryIntent", len)?;
        if true {
            struct_ser.serialize_field("email", &self.email)?;
        }
        if true {
            struct_ser.serialize_field("targetPublicKey", &self.target_public_key)?;
        }
        if let Some(v) = self.expiration_seconds.as_ref() {
            struct_ser.serialize_field("expirationSeconds", v)?;
        }
        if let Some(v) = self.email_customization.as_ref() {
            struct_ser.serialize_field("emailCustomization", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitUserEmailRecoveryIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "email",
            "target_public_key",
            "targetPublicKey",
            "expiration_seconds",
            "expirationSeconds",
            "email_customization",
            "emailCustomization",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Email,
            TargetPublicKey,
            ExpirationSeconds,
            EmailCustomization,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "email" => Ok(GeneratedField::Email),
                            "targetPublicKey" | "target_public_key" => Ok(GeneratedField::TargetPublicKey),
                            "expirationSeconds" | "expiration_seconds" => Ok(GeneratedField::ExpirationSeconds),
                            "emailCustomization" | "email_customization" => Ok(GeneratedField::EmailCustomization),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitUserEmailRecoveryIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InitUserEmailRecoveryIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitUserEmailRecoveryIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut email__ = None;
                let mut target_public_key__ = None;
                let mut expiration_seconds__ = None;
                let mut email_customization__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TargetPublicKey => {
                            if target_public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targetPublicKey"));
                            }
                            target_public_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExpirationSeconds => {
                            if expiration_seconds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expirationSeconds"));
                            }
                            expiration_seconds__ = map_.next_value()?;
                        }
                        GeneratedField::EmailCustomization => {
                            if email_customization__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emailCustomization"));
                            }
                            email_customization__ = map_.next_value()?;
                        }
                    }
                }
                Ok(InitUserEmailRecoveryIntent {
                    email: email__.unwrap_or_default(),
                    target_public_key: target_public_key__.unwrap_or_default(),
                    expiration_seconds: expiration_seconds__,
                    email_customization: email_customization__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InitUserEmailRecoveryIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitUserEmailRecoveryResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InitUserEmailRecoveryResult", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitUserEmailRecoveryResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitUserEmailRecoveryResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InitUserEmailRecoveryResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitUserEmailRecoveryResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InitUserEmailRecoveryResult {
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InitUserEmailRecoveryResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Intent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.inner.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.Intent", len)?;
        if let Some(v) = self.inner.as_ref() {
            match v {
                intent::Inner::CreateOrganizationIntent(v) => {
                    struct_ser.serialize_field("createOrganizationIntent", v)?;
                }
                intent::Inner::CreateAuthenticatorsIntent(v) => {
                    struct_ser.serialize_field("createAuthenticatorsIntent", v)?;
                }
                intent::Inner::CreateUsersIntent(v) => {
                    struct_ser.serialize_field("createUsersIntent", v)?;
                }
                intent::Inner::CreatePrivateKeysIntent(v) => {
                    struct_ser.serialize_field("createPrivateKeysIntent", v)?;
                }
                intent::Inner::SignRawPayloadIntent(v) => {
                    struct_ser.serialize_field("signRawPayloadIntent", v)?;
                }
                intent::Inner::CreateInvitationsIntent(v) => {
                    struct_ser.serialize_field("createInvitationsIntent", v)?;
                }
                intent::Inner::AcceptInvitationIntent(v) => {
                    struct_ser.serialize_field("acceptInvitationIntent", v)?;
                }
                intent::Inner::CreatePolicyIntent(v) => {
                    struct_ser.serialize_field("createPolicyIntent", v)?;
                }
                intent::Inner::DisablePrivateKeyIntent(v) => {
                    struct_ser.serialize_field("disablePrivateKeyIntent", v)?;
                }
                intent::Inner::DeleteUsersIntent(v) => {
                    struct_ser.serialize_field("deleteUsersIntent", v)?;
                }
                intent::Inner::DeleteAuthenticatorsIntent(v) => {
                    struct_ser.serialize_field("deleteAuthenticatorsIntent", v)?;
                }
                intent::Inner::DeleteInvitationIntent(v) => {
                    struct_ser.serialize_field("deleteInvitationIntent", v)?;
                }
                intent::Inner::DeleteOrganizationIntent(v) => {
                    struct_ser.serialize_field("deleteOrganizationIntent", v)?;
                }
                intent::Inner::DeletePolicyIntent(v) => {
                    struct_ser.serialize_field("deletePolicyIntent", v)?;
                }
                intent::Inner::CreateUserTagIntent(v) => {
                    struct_ser.serialize_field("createUserTagIntent", v)?;
                }
                intent::Inner::DeleteUserTagsIntent(v) => {
                    struct_ser.serialize_field("deleteUserTagsIntent", v)?;
                }
                intent::Inner::SignTransactionIntent(v) => {
                    struct_ser.serialize_field("signTransactionIntent", v)?;
                }
                intent::Inner::CreateApiKeysIntent(v) => {
                    struct_ser.serialize_field("createApiKeysIntent", v)?;
                }
                intent::Inner::DeleteApiKeysIntent(v) => {
                    struct_ser.serialize_field("deleteApiKeysIntent", v)?;
                }
                intent::Inner::ApproveActivityIntent(v) => {
                    struct_ser.serialize_field("approveActivityIntent", v)?;
                }
                intent::Inner::RejectActivityIntent(v) => {
                    struct_ser.serialize_field("rejectActivityIntent", v)?;
                }
                intent::Inner::CreatePrivateKeyTagIntent(v) => {
                    struct_ser.serialize_field("createPrivateKeyTagIntent", v)?;
                }
                intent::Inner::DeletePrivateKeyTagsIntent(v) => {
                    struct_ser.serialize_field("deletePrivateKeyTagsIntent", v)?;
                }
                intent::Inner::CreatePolicyIntentV2(v) => {
                    struct_ser.serialize_field("createPolicyIntentV2", v)?;
                }
                intent::Inner::SetPaymentMethodIntent(v) => {
                    struct_ser.serialize_field("setPaymentMethodIntent", v)?;
                }
                intent::Inner::ActivateBillingTierIntent(v) => {
                    struct_ser.serialize_field("activateBillingTierIntent", v)?;
                }
                intent::Inner::DeletePaymentMethodIntent(v) => {
                    struct_ser.serialize_field("deletePaymentMethodIntent", v)?;
                }
                intent::Inner::CreatePolicyIntentV3(v) => {
                    struct_ser.serialize_field("createPolicyIntentV3", v)?;
                }
                intent::Inner::CreateApiOnlyUsersIntent(v) => {
                    struct_ser.serialize_field("createApiOnlyUsersIntent", v)?;
                }
                intent::Inner::UpdateRootQuorumIntent(v) => {
                    struct_ser.serialize_field("updateRootQuorumIntent", v)?;
                }
                intent::Inner::UpdateUserTagIntent(v) => {
                    struct_ser.serialize_field("updateUserTagIntent", v)?;
                }
                intent::Inner::UpdatePrivateKeyTagIntent(v) => {
                    struct_ser.serialize_field("updatePrivateKeyTagIntent", v)?;
                }
                intent::Inner::CreateAuthenticatorsIntentV2(v) => {
                    struct_ser.serialize_field("createAuthenticatorsIntentV2", v)?;
                }
                intent::Inner::AcceptInvitationIntentV2(v) => {
                    struct_ser.serialize_field("acceptInvitationIntentV2", v)?;
                }
                intent::Inner::CreateOrganizationIntentV2(v) => {
                    struct_ser.serialize_field("createOrganizationIntentV2", v)?;
                }
                intent::Inner::CreateUsersIntentV2(v) => {
                    struct_ser.serialize_field("createUsersIntentV2", v)?;
                }
                intent::Inner::CreateSubOrganizationIntent(v) => {
                    struct_ser.serialize_field("createSubOrganizationIntent", v)?;
                }
                intent::Inner::CreateSubOrganizationIntentV2(v) => {
                    struct_ser.serialize_field("createSubOrganizationIntentV2", v)?;
                }
                intent::Inner::UpdateAllowedOriginsIntent(v) => {
                    struct_ser.serialize_field("updateAllowedOriginsIntent", v)?;
                }
                intent::Inner::CreatePrivateKeysIntentV2(v) => {
                    struct_ser.serialize_field("createPrivateKeysIntentV2", v)?;
                }
                intent::Inner::UpdateUserIntent(v) => {
                    struct_ser.serialize_field("updateUserIntent", v)?;
                }
                intent::Inner::UpdatePolicyIntent(v) => {
                    struct_ser.serialize_field("updatePolicyIntent", v)?;
                }
                intent::Inner::SetPaymentMethodIntentV2(v) => {
                    struct_ser.serialize_field("setPaymentMethodIntentV2", v)?;
                }
                intent::Inner::CreateSubOrganizationIntentV3(v) => {
                    struct_ser.serialize_field("createSubOrganizationIntentV3", v)?;
                }
                intent::Inner::CreateWalletIntent(v) => {
                    struct_ser.serialize_field("createWalletIntent", v)?;
                }
                intent::Inner::CreateWalletAccountsIntent(v) => {
                    struct_ser.serialize_field("createWalletAccountsIntent", v)?;
                }
                intent::Inner::InitUserEmailRecoveryIntent(v) => {
                    struct_ser.serialize_field("initUserEmailRecoveryIntent", v)?;
                }
                intent::Inner::RecoverUserIntent(v) => {
                    struct_ser.serialize_field("recoverUserIntent", v)?;
                }
                intent::Inner::SetOrganizationFeatureIntent(v) => {
                    struct_ser.serialize_field("setOrganizationFeatureIntent", v)?;
                }
                intent::Inner::RemoveOrganizationFeatureIntent(v) => {
                    struct_ser.serialize_field("removeOrganizationFeatureIntent", v)?;
                }
                intent::Inner::SignRawPayloadIntentV2(v) => {
                    struct_ser.serialize_field("signRawPayloadIntentV2", v)?;
                }
                intent::Inner::SignTransactionIntentV2(v) => {
                    struct_ser.serialize_field("signTransactionIntentV2", v)?;
                }
                intent::Inner::ExportPrivateKeyIntent(v) => {
                    struct_ser.serialize_field("exportPrivateKeyIntent", v)?;
                }
                intent::Inner::ExportWalletIntent(v) => {
                    struct_ser.serialize_field("exportWalletIntent", v)?;
                }
                intent::Inner::CreateSubOrganizationIntentV4(v) => {
                    struct_ser.serialize_field("createSubOrganizationIntentV4", v)?;
                }
                intent::Inner::EmailAuthIntent(v) => {
                    struct_ser.serialize_field("emailAuthIntent", v)?;
                }
                intent::Inner::ExportWalletAccountIntent(v) => {
                    struct_ser.serialize_field("exportWalletAccountIntent", v)?;
                }
                intent::Inner::InitImportWalletIntent(v) => {
                    struct_ser.serialize_field("initImportWalletIntent", v)?;
                }
                intent::Inner::ImportWalletIntent(v) => {
                    struct_ser.serialize_field("importWalletIntent", v)?;
                }
                intent::Inner::InitImportPrivateKeyIntent(v) => {
                    struct_ser.serialize_field("initImportPrivateKeyIntent", v)?;
                }
                intent::Inner::ImportPrivateKeyIntent(v) => {
                    struct_ser.serialize_field("importPrivateKeyIntent", v)?;
                }
                intent::Inner::CreatePoliciesIntent(v) => {
                    struct_ser.serialize_field("createPoliciesIntent", v)?;
                }
                intent::Inner::SignRawPayloadsIntent(v) => {
                    struct_ser.serialize_field("signRawPayloadsIntent", v)?;
                }
                intent::Inner::CreateReadOnlySessionIntent(v) => {
                    struct_ser.serialize_field("createReadOnlySessionIntent", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Intent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "create_organization_intent",
            "createOrganizationIntent",
            "create_authenticators_intent",
            "createAuthenticatorsIntent",
            "create_users_intent",
            "createUsersIntent",
            "create_private_keys_intent",
            "createPrivateKeysIntent",
            "sign_raw_payload_intent",
            "signRawPayloadIntent",
            "create_invitations_intent",
            "createInvitationsIntent",
            "accept_invitation_intent",
            "acceptInvitationIntent",
            "create_policy_intent",
            "createPolicyIntent",
            "disable_private_key_intent",
            "disablePrivateKeyIntent",
            "delete_users_intent",
            "deleteUsersIntent",
            "delete_authenticators_intent",
            "deleteAuthenticatorsIntent",
            "delete_invitation_intent",
            "deleteInvitationIntent",
            "delete_organization_intent",
            "deleteOrganizationIntent",
            "delete_policy_intent",
            "deletePolicyIntent",
            "create_user_tag_intent",
            "createUserTagIntent",
            "delete_user_tags_intent",
            "deleteUserTagsIntent",
            "sign_transaction_intent",
            "signTransactionIntent",
            "create_api_keys_intent",
            "createApiKeysIntent",
            "delete_api_keys_intent",
            "deleteApiKeysIntent",
            "approve_activity_intent",
            "approveActivityIntent",
            "reject_activity_intent",
            "rejectActivityIntent",
            "create_private_key_tag_intent",
            "createPrivateKeyTagIntent",
            "delete_private_key_tags_intent",
            "deletePrivateKeyTagsIntent",
            "create_policy_intent_v2",
            "createPolicyIntentV2",
            "set_payment_method_intent",
            "setPaymentMethodIntent",
            "activate_billing_tier_intent",
            "activateBillingTierIntent",
            "delete_payment_method_intent",
            "deletePaymentMethodIntent",
            "create_policy_intent_v3",
            "createPolicyIntentV3",
            "create_api_only_users_intent",
            "createApiOnlyUsersIntent",
            "update_root_quorum_intent",
            "updateRootQuorumIntent",
            "update_user_tag_intent",
            "updateUserTagIntent",
            "update_private_key_tag_intent",
            "updatePrivateKeyTagIntent",
            "create_authenticators_intent_v2",
            "createAuthenticatorsIntentV2",
            "accept_invitation_intent_v2",
            "acceptInvitationIntentV2",
            "create_organization_intent_v2",
            "createOrganizationIntentV2",
            "create_users_intent_v2",
            "createUsersIntentV2",
            "create_sub_organization_intent",
            "createSubOrganizationIntent",
            "create_sub_organization_intent_v2",
            "createSubOrganizationIntentV2",
            "update_allowed_origins_intent",
            "updateAllowedOriginsIntent",
            "create_private_keys_intent_v2",
            "createPrivateKeysIntentV2",
            "update_user_intent",
            "updateUserIntent",
            "update_policy_intent",
            "updatePolicyIntent",
            "set_payment_method_intent_v2",
            "setPaymentMethodIntentV2",
            "create_sub_organization_intent_v3",
            "createSubOrganizationIntentV3",
            "create_wallet_intent",
            "createWalletIntent",
            "create_wallet_accounts_intent",
            "createWalletAccountsIntent",
            "init_user_email_recovery_intent",
            "initUserEmailRecoveryIntent",
            "recover_user_intent",
            "recoverUserIntent",
            "set_organization_feature_intent",
            "setOrganizationFeatureIntent",
            "remove_organization_feature_intent",
            "removeOrganizationFeatureIntent",
            "sign_raw_payload_intent_v2",
            "signRawPayloadIntentV2",
            "sign_transaction_intent_v2",
            "signTransactionIntentV2",
            "export_private_key_intent",
            "exportPrivateKeyIntent",
            "export_wallet_intent",
            "exportWalletIntent",
            "create_sub_organization_intent_v4",
            "createSubOrganizationIntentV4",
            "email_auth_intent",
            "emailAuthIntent",
            "export_wallet_account_intent",
            "exportWalletAccountIntent",
            "init_import_wallet_intent",
            "initImportWalletIntent",
            "import_wallet_intent",
            "importWalletIntent",
            "init_import_private_key_intent",
            "initImportPrivateKeyIntent",
            "import_private_key_intent",
            "importPrivateKeyIntent",
            "create_policies_intent",
            "createPoliciesIntent",
            "sign_raw_payloads_intent",
            "signRawPayloadsIntent",
            "create_read_only_session_intent",
            "createReadOnlySessionIntent",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CreateOrganizationIntent,
            CreateAuthenticatorsIntent,
            CreateUsersIntent,
            CreatePrivateKeysIntent,
            SignRawPayloadIntent,
            CreateInvitationsIntent,
            AcceptInvitationIntent,
            CreatePolicyIntent,
            DisablePrivateKeyIntent,
            DeleteUsersIntent,
            DeleteAuthenticatorsIntent,
            DeleteInvitationIntent,
            DeleteOrganizationIntent,
            DeletePolicyIntent,
            CreateUserTagIntent,
            DeleteUserTagsIntent,
            SignTransactionIntent,
            CreateApiKeysIntent,
            DeleteApiKeysIntent,
            ApproveActivityIntent,
            RejectActivityIntent,
            CreatePrivateKeyTagIntent,
            DeletePrivateKeyTagsIntent,
            CreatePolicyIntentV2,
            SetPaymentMethodIntent,
            ActivateBillingTierIntent,
            DeletePaymentMethodIntent,
            CreatePolicyIntentV3,
            CreateApiOnlyUsersIntent,
            UpdateRootQuorumIntent,
            UpdateUserTagIntent,
            UpdatePrivateKeyTagIntent,
            CreateAuthenticatorsIntentV2,
            AcceptInvitationIntentV2,
            CreateOrganizationIntentV2,
            CreateUsersIntentV2,
            CreateSubOrganizationIntent,
            CreateSubOrganizationIntentV2,
            UpdateAllowedOriginsIntent,
            CreatePrivateKeysIntentV2,
            UpdateUserIntent,
            UpdatePolicyIntent,
            SetPaymentMethodIntentV2,
            CreateSubOrganizationIntentV3,
            CreateWalletIntent,
            CreateWalletAccountsIntent,
            InitUserEmailRecoveryIntent,
            RecoverUserIntent,
            SetOrganizationFeatureIntent,
            RemoveOrganizationFeatureIntent,
            SignRawPayloadIntentV2,
            SignTransactionIntentV2,
            ExportPrivateKeyIntent,
            ExportWalletIntent,
            CreateSubOrganizationIntentV4,
            EmailAuthIntent,
            ExportWalletAccountIntent,
            InitImportWalletIntent,
            ImportWalletIntent,
            InitImportPrivateKeyIntent,
            ImportPrivateKeyIntent,
            CreatePoliciesIntent,
            SignRawPayloadsIntent,
            CreateReadOnlySessionIntent,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "createOrganizationIntent" | "create_organization_intent" => Ok(GeneratedField::CreateOrganizationIntent),
                            "createAuthenticatorsIntent" | "create_authenticators_intent" => Ok(GeneratedField::CreateAuthenticatorsIntent),
                            "createUsersIntent" | "create_users_intent" => Ok(GeneratedField::CreateUsersIntent),
                            "createPrivateKeysIntent" | "create_private_keys_intent" => Ok(GeneratedField::CreatePrivateKeysIntent),
                            "signRawPayloadIntent" | "sign_raw_payload_intent" => Ok(GeneratedField::SignRawPayloadIntent),
                            "createInvitationsIntent" | "create_invitations_intent" => Ok(GeneratedField::CreateInvitationsIntent),
                            "acceptInvitationIntent" | "accept_invitation_intent" => Ok(GeneratedField::AcceptInvitationIntent),
                            "createPolicyIntent" | "create_policy_intent" => Ok(GeneratedField::CreatePolicyIntent),
                            "disablePrivateKeyIntent" | "disable_private_key_intent" => Ok(GeneratedField::DisablePrivateKeyIntent),
                            "deleteUsersIntent" | "delete_users_intent" => Ok(GeneratedField::DeleteUsersIntent),
                            "deleteAuthenticatorsIntent" | "delete_authenticators_intent" => Ok(GeneratedField::DeleteAuthenticatorsIntent),
                            "deleteInvitationIntent" | "delete_invitation_intent" => Ok(GeneratedField::DeleteInvitationIntent),
                            "deleteOrganizationIntent" | "delete_organization_intent" => Ok(GeneratedField::DeleteOrganizationIntent),
                            "deletePolicyIntent" | "delete_policy_intent" => Ok(GeneratedField::DeletePolicyIntent),
                            "createUserTagIntent" | "create_user_tag_intent" => Ok(GeneratedField::CreateUserTagIntent),
                            "deleteUserTagsIntent" | "delete_user_tags_intent" => Ok(GeneratedField::DeleteUserTagsIntent),
                            "signTransactionIntent" | "sign_transaction_intent" => Ok(GeneratedField::SignTransactionIntent),
                            "createApiKeysIntent" | "create_api_keys_intent" => Ok(GeneratedField::CreateApiKeysIntent),
                            "deleteApiKeysIntent" | "delete_api_keys_intent" => Ok(GeneratedField::DeleteApiKeysIntent),
                            "approveActivityIntent" | "approve_activity_intent" => Ok(GeneratedField::ApproveActivityIntent),
                            "rejectActivityIntent" | "reject_activity_intent" => Ok(GeneratedField::RejectActivityIntent),
                            "createPrivateKeyTagIntent" | "create_private_key_tag_intent" => Ok(GeneratedField::CreatePrivateKeyTagIntent),
                            "deletePrivateKeyTagsIntent" | "delete_private_key_tags_intent" => Ok(GeneratedField::DeletePrivateKeyTagsIntent),
                            "createPolicyIntentV2" | "create_policy_intent_v2" => Ok(GeneratedField::CreatePolicyIntentV2),
                            "setPaymentMethodIntent" | "set_payment_method_intent" => Ok(GeneratedField::SetPaymentMethodIntent),
                            "activateBillingTierIntent" | "activate_billing_tier_intent" => Ok(GeneratedField::ActivateBillingTierIntent),
                            "deletePaymentMethodIntent" | "delete_payment_method_intent" => Ok(GeneratedField::DeletePaymentMethodIntent),
                            "createPolicyIntentV3" | "create_policy_intent_v3" => Ok(GeneratedField::CreatePolicyIntentV3),
                            "createApiOnlyUsersIntent" | "create_api_only_users_intent" => Ok(GeneratedField::CreateApiOnlyUsersIntent),
                            "updateRootQuorumIntent" | "update_root_quorum_intent" => Ok(GeneratedField::UpdateRootQuorumIntent),
                            "updateUserTagIntent" | "update_user_tag_intent" => Ok(GeneratedField::UpdateUserTagIntent),
                            "updatePrivateKeyTagIntent" | "update_private_key_tag_intent" => Ok(GeneratedField::UpdatePrivateKeyTagIntent),
                            "createAuthenticatorsIntentV2" | "create_authenticators_intent_v2" => Ok(GeneratedField::CreateAuthenticatorsIntentV2),
                            "acceptInvitationIntentV2" | "accept_invitation_intent_v2" => Ok(GeneratedField::AcceptInvitationIntentV2),
                            "createOrganizationIntentV2" | "create_organization_intent_v2" => Ok(GeneratedField::CreateOrganizationIntentV2),
                            "createUsersIntentV2" | "create_users_intent_v2" => Ok(GeneratedField::CreateUsersIntentV2),
                            "createSubOrganizationIntent" | "create_sub_organization_intent" => Ok(GeneratedField::CreateSubOrganizationIntent),
                            "createSubOrganizationIntentV2" | "create_sub_organization_intent_v2" => Ok(GeneratedField::CreateSubOrganizationIntentV2),
                            "updateAllowedOriginsIntent" | "update_allowed_origins_intent" => Ok(GeneratedField::UpdateAllowedOriginsIntent),
                            "createPrivateKeysIntentV2" | "create_private_keys_intent_v2" => Ok(GeneratedField::CreatePrivateKeysIntentV2),
                            "updateUserIntent" | "update_user_intent" => Ok(GeneratedField::UpdateUserIntent),
                            "updatePolicyIntent" | "update_policy_intent" => Ok(GeneratedField::UpdatePolicyIntent),
                            "setPaymentMethodIntentV2" | "set_payment_method_intent_v2" => Ok(GeneratedField::SetPaymentMethodIntentV2),
                            "createSubOrganizationIntentV3" | "create_sub_organization_intent_v3" => Ok(GeneratedField::CreateSubOrganizationIntentV3),
                            "createWalletIntent" | "create_wallet_intent" => Ok(GeneratedField::CreateWalletIntent),
                            "createWalletAccountsIntent" | "create_wallet_accounts_intent" => Ok(GeneratedField::CreateWalletAccountsIntent),
                            "initUserEmailRecoveryIntent" | "init_user_email_recovery_intent" => Ok(GeneratedField::InitUserEmailRecoveryIntent),
                            "recoverUserIntent" | "recover_user_intent" => Ok(GeneratedField::RecoverUserIntent),
                            "setOrganizationFeatureIntent" | "set_organization_feature_intent" => Ok(GeneratedField::SetOrganizationFeatureIntent),
                            "removeOrganizationFeatureIntent" | "remove_organization_feature_intent" => Ok(GeneratedField::RemoveOrganizationFeatureIntent),
                            "signRawPayloadIntentV2" | "sign_raw_payload_intent_v2" => Ok(GeneratedField::SignRawPayloadIntentV2),
                            "signTransactionIntentV2" | "sign_transaction_intent_v2" => Ok(GeneratedField::SignTransactionIntentV2),
                            "exportPrivateKeyIntent" | "export_private_key_intent" => Ok(GeneratedField::ExportPrivateKeyIntent),
                            "exportWalletIntent" | "export_wallet_intent" => Ok(GeneratedField::ExportWalletIntent),
                            "createSubOrganizationIntentV4" | "create_sub_organization_intent_v4" => Ok(GeneratedField::CreateSubOrganizationIntentV4),
                            "emailAuthIntent" | "email_auth_intent" => Ok(GeneratedField::EmailAuthIntent),
                            "exportWalletAccountIntent" | "export_wallet_account_intent" => Ok(GeneratedField::ExportWalletAccountIntent),
                            "initImportWalletIntent" | "init_import_wallet_intent" => Ok(GeneratedField::InitImportWalletIntent),
                            "importWalletIntent" | "import_wallet_intent" => Ok(GeneratedField::ImportWalletIntent),
                            "initImportPrivateKeyIntent" | "init_import_private_key_intent" => Ok(GeneratedField::InitImportPrivateKeyIntent),
                            "importPrivateKeyIntent" | "import_private_key_intent" => Ok(GeneratedField::ImportPrivateKeyIntent),
                            "createPoliciesIntent" | "create_policies_intent" => Ok(GeneratedField::CreatePoliciesIntent),
                            "signRawPayloadsIntent" | "sign_raw_payloads_intent" => Ok(GeneratedField::SignRawPayloadsIntent),
                            "createReadOnlySessionIntent" | "create_read_only_session_intent" => Ok(GeneratedField::CreateReadOnlySessionIntent),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Intent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.Intent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Intent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CreateOrganizationIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createOrganizationIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateOrganizationIntent)
;
                        }
                        GeneratedField::CreateAuthenticatorsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createAuthenticatorsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateAuthenticatorsIntent)
;
                        }
                        GeneratedField::CreateUsersIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createUsersIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateUsersIntent)
;
                        }
                        GeneratedField::CreatePrivateKeysIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPrivateKeysIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePrivateKeysIntent)
;
                        }
                        GeneratedField::SignRawPayloadIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signRawPayloadIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SignRawPayloadIntent)
;
                        }
                        GeneratedField::CreateInvitationsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createInvitationsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateInvitationsIntent)
;
                        }
                        GeneratedField::AcceptInvitationIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acceptInvitationIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::AcceptInvitationIntent)
;
                        }
                        GeneratedField::CreatePolicyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPolicyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePolicyIntent)
;
                        }
                        GeneratedField::DisablePrivateKeyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disablePrivateKeyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DisablePrivateKeyIntent)
;
                        }
                        GeneratedField::DeleteUsersIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteUsersIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeleteUsersIntent)
;
                        }
                        GeneratedField::DeleteAuthenticatorsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteAuthenticatorsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeleteAuthenticatorsIntent)
;
                        }
                        GeneratedField::DeleteInvitationIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteInvitationIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeleteInvitationIntent)
;
                        }
                        GeneratedField::DeleteOrganizationIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteOrganizationIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeleteOrganizationIntent)
;
                        }
                        GeneratedField::DeletePolicyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deletePolicyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeletePolicyIntent)
;
                        }
                        GeneratedField::CreateUserTagIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createUserTagIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateUserTagIntent)
;
                        }
                        GeneratedField::DeleteUserTagsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteUserTagsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeleteUserTagsIntent)
;
                        }
                        GeneratedField::SignTransactionIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signTransactionIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SignTransactionIntent)
;
                        }
                        GeneratedField::CreateApiKeysIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createApiKeysIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateApiKeysIntent)
;
                        }
                        GeneratedField::DeleteApiKeysIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteApiKeysIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeleteApiKeysIntent)
;
                        }
                        GeneratedField::ApproveActivityIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("approveActivityIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ApproveActivityIntent)
;
                        }
                        GeneratedField::RejectActivityIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rejectActivityIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::RejectActivityIntent)
;
                        }
                        GeneratedField::CreatePrivateKeyTagIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPrivateKeyTagIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePrivateKeyTagIntent)
;
                        }
                        GeneratedField::DeletePrivateKeyTagsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deletePrivateKeyTagsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeletePrivateKeyTagsIntent)
;
                        }
                        GeneratedField::CreatePolicyIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPolicyIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePolicyIntentV2)
;
                        }
                        GeneratedField::SetPaymentMethodIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setPaymentMethodIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SetPaymentMethodIntent)
;
                        }
                        GeneratedField::ActivateBillingTierIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("activateBillingTierIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ActivateBillingTierIntent)
;
                        }
                        GeneratedField::DeletePaymentMethodIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deletePaymentMethodIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::DeletePaymentMethodIntent)
;
                        }
                        GeneratedField::CreatePolicyIntentV3 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPolicyIntentV3"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePolicyIntentV3)
;
                        }
                        GeneratedField::CreateApiOnlyUsersIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createApiOnlyUsersIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateApiOnlyUsersIntent)
;
                        }
                        GeneratedField::UpdateRootQuorumIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateRootQuorumIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::UpdateRootQuorumIntent)
;
                        }
                        GeneratedField::UpdateUserTagIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateUserTagIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::UpdateUserTagIntent)
;
                        }
                        GeneratedField::UpdatePrivateKeyTagIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatePrivateKeyTagIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::UpdatePrivateKeyTagIntent)
;
                        }
                        GeneratedField::CreateAuthenticatorsIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createAuthenticatorsIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateAuthenticatorsIntentV2)
;
                        }
                        GeneratedField::AcceptInvitationIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acceptInvitationIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::AcceptInvitationIntentV2)
;
                        }
                        GeneratedField::CreateOrganizationIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createOrganizationIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateOrganizationIntentV2)
;
                        }
                        GeneratedField::CreateUsersIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createUsersIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateUsersIntentV2)
;
                        }
                        GeneratedField::CreateSubOrganizationIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateSubOrganizationIntent)
;
                        }
                        GeneratedField::CreateSubOrganizationIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateSubOrganizationIntentV2)
;
                        }
                        GeneratedField::UpdateAllowedOriginsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateAllowedOriginsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::UpdateAllowedOriginsIntent)
;
                        }
                        GeneratedField::CreatePrivateKeysIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPrivateKeysIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePrivateKeysIntentV2)
;
                        }
                        GeneratedField::UpdateUserIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateUserIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::UpdateUserIntent)
;
                        }
                        GeneratedField::UpdatePolicyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatePolicyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::UpdatePolicyIntent)
;
                        }
                        GeneratedField::SetPaymentMethodIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setPaymentMethodIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SetPaymentMethodIntentV2)
;
                        }
                        GeneratedField::CreateSubOrganizationIntentV3 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationIntentV3"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateSubOrganizationIntentV3)
;
                        }
                        GeneratedField::CreateWalletIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createWalletIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateWalletIntent)
;
                        }
                        GeneratedField::CreateWalletAccountsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createWalletAccountsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateWalletAccountsIntent)
;
                        }
                        GeneratedField::InitUserEmailRecoveryIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initUserEmailRecoveryIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::InitUserEmailRecoveryIntent)
;
                        }
                        GeneratedField::RecoverUserIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recoverUserIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::RecoverUserIntent)
;
                        }
                        GeneratedField::SetOrganizationFeatureIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setOrganizationFeatureIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SetOrganizationFeatureIntent)
;
                        }
                        GeneratedField::RemoveOrganizationFeatureIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("removeOrganizationFeatureIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::RemoveOrganizationFeatureIntent)
;
                        }
                        GeneratedField::SignRawPayloadIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signRawPayloadIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SignRawPayloadIntentV2)
;
                        }
                        GeneratedField::SignTransactionIntentV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signTransactionIntentV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SignTransactionIntentV2)
;
                        }
                        GeneratedField::ExportPrivateKeyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportPrivateKeyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ExportPrivateKeyIntent)
;
                        }
                        GeneratedField::ExportWalletIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportWalletIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ExportWalletIntent)
;
                        }
                        GeneratedField::CreateSubOrganizationIntentV4 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationIntentV4"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateSubOrganizationIntentV4)
;
                        }
                        GeneratedField::EmailAuthIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emailAuthIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::EmailAuthIntent)
;
                        }
                        GeneratedField::ExportWalletAccountIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportWalletAccountIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ExportWalletAccountIntent)
;
                        }
                        GeneratedField::InitImportWalletIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initImportWalletIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::InitImportWalletIntent)
;
                        }
                        GeneratedField::ImportWalletIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importWalletIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ImportWalletIntent)
;
                        }
                        GeneratedField::InitImportPrivateKeyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initImportPrivateKeyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::InitImportPrivateKeyIntent)
;
                        }
                        GeneratedField::ImportPrivateKeyIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importPrivateKeyIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::ImportPrivateKeyIntent)
;
                        }
                        GeneratedField::CreatePoliciesIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPoliciesIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreatePoliciesIntent)
;
                        }
                        GeneratedField::SignRawPayloadsIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signRawPayloadsIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::SignRawPayloadsIntent)
;
                        }
                        GeneratedField::CreateReadOnlySessionIntent => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createReadOnlySessionIntent"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(intent::Inner::CreateReadOnlySessionIntent)
;
                        }
                    }
                }
                Ok(Intent {
                    inner: inner__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.Intent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InvitationParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.InvitationParams", len)?;
        if true {
            struct_ser.serialize_field("receiverUserName", &self.receiver_user_name)?;
        }
        if true {
            struct_ser.serialize_field("receiverUserEmail", &self.receiver_user_email)?;
        }
        if true {
            struct_ser.serialize_field("receiverUserTags", &self.receiver_user_tags)?;
        }
        if true {
            let v = super::super::common::v1::AccessType::try_from(self.access_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.access_type)))?;
            struct_ser.serialize_field("accessType", &v)?;
        }
        if true {
            struct_ser.serialize_field("senderUserId", &self.sender_user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InvitationParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "receiver_user_name",
            "receiverUserName",
            "receiver_user_email",
            "receiverUserEmail",
            "receiver_user_tags",
            "receiverUserTags",
            "access_type",
            "accessType",
            "sender_user_id",
            "senderUserId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ReceiverUserName,
            ReceiverUserEmail,
            ReceiverUserTags,
            AccessType,
            SenderUserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "receiverUserName" | "receiver_user_name" => Ok(GeneratedField::ReceiverUserName),
                            "receiverUserEmail" | "receiver_user_email" => Ok(GeneratedField::ReceiverUserEmail),
                            "receiverUserTags" | "receiver_user_tags" => Ok(GeneratedField::ReceiverUserTags),
                            "accessType" | "access_type" => Ok(GeneratedField::AccessType),
                            "senderUserId" | "sender_user_id" => Ok(GeneratedField::SenderUserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InvitationParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.InvitationParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InvitationParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut receiver_user_name__ = None;
                let mut receiver_user_email__ = None;
                let mut receiver_user_tags__ = None;
                let mut access_type__ = None;
                let mut sender_user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ReceiverUserName => {
                            if receiver_user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiverUserName"));
                            }
                            receiver_user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReceiverUserEmail => {
                            if receiver_user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiverUserEmail"));
                            }
                            receiver_user_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReceiverUserTags => {
                            if receiver_user_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiverUserTags"));
                            }
                            receiver_user_tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AccessType => {
                            if access_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessType"));
                            }
                            access_type__ = Some(map_.next_value::<super::super::common::v1::AccessType>()? as i32);
                        }
                        GeneratedField::SenderUserId => {
                            if sender_user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("senderUserId"));
                            }
                            sender_user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(InvitationParams {
                    receiver_user_name: receiver_user_name__.unwrap_or_default(),
                    receiver_user_email: receiver_user_email__.unwrap_or_default(),
                    receiver_user_tags: receiver_user_tags__.unwrap_or_default(),
                    access_type: access_type__.unwrap_or_default(),
                    sender_user_id: sender_user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.InvitationParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PrivateKeyParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.PrivateKeyParams", len)?;
        if true {
            struct_ser.serialize_field("privateKeyName", &self.private_key_name)?;
        }
        if true {
            let v = super::super::common::v1::Curve::try_from(self.curve)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.curve)))?;
            struct_ser.serialize_field("curve", &v)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyTags", &self.private_key_tags)?;
        }
        if true {
            let v = self.address_formats.iter().cloned().map(|v| {
                super::super::common::v1::AddressFormat::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("addressFormats", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PrivateKeyParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_name",
            "privateKeyName",
            "curve",
            "private_key_tags",
            "privateKeyTags",
            "address_formats",
            "addressFormats",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyName,
            Curve,
            PrivateKeyTags,
            AddressFormats,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyName" | "private_key_name" => Ok(GeneratedField::PrivateKeyName),
                            "curve" => Ok(GeneratedField::Curve),
                            "privateKeyTags" | "private_key_tags" => Ok(GeneratedField::PrivateKeyTags),
                            "addressFormats" | "address_formats" => Ok(GeneratedField::AddressFormats),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PrivateKeyParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.PrivateKeyParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PrivateKeyParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_name__ = None;
                let mut curve__ = None;
                let mut private_key_tags__ = None;
                let mut address_formats__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyName => {
                            if private_key_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyName"));
                            }
                            private_key_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Curve => {
                            if curve__.is_some() {
                                return Err(serde::de::Error::duplicate_field("curve"));
                            }
                            curve__ = Some(map_.next_value::<super::super::common::v1::Curve>()? as i32);
                        }
                        GeneratedField::PrivateKeyTags => {
                            if private_key_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTags"));
                            }
                            private_key_tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AddressFormats => {
                            if address_formats__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressFormats"));
                            }
                            address_formats__ = Some(map_.next_value::<Vec<super::super::common::v1::AddressFormat>>()?.into_iter().map(|x| x as i32).collect());
                        }
                    }
                }
                Ok(PrivateKeyParams {
                    private_key_name: private_key_name__.unwrap_or_default(),
                    curve: curve__.unwrap_or_default(),
                    private_key_tags: private_key_tags__.unwrap_or_default(),
                    address_formats: address_formats__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.PrivateKeyParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PrivateKeyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.PrivateKeyResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PrivateKeyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            Addresses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PrivateKeyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.PrivateKeyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PrivateKeyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut addresses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(PrivateKeyResult {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.PrivateKeyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RecoverUserIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.RecoverUserIntent", len)?;
        if let Some(v) = self.authenticator.as_ref() {
            struct_ser.serialize_field("authenticator", v)?;
        }
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RecoverUserIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticator",
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authenticator,
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticator" => Ok(GeneratedField::Authenticator),
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RecoverUserIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.RecoverUserIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RecoverUserIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticator__ = None;
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Authenticator => {
                            if authenticator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticator"));
                            }
                            authenticator__ = map_.next_value()?;
                        }
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RecoverUserIntent {
                    authenticator: authenticator__,
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.RecoverUserIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RecoverUserResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.RecoverUserResult", len)?;
        if true {
            struct_ser.serialize_field("authenticatorId", &self.authenticator_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RecoverUserResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authenticator_id",
            "authenticatorId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AuthenticatorId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "authenticatorId" | "authenticator_id" => Ok(GeneratedField::AuthenticatorId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RecoverUserResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.RecoverUserResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RecoverUserResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authenticator_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AuthenticatorId => {
                            if authenticator_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorId"));
                            }
                            authenticator_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RecoverUserResult {
                    authenticator_id: authenticator_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.RecoverUserResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RejectActivityIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.RejectActivityIntent", len)?;
        if true {
            struct_ser.serialize_field("fingerprint", &self.fingerprint)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RejectActivityIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fingerprint",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fingerprint,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "fingerprint" => Ok(GeneratedField::Fingerprint),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RejectActivityIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.RejectActivityIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RejectActivityIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fingerprint__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Fingerprint => {
                            if fingerprint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fingerprint"));
                            }
                            fingerprint__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RejectActivityIntent {
                    fingerprint: fingerprint__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.RejectActivityIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoveOrganizationFeatureIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.RemoveOrganizationFeatureIntent", len)?;
        if true {
            let v = super::super::common::v1::FeatureName::try_from(self.name)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.name)))?;
            struct_ser.serialize_field("name", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoveOrganizationFeatureIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RemoveOrganizationFeatureIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.RemoveOrganizationFeatureIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RemoveOrganizationFeatureIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value::<super::super::common::v1::FeatureName>()? as i32);
                        }
                    }
                }
                Ok(RemoveOrganizationFeatureIntent {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.RemoveOrganizationFeatureIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoveOrganizationFeatureResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.RemoveOrganizationFeatureResult", len)?;
        if true {
            struct_ser.serialize_field("features", &self.features)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoveOrganizationFeatureResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "features",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Features,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "features" => Ok(GeneratedField::Features),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RemoveOrganizationFeatureResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.RemoveOrganizationFeatureResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RemoveOrganizationFeatureResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut features__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Features => {
                            if features__.is_some() {
                                return Err(serde::de::Error::duplicate_field("features"));
                            }
                            features__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RemoveOrganizationFeatureResult {
                    features: features__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.RemoveOrganizationFeatureResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Result {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.inner.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.Result", len)?;
        if let Some(v) = self.inner.as_ref() {
            match v {
                result::Inner::CreateOrganizationResult(v) => {
                    struct_ser.serialize_field("createOrganizationResult", v)?;
                }
                result::Inner::CreateAuthenticatorsResult(v) => {
                    struct_ser.serialize_field("createAuthenticatorsResult", v)?;
                }
                result::Inner::CreateUsersResult(v) => {
                    struct_ser.serialize_field("createUsersResult", v)?;
                }
                result::Inner::CreatePrivateKeysResult(v) => {
                    struct_ser.serialize_field("createPrivateKeysResult", v)?;
                }
                result::Inner::CreateInvitationsResult(v) => {
                    struct_ser.serialize_field("createInvitationsResult", v)?;
                }
                result::Inner::AcceptInvitationResult(v) => {
                    struct_ser.serialize_field("acceptInvitationResult", v)?;
                }
                result::Inner::SignRawPayloadResult(v) => {
                    struct_ser.serialize_field("signRawPayloadResult", v)?;
                }
                result::Inner::CreatePolicyResult(v) => {
                    struct_ser.serialize_field("createPolicyResult", v)?;
                }
                result::Inner::DisablePrivateKeyResult(v) => {
                    struct_ser.serialize_field("disablePrivateKeyResult", v)?;
                }
                result::Inner::DeleteUsersResult(v) => {
                    struct_ser.serialize_field("deleteUsersResult", v)?;
                }
                result::Inner::DeleteAuthenticatorsResult(v) => {
                    struct_ser.serialize_field("deleteAuthenticatorsResult", v)?;
                }
                result::Inner::DeleteInvitationResult(v) => {
                    struct_ser.serialize_field("deleteInvitationResult", v)?;
                }
                result::Inner::DeleteOrganizationResult(v) => {
                    struct_ser.serialize_field("deleteOrganizationResult", v)?;
                }
                result::Inner::DeletePolicyResult(v) => {
                    struct_ser.serialize_field("deletePolicyResult", v)?;
                }
                result::Inner::CreateUserTagResult(v) => {
                    struct_ser.serialize_field("createUserTagResult", v)?;
                }
                result::Inner::DeleteUserTagsResult(v) => {
                    struct_ser.serialize_field("deleteUserTagsResult", v)?;
                }
                result::Inner::SignTransactionResult(v) => {
                    struct_ser.serialize_field("signTransactionResult", v)?;
                }
                result::Inner::DeleteApiKeysResult(v) => {
                    struct_ser.serialize_field("deleteApiKeysResult", v)?;
                }
                result::Inner::CreateApiKeysResult(v) => {
                    struct_ser.serialize_field("createApiKeysResult", v)?;
                }
                result::Inner::CreatePrivateKeyTagResult(v) => {
                    struct_ser.serialize_field("createPrivateKeyTagResult", v)?;
                }
                result::Inner::DeletePrivateKeyTagsResult(v) => {
                    struct_ser.serialize_field("deletePrivateKeyTagsResult", v)?;
                }
                result::Inner::SetPaymentMethodResult(v) => {
                    struct_ser.serialize_field("setPaymentMethodResult", v)?;
                }
                result::Inner::ActivateBillingTierResult(v) => {
                    struct_ser.serialize_field("activateBillingTierResult", v)?;
                }
                result::Inner::DeletePaymentMethodResult(v) => {
                    struct_ser.serialize_field("deletePaymentMethodResult", v)?;
                }
                result::Inner::CreateApiOnlyUsersResult(v) => {
                    struct_ser.serialize_field("createApiOnlyUsersResult", v)?;
                }
                result::Inner::UpdateRootQuorumResult(v) => {
                    struct_ser.serialize_field("updateRootQuorumResult", v)?;
                }
                result::Inner::UpdateUserTagResult(v) => {
                    struct_ser.serialize_field("updateUserTagResult", v)?;
                }
                result::Inner::UpdatePrivateKeyTagResult(v) => {
                    struct_ser.serialize_field("updatePrivateKeyTagResult", v)?;
                }
                result::Inner::CreateSubOrganizationResult(v) => {
                    struct_ser.serialize_field("createSubOrganizationResult", v)?;
                }
                result::Inner::UpdateAllowedOriginsResult(v) => {
                    struct_ser.serialize_field("updateAllowedOriginsResult", v)?;
                }
                result::Inner::CreatePrivateKeysResultV2(v) => {
                    struct_ser.serialize_field("createPrivateKeysResultV2", v)?;
                }
                result::Inner::UpdateUserResult(v) => {
                    struct_ser.serialize_field("updateUserResult", v)?;
                }
                result::Inner::UpdatePolicyResult(v) => {
                    struct_ser.serialize_field("updatePolicyResult", v)?;
                }
                result::Inner::CreateSubOrganizationResultV3(v) => {
                    struct_ser.serialize_field("createSubOrganizationResultV3", v)?;
                }
                result::Inner::CreateWalletResult(v) => {
                    struct_ser.serialize_field("createWalletResult", v)?;
                }
                result::Inner::CreateWalletAccountsResult(v) => {
                    struct_ser.serialize_field("createWalletAccountsResult", v)?;
                }
                result::Inner::InitUserEmailRecoveryResult(v) => {
                    struct_ser.serialize_field("initUserEmailRecoveryResult", v)?;
                }
                result::Inner::RecoverUserResult(v) => {
                    struct_ser.serialize_field("recoverUserResult", v)?;
                }
                result::Inner::SetOrganizationFeatureResult(v) => {
                    struct_ser.serialize_field("setOrganizationFeatureResult", v)?;
                }
                result::Inner::RemoveOrganizationFeatureResult(v) => {
                    struct_ser.serialize_field("removeOrganizationFeatureResult", v)?;
                }
                result::Inner::ExportPrivateKeyResult(v) => {
                    struct_ser.serialize_field("exportPrivateKeyResult", v)?;
                }
                result::Inner::ExportWalletResult(v) => {
                    struct_ser.serialize_field("exportWalletResult", v)?;
                }
                result::Inner::CreateSubOrganizationResultV4(v) => {
                    struct_ser.serialize_field("createSubOrganizationResultV4", v)?;
                }
                result::Inner::EmailAuthResult(v) => {
                    struct_ser.serialize_field("emailAuthResult", v)?;
                }
                result::Inner::ExportWalletAccountResult(v) => {
                    struct_ser.serialize_field("exportWalletAccountResult", v)?;
                }
                result::Inner::InitImportWalletResult(v) => {
                    struct_ser.serialize_field("initImportWalletResult", v)?;
                }
                result::Inner::ImportWalletResult(v) => {
                    struct_ser.serialize_field("importWalletResult", v)?;
                }
                result::Inner::InitImportPrivateKeyResult(v) => {
                    struct_ser.serialize_field("initImportPrivateKeyResult", v)?;
                }
                result::Inner::ImportPrivateKeyResult(v) => {
                    struct_ser.serialize_field("importPrivateKeyResult", v)?;
                }
                result::Inner::CreatePoliciesResult(v) => {
                    struct_ser.serialize_field("createPoliciesResult", v)?;
                }
                result::Inner::SignRawPayloadsResult(v) => {
                    struct_ser.serialize_field("signRawPayloadsResult", v)?;
                }
                result::Inner::CreateReadOnlySessionResult(v) => {
                    struct_ser.serialize_field("createReadOnlySessionResult", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Result {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "create_organization_result",
            "createOrganizationResult",
            "create_authenticators_result",
            "createAuthenticatorsResult",
            "create_users_result",
            "createUsersResult",
            "create_private_keys_result",
            "createPrivateKeysResult",
            "create_invitations_result",
            "createInvitationsResult",
            "accept_invitation_result",
            "acceptInvitationResult",
            "sign_raw_payload_result",
            "signRawPayloadResult",
            "create_policy_result",
            "createPolicyResult",
            "disable_private_key_result",
            "disablePrivateKeyResult",
            "delete_users_result",
            "deleteUsersResult",
            "delete_authenticators_result",
            "deleteAuthenticatorsResult",
            "delete_invitation_result",
            "deleteInvitationResult",
            "delete_organization_result",
            "deleteOrganizationResult",
            "delete_policy_result",
            "deletePolicyResult",
            "create_user_tag_result",
            "createUserTagResult",
            "delete_user_tags_result",
            "deleteUserTagsResult",
            "sign_transaction_result",
            "signTransactionResult",
            "delete_api_keys_result",
            "deleteApiKeysResult",
            "create_api_keys_result",
            "createApiKeysResult",
            "create_private_key_tag_result",
            "createPrivateKeyTagResult",
            "delete_private_key_tags_result",
            "deletePrivateKeyTagsResult",
            "set_payment_method_result",
            "setPaymentMethodResult",
            "activate_billing_tier_result",
            "activateBillingTierResult",
            "delete_payment_method_result",
            "deletePaymentMethodResult",
            "create_api_only_users_result",
            "createApiOnlyUsersResult",
            "update_root_quorum_result",
            "updateRootQuorumResult",
            "update_user_tag_result",
            "updateUserTagResult",
            "update_private_key_tag_result",
            "updatePrivateKeyTagResult",
            "create_sub_organization_result",
            "createSubOrganizationResult",
            "update_allowed_origins_result",
            "updateAllowedOriginsResult",
            "create_private_keys_result_v2",
            "createPrivateKeysResultV2",
            "update_user_result",
            "updateUserResult",
            "update_policy_result",
            "updatePolicyResult",
            "create_sub_organization_result_v3",
            "createSubOrganizationResultV3",
            "create_wallet_result",
            "createWalletResult",
            "create_wallet_accounts_result",
            "createWalletAccountsResult",
            "init_user_email_recovery_result",
            "initUserEmailRecoveryResult",
            "recover_user_result",
            "recoverUserResult",
            "set_organization_feature_result",
            "setOrganizationFeatureResult",
            "remove_organization_feature_result",
            "removeOrganizationFeatureResult",
            "export_private_key_result",
            "exportPrivateKeyResult",
            "export_wallet_result",
            "exportWalletResult",
            "create_sub_organization_result_v4",
            "createSubOrganizationResultV4",
            "email_auth_result",
            "emailAuthResult",
            "export_wallet_account_result",
            "exportWalletAccountResult",
            "init_import_wallet_result",
            "initImportWalletResult",
            "import_wallet_result",
            "importWalletResult",
            "init_import_private_key_result",
            "initImportPrivateKeyResult",
            "import_private_key_result",
            "importPrivateKeyResult",
            "create_policies_result",
            "createPoliciesResult",
            "sign_raw_payloads_result",
            "signRawPayloadsResult",
            "create_read_only_session_result",
            "createReadOnlySessionResult",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CreateOrganizationResult,
            CreateAuthenticatorsResult,
            CreateUsersResult,
            CreatePrivateKeysResult,
            CreateInvitationsResult,
            AcceptInvitationResult,
            SignRawPayloadResult,
            CreatePolicyResult,
            DisablePrivateKeyResult,
            DeleteUsersResult,
            DeleteAuthenticatorsResult,
            DeleteInvitationResult,
            DeleteOrganizationResult,
            DeletePolicyResult,
            CreateUserTagResult,
            DeleteUserTagsResult,
            SignTransactionResult,
            DeleteApiKeysResult,
            CreateApiKeysResult,
            CreatePrivateKeyTagResult,
            DeletePrivateKeyTagsResult,
            SetPaymentMethodResult,
            ActivateBillingTierResult,
            DeletePaymentMethodResult,
            CreateApiOnlyUsersResult,
            UpdateRootQuorumResult,
            UpdateUserTagResult,
            UpdatePrivateKeyTagResult,
            CreateSubOrganizationResult,
            UpdateAllowedOriginsResult,
            CreatePrivateKeysResultV2,
            UpdateUserResult,
            UpdatePolicyResult,
            CreateSubOrganizationResultV3,
            CreateWalletResult,
            CreateWalletAccountsResult,
            InitUserEmailRecoveryResult,
            RecoverUserResult,
            SetOrganizationFeatureResult,
            RemoveOrganizationFeatureResult,
            ExportPrivateKeyResult,
            ExportWalletResult,
            CreateSubOrganizationResultV4,
            EmailAuthResult,
            ExportWalletAccountResult,
            InitImportWalletResult,
            ImportWalletResult,
            InitImportPrivateKeyResult,
            ImportPrivateKeyResult,
            CreatePoliciesResult,
            SignRawPayloadsResult,
            CreateReadOnlySessionResult,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "createOrganizationResult" | "create_organization_result" => Ok(GeneratedField::CreateOrganizationResult),
                            "createAuthenticatorsResult" | "create_authenticators_result" => Ok(GeneratedField::CreateAuthenticatorsResult),
                            "createUsersResult" | "create_users_result" => Ok(GeneratedField::CreateUsersResult),
                            "createPrivateKeysResult" | "create_private_keys_result" => Ok(GeneratedField::CreatePrivateKeysResult),
                            "createInvitationsResult" | "create_invitations_result" => Ok(GeneratedField::CreateInvitationsResult),
                            "acceptInvitationResult" | "accept_invitation_result" => Ok(GeneratedField::AcceptInvitationResult),
                            "signRawPayloadResult" | "sign_raw_payload_result" => Ok(GeneratedField::SignRawPayloadResult),
                            "createPolicyResult" | "create_policy_result" => Ok(GeneratedField::CreatePolicyResult),
                            "disablePrivateKeyResult" | "disable_private_key_result" => Ok(GeneratedField::DisablePrivateKeyResult),
                            "deleteUsersResult" | "delete_users_result" => Ok(GeneratedField::DeleteUsersResult),
                            "deleteAuthenticatorsResult" | "delete_authenticators_result" => Ok(GeneratedField::DeleteAuthenticatorsResult),
                            "deleteInvitationResult" | "delete_invitation_result" => Ok(GeneratedField::DeleteInvitationResult),
                            "deleteOrganizationResult" | "delete_organization_result" => Ok(GeneratedField::DeleteOrganizationResult),
                            "deletePolicyResult" | "delete_policy_result" => Ok(GeneratedField::DeletePolicyResult),
                            "createUserTagResult" | "create_user_tag_result" => Ok(GeneratedField::CreateUserTagResult),
                            "deleteUserTagsResult" | "delete_user_tags_result" => Ok(GeneratedField::DeleteUserTagsResult),
                            "signTransactionResult" | "sign_transaction_result" => Ok(GeneratedField::SignTransactionResult),
                            "deleteApiKeysResult" | "delete_api_keys_result" => Ok(GeneratedField::DeleteApiKeysResult),
                            "createApiKeysResult" | "create_api_keys_result" => Ok(GeneratedField::CreateApiKeysResult),
                            "createPrivateKeyTagResult" | "create_private_key_tag_result" => Ok(GeneratedField::CreatePrivateKeyTagResult),
                            "deletePrivateKeyTagsResult" | "delete_private_key_tags_result" => Ok(GeneratedField::DeletePrivateKeyTagsResult),
                            "setPaymentMethodResult" | "set_payment_method_result" => Ok(GeneratedField::SetPaymentMethodResult),
                            "activateBillingTierResult" | "activate_billing_tier_result" => Ok(GeneratedField::ActivateBillingTierResult),
                            "deletePaymentMethodResult" | "delete_payment_method_result" => Ok(GeneratedField::DeletePaymentMethodResult),
                            "createApiOnlyUsersResult" | "create_api_only_users_result" => Ok(GeneratedField::CreateApiOnlyUsersResult),
                            "updateRootQuorumResult" | "update_root_quorum_result" => Ok(GeneratedField::UpdateRootQuorumResult),
                            "updateUserTagResult" | "update_user_tag_result" => Ok(GeneratedField::UpdateUserTagResult),
                            "updatePrivateKeyTagResult" | "update_private_key_tag_result" => Ok(GeneratedField::UpdatePrivateKeyTagResult),
                            "createSubOrganizationResult" | "create_sub_organization_result" => Ok(GeneratedField::CreateSubOrganizationResult),
                            "updateAllowedOriginsResult" | "update_allowed_origins_result" => Ok(GeneratedField::UpdateAllowedOriginsResult),
                            "createPrivateKeysResultV2" | "create_private_keys_result_v2" => Ok(GeneratedField::CreatePrivateKeysResultV2),
                            "updateUserResult" | "update_user_result" => Ok(GeneratedField::UpdateUserResult),
                            "updatePolicyResult" | "update_policy_result" => Ok(GeneratedField::UpdatePolicyResult),
                            "createSubOrganizationResultV3" | "create_sub_organization_result_v3" => Ok(GeneratedField::CreateSubOrganizationResultV3),
                            "createWalletResult" | "create_wallet_result" => Ok(GeneratedField::CreateWalletResult),
                            "createWalletAccountsResult" | "create_wallet_accounts_result" => Ok(GeneratedField::CreateWalletAccountsResult),
                            "initUserEmailRecoveryResult" | "init_user_email_recovery_result" => Ok(GeneratedField::InitUserEmailRecoveryResult),
                            "recoverUserResult" | "recover_user_result" => Ok(GeneratedField::RecoverUserResult),
                            "setOrganizationFeatureResult" | "set_organization_feature_result" => Ok(GeneratedField::SetOrganizationFeatureResult),
                            "removeOrganizationFeatureResult" | "remove_organization_feature_result" => Ok(GeneratedField::RemoveOrganizationFeatureResult),
                            "exportPrivateKeyResult" | "export_private_key_result" => Ok(GeneratedField::ExportPrivateKeyResult),
                            "exportWalletResult" | "export_wallet_result" => Ok(GeneratedField::ExportWalletResult),
                            "createSubOrganizationResultV4" | "create_sub_organization_result_v4" => Ok(GeneratedField::CreateSubOrganizationResultV4),
                            "emailAuthResult" | "email_auth_result" => Ok(GeneratedField::EmailAuthResult),
                            "exportWalletAccountResult" | "export_wallet_account_result" => Ok(GeneratedField::ExportWalletAccountResult),
                            "initImportWalletResult" | "init_import_wallet_result" => Ok(GeneratedField::InitImportWalletResult),
                            "importWalletResult" | "import_wallet_result" => Ok(GeneratedField::ImportWalletResult),
                            "initImportPrivateKeyResult" | "init_import_private_key_result" => Ok(GeneratedField::InitImportPrivateKeyResult),
                            "importPrivateKeyResult" | "import_private_key_result" => Ok(GeneratedField::ImportPrivateKeyResult),
                            "createPoliciesResult" | "create_policies_result" => Ok(GeneratedField::CreatePoliciesResult),
                            "signRawPayloadsResult" | "sign_raw_payloads_result" => Ok(GeneratedField::SignRawPayloadsResult),
                            "createReadOnlySessionResult" | "create_read_only_session_result" => Ok(GeneratedField::CreateReadOnlySessionResult),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Result;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.Result")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Result, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CreateOrganizationResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createOrganizationResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateOrganizationResult)
;
                        }
                        GeneratedField::CreateAuthenticatorsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createAuthenticatorsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateAuthenticatorsResult)
;
                        }
                        GeneratedField::CreateUsersResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createUsersResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateUsersResult)
;
                        }
                        GeneratedField::CreatePrivateKeysResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPrivateKeysResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreatePrivateKeysResult)
;
                        }
                        GeneratedField::CreateInvitationsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createInvitationsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateInvitationsResult)
;
                        }
                        GeneratedField::AcceptInvitationResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("acceptInvitationResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::AcceptInvitationResult)
;
                        }
                        GeneratedField::SignRawPayloadResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signRawPayloadResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::SignRawPayloadResult)
;
                        }
                        GeneratedField::CreatePolicyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPolicyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreatePolicyResult)
;
                        }
                        GeneratedField::DisablePrivateKeyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disablePrivateKeyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DisablePrivateKeyResult)
;
                        }
                        GeneratedField::DeleteUsersResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteUsersResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeleteUsersResult)
;
                        }
                        GeneratedField::DeleteAuthenticatorsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteAuthenticatorsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeleteAuthenticatorsResult)
;
                        }
                        GeneratedField::DeleteInvitationResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteInvitationResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeleteInvitationResult)
;
                        }
                        GeneratedField::DeleteOrganizationResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteOrganizationResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeleteOrganizationResult)
;
                        }
                        GeneratedField::DeletePolicyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deletePolicyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeletePolicyResult)
;
                        }
                        GeneratedField::CreateUserTagResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createUserTagResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateUserTagResult)
;
                        }
                        GeneratedField::DeleteUserTagsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteUserTagsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeleteUserTagsResult)
;
                        }
                        GeneratedField::SignTransactionResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signTransactionResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::SignTransactionResult)
;
                        }
                        GeneratedField::DeleteApiKeysResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deleteApiKeysResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeleteApiKeysResult)
;
                        }
                        GeneratedField::CreateApiKeysResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createApiKeysResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateApiKeysResult)
;
                        }
                        GeneratedField::CreatePrivateKeyTagResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPrivateKeyTagResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreatePrivateKeyTagResult)
;
                        }
                        GeneratedField::DeletePrivateKeyTagsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deletePrivateKeyTagsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeletePrivateKeyTagsResult)
;
                        }
                        GeneratedField::SetPaymentMethodResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setPaymentMethodResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::SetPaymentMethodResult)
;
                        }
                        GeneratedField::ActivateBillingTierResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("activateBillingTierResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::ActivateBillingTierResult)
;
                        }
                        GeneratedField::DeletePaymentMethodResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deletePaymentMethodResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::DeletePaymentMethodResult)
;
                        }
                        GeneratedField::CreateApiOnlyUsersResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createApiOnlyUsersResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateApiOnlyUsersResult)
;
                        }
                        GeneratedField::UpdateRootQuorumResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateRootQuorumResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::UpdateRootQuorumResult)
;
                        }
                        GeneratedField::UpdateUserTagResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateUserTagResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::UpdateUserTagResult)
;
                        }
                        GeneratedField::UpdatePrivateKeyTagResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatePrivateKeyTagResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::UpdatePrivateKeyTagResult)
;
                        }
                        GeneratedField::CreateSubOrganizationResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateSubOrganizationResult)
;
                        }
                        GeneratedField::UpdateAllowedOriginsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateAllowedOriginsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::UpdateAllowedOriginsResult)
;
                        }
                        GeneratedField::CreatePrivateKeysResultV2 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPrivateKeysResultV2"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreatePrivateKeysResultV2)
;
                        }
                        GeneratedField::UpdateUserResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateUserResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::UpdateUserResult)
;
                        }
                        GeneratedField::UpdatePolicyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatePolicyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::UpdatePolicyResult)
;
                        }
                        GeneratedField::CreateSubOrganizationResultV3 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationResultV3"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateSubOrganizationResultV3)
;
                        }
                        GeneratedField::CreateWalletResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createWalletResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateWalletResult)
;
                        }
                        GeneratedField::CreateWalletAccountsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createWalletAccountsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateWalletAccountsResult)
;
                        }
                        GeneratedField::InitUserEmailRecoveryResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initUserEmailRecoveryResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::InitUserEmailRecoveryResult)
;
                        }
                        GeneratedField::RecoverUserResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("recoverUserResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::RecoverUserResult)
;
                        }
                        GeneratedField::SetOrganizationFeatureResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("setOrganizationFeatureResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::SetOrganizationFeatureResult)
;
                        }
                        GeneratedField::RemoveOrganizationFeatureResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("removeOrganizationFeatureResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::RemoveOrganizationFeatureResult)
;
                        }
                        GeneratedField::ExportPrivateKeyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportPrivateKeyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::ExportPrivateKeyResult)
;
                        }
                        GeneratedField::ExportWalletResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportWalletResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::ExportWalletResult)
;
                        }
                        GeneratedField::CreateSubOrganizationResultV4 => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createSubOrganizationResultV4"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateSubOrganizationResultV4)
;
                        }
                        GeneratedField::EmailAuthResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emailAuthResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::EmailAuthResult)
;
                        }
                        GeneratedField::ExportWalletAccountResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exportWalletAccountResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::ExportWalletAccountResult)
;
                        }
                        GeneratedField::InitImportWalletResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initImportWalletResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::InitImportWalletResult)
;
                        }
                        GeneratedField::ImportWalletResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importWalletResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::ImportWalletResult)
;
                        }
                        GeneratedField::InitImportPrivateKeyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initImportPrivateKeyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::InitImportPrivateKeyResult)
;
                        }
                        GeneratedField::ImportPrivateKeyResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("importPrivateKeyResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::ImportPrivateKeyResult)
;
                        }
                        GeneratedField::CreatePoliciesResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createPoliciesResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreatePoliciesResult)
;
                        }
                        GeneratedField::SignRawPayloadsResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signRawPayloadsResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::SignRawPayloadsResult)
;
                        }
                        GeneratedField::CreateReadOnlySessionResult => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createReadOnlySessionResult"));
                            }
                            inner__ = map_.next_value::<::std::option::Option<_>>()?.map(result::Inner::CreateReadOnlySessionResult)
;
                        }
                    }
                }
                Ok(Result {
                    inner: inner__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.Result", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RootUserParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.RootUserParams", len)?;
        if true {
            struct_ser.serialize_field("userName", &self.user_name)?;
        }
        if let Some(v) = self.user_email.as_ref() {
            struct_ser.serialize_field("userEmail", v)?;
        }
        if true {
            struct_ser.serialize_field("apiKeys", &self.api_keys)?;
        }
        if true {
            struct_ser.serialize_field("authenticators", &self.authenticators)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RootUserParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_name",
            "userName",
            "user_email",
            "userEmail",
            "api_keys",
            "apiKeys",
            "authenticators",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserName,
            UserEmail,
            ApiKeys,
            Authenticators,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userName" | "user_name" => Ok(GeneratedField::UserName),
                            "userEmail" | "user_email" => Ok(GeneratedField::UserEmail),
                            "apiKeys" | "api_keys" => Ok(GeneratedField::ApiKeys),
                            "authenticators" => Ok(GeneratedField::Authenticators),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RootUserParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.RootUserParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RootUserParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_name__ = None;
                let mut user_email__ = None;
                let mut api_keys__ = None;
                let mut authenticators__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserName => {
                            if user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userName"));
                            }
                            user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserEmail => {
                            if user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userEmail"));
                            }
                            user_email__ = map_.next_value()?;
                        }
                        GeneratedField::ApiKeys => {
                            if api_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeys"));
                            }
                            api_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Authenticators => {
                            if authenticators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticators"));
                            }
                            authenticators__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(RootUserParams {
                    user_name: user_name__.unwrap_or_default(),
                    user_email: user_email__,
                    api_keys: api_keys__.unwrap_or_default(),
                    authenticators: authenticators__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.RootUserParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Selector {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.Selector", len)?;
        if true {
            struct_ser.serialize_field("subject", &self.subject)?;
        }
        if true {
            let v = super::super::common::v1::Operator::try_from(self.operator)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.operator)))?;
            struct_ser.serialize_field("operator", &v)?;
        }
        if true {
            struct_ser.serialize_field("target", &self.target)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Selector {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "subject",
            "operator",
            "target",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Subject,
            Operator,
            Target,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subject" => Ok(GeneratedField::Subject),
                            "operator" => Ok(GeneratedField::Operator),
                            "target" => Ok(GeneratedField::Target),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Selector;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.Selector")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Selector, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut subject__ = None;
                let mut operator__ = None;
                let mut target__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Subject => {
                            if subject__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subject"));
                            }
                            subject__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Operator => {
                            if operator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operator"));
                            }
                            operator__ = Some(map_.next_value::<super::super::common::v1::Operator>()? as i32);
                        }
                        GeneratedField::Target => {
                            if target__.is_some() {
                                return Err(serde::de::Error::duplicate_field("target"));
                            }
                            target__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Selector {
                    subject: subject__.unwrap_or_default(),
                    operator: operator__.unwrap_or_default(),
                    target: target__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.Selector", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SelectorV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SelectorV2", len)?;
        if true {
            struct_ser.serialize_field("subject", &self.subject)?;
        }
        if true {
            let v = super::super::common::v1::Operator::try_from(self.operator)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.operator)))?;
            struct_ser.serialize_field("operator", &v)?;
        }
        if true {
            struct_ser.serialize_field("targets", &self.targets)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SelectorV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "subject",
            "operator",
            "targets",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Subject,
            Operator,
            Targets,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "subject" => Ok(GeneratedField::Subject),
                            "operator" => Ok(GeneratedField::Operator),
                            "targets" => Ok(GeneratedField::Targets),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SelectorV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SelectorV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SelectorV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut subject__ = None;
                let mut operator__ = None;
                let mut targets__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Subject => {
                            if subject__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subject"));
                            }
                            subject__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Operator => {
                            if operator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operator"));
                            }
                            operator__ = Some(map_.next_value::<super::super::common::v1::Operator>()? as i32);
                        }
                        GeneratedField::Targets => {
                            if targets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("targets"));
                            }
                            targets__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SelectorV2 {
                    subject: subject__.unwrap_or_default(),
                    operator: operator__.unwrap_or_default(),
                    targets: targets__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SelectorV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetOrganizationFeatureIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SetOrganizationFeatureIntent", len)?;
        if true {
            let v = super::super::common::v1::FeatureName::try_from(self.name)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.name)))?;
            struct_ser.serialize_field("name", &v)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetOrganizationFeatureIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Value,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetOrganizationFeatureIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SetOrganizationFeatureIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetOrganizationFeatureIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value::<super::super::common::v1::FeatureName>()? as i32);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SetOrganizationFeatureIntent {
                    name: name__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SetOrganizationFeatureIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetOrganizationFeatureResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SetOrganizationFeatureResult", len)?;
        if true {
            struct_ser.serialize_field("features", &self.features)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetOrganizationFeatureResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "features",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Features,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "features" => Ok(GeneratedField::Features),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetOrganizationFeatureResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SetOrganizationFeatureResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetOrganizationFeatureResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut features__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Features => {
                            if features__.is_some() {
                                return Err(serde::de::Error::duplicate_field("features"));
                            }
                            features__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetOrganizationFeatureResult {
                    features: features__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SetOrganizationFeatureResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetPaymentMethodIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SetPaymentMethodIntent", len)?;
        if true {
            struct_ser.serialize_field("number", &self.number)?;
        }
        if true {
            struct_ser.serialize_field("cvv", &self.cvv)?;
        }
        if true {
            struct_ser.serialize_field("expiryMonth", &self.expiry_month)?;
        }
        if true {
            struct_ser.serialize_field("expiryYear", &self.expiry_year)?;
        }
        if true {
            struct_ser.serialize_field("cardHolderEmail", &self.card_holder_email)?;
        }
        if true {
            struct_ser.serialize_field("cardHolderName", &self.card_holder_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetPaymentMethodIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "number",
            "cvv",
            "expiry_month",
            "expiryMonth",
            "expiry_year",
            "expiryYear",
            "card_holder_email",
            "cardHolderEmail",
            "card_holder_name",
            "cardHolderName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Number,
            Cvv,
            ExpiryMonth,
            ExpiryYear,
            CardHolderEmail,
            CardHolderName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "number" => Ok(GeneratedField::Number),
                            "cvv" => Ok(GeneratedField::Cvv),
                            "expiryMonth" | "expiry_month" => Ok(GeneratedField::ExpiryMonth),
                            "expiryYear" | "expiry_year" => Ok(GeneratedField::ExpiryYear),
                            "cardHolderEmail" | "card_holder_email" => Ok(GeneratedField::CardHolderEmail),
                            "cardHolderName" | "card_holder_name" => Ok(GeneratedField::CardHolderName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetPaymentMethodIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SetPaymentMethodIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetPaymentMethodIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut number__ = None;
                let mut cvv__ = None;
                let mut expiry_month__ = None;
                let mut expiry_year__ = None;
                let mut card_holder_email__ = None;
                let mut card_holder_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Number => {
                            if number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("number"));
                            }
                            number__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Cvv => {
                            if cvv__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cvv"));
                            }
                            cvv__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExpiryMonth => {
                            if expiry_month__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryMonth"));
                            }
                            expiry_month__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExpiryYear => {
                            if expiry_year__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryYear"));
                            }
                            expiry_year__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CardHolderEmail => {
                            if card_holder_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cardHolderEmail"));
                            }
                            card_holder_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CardHolderName => {
                            if card_holder_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cardHolderName"));
                            }
                            card_holder_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetPaymentMethodIntent {
                    number: number__.unwrap_or_default(),
                    cvv: cvv__.unwrap_or_default(),
                    expiry_month: expiry_month__.unwrap_or_default(),
                    expiry_year: expiry_year__.unwrap_or_default(),
                    card_holder_email: card_holder_email__.unwrap_or_default(),
                    card_holder_name: card_holder_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SetPaymentMethodIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetPaymentMethodIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SetPaymentMethodIntentV2", len)?;
        if true {
            struct_ser.serialize_field("paymentMethodId", &self.payment_method_id)?;
        }
        if true {
            struct_ser.serialize_field("cardHolderEmail", &self.card_holder_email)?;
        }
        if true {
            struct_ser.serialize_field("cardHolderName", &self.card_holder_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetPaymentMethodIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payment_method_id",
            "paymentMethodId",
            "card_holder_email",
            "cardHolderEmail",
            "card_holder_name",
            "cardHolderName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PaymentMethodId,
            CardHolderEmail,
            CardHolderName,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "paymentMethodId" | "payment_method_id" => Ok(GeneratedField::PaymentMethodId),
                            "cardHolderEmail" | "card_holder_email" => Ok(GeneratedField::CardHolderEmail),
                            "cardHolderName" | "card_holder_name" => Ok(GeneratedField::CardHolderName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetPaymentMethodIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SetPaymentMethodIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetPaymentMethodIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payment_method_id__ = None;
                let mut card_holder_email__ = None;
                let mut card_holder_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PaymentMethodId => {
                            if payment_method_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("paymentMethodId"));
                            }
                            payment_method_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CardHolderEmail => {
                            if card_holder_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cardHolderEmail"));
                            }
                            card_holder_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CardHolderName => {
                            if card_holder_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cardHolderName"));
                            }
                            card_holder_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetPaymentMethodIntentV2 {
                    payment_method_id: payment_method_id__.unwrap_or_default(),
                    card_holder_email: card_holder_email__.unwrap_or_default(),
                    card_holder_name: card_holder_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SetPaymentMethodIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetPaymentMethodResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SetPaymentMethodResult", len)?;
        if true {
            struct_ser.serialize_field("lastFour", &self.last_four)?;
        }
        if true {
            struct_ser.serialize_field("cardHolderName", &self.card_holder_name)?;
        }
        if true {
            struct_ser.serialize_field("cardHolderEmail", &self.card_holder_email)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetPaymentMethodResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "last_four",
            "lastFour",
            "card_holder_name",
            "cardHolderName",
            "card_holder_email",
            "cardHolderEmail",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LastFour,
            CardHolderName,
            CardHolderEmail,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "lastFour" | "last_four" => Ok(GeneratedField::LastFour),
                            "cardHolderName" | "card_holder_name" => Ok(GeneratedField::CardHolderName),
                            "cardHolderEmail" | "card_holder_email" => Ok(GeneratedField::CardHolderEmail),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetPaymentMethodResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SetPaymentMethodResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetPaymentMethodResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut last_four__ = None;
                let mut card_holder_name__ = None;
                let mut card_holder_email__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::LastFour => {
                            if last_four__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lastFour"));
                            }
                            last_four__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CardHolderName => {
                            if card_holder_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cardHolderName"));
                            }
                            card_holder_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CardHolderEmail => {
                            if card_holder_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cardHolderEmail"));
                            }
                            card_holder_email__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetPaymentMethodResult {
                    last_four: last_four__.unwrap_or_default(),
                    card_holder_name: card_holder_name__.unwrap_or_default(),
                    card_holder_email: card_holder_email__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SetPaymentMethodResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignRawPayloadIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignRawPayloadIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("payload", &self.payload)?;
        }
        if true {
            let v = super::super::common::v1::PayloadEncoding::try_from(self.encoding)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.encoding)))?;
            struct_ser.serialize_field("encoding", &v)?;
        }
        if true {
            let v = super::super::common::v1::HashFunction::try_from(self.hash_function)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.hash_function)))?;
            struct_ser.serialize_field("hashFunction", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignRawPayloadIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "payload",
            "encoding",
            "hash_function",
            "hashFunction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            Payload,
            Encoding,
            HashFunction,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "payload" => Ok(GeneratedField::Payload),
                            "encoding" => Ok(GeneratedField::Encoding),
                            "hashFunction" | "hash_function" => Ok(GeneratedField::HashFunction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignRawPayloadIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignRawPayloadIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignRawPayloadIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut payload__ = None;
                let mut encoding__ = None;
                let mut hash_function__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Encoding => {
                            if encoding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encoding"));
                            }
                            encoding__ = Some(map_.next_value::<super::super::common::v1::PayloadEncoding>()? as i32);
                        }
                        GeneratedField::HashFunction => {
                            if hash_function__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashFunction"));
                            }
                            hash_function__ = Some(map_.next_value::<super::super::common::v1::HashFunction>()? as i32);
                        }
                    }
                }
                Ok(SignRawPayloadIntent {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    payload: payload__.unwrap_or_default(),
                    encoding: encoding__.unwrap_or_default(),
                    hash_function: hash_function__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignRawPayloadIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignRawPayloadIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignRawPayloadIntentV2", len)?;
        if true {
            struct_ser.serialize_field("signWith", &self.sign_with)?;
        }
        if true {
            struct_ser.serialize_field("payload", &self.payload)?;
        }
        if true {
            let v = super::super::common::v1::PayloadEncoding::try_from(self.encoding)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.encoding)))?;
            struct_ser.serialize_field("encoding", &v)?;
        }
        if true {
            let v = super::super::common::v1::HashFunction::try_from(self.hash_function)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.hash_function)))?;
            struct_ser.serialize_field("hashFunction", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignRawPayloadIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sign_with",
            "signWith",
            "payload",
            "encoding",
            "hash_function",
            "hashFunction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignWith,
            Payload,
            Encoding,
            HashFunction,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "signWith" | "sign_with" => Ok(GeneratedField::SignWith),
                            "payload" => Ok(GeneratedField::Payload),
                            "encoding" => Ok(GeneratedField::Encoding),
                            "hashFunction" | "hash_function" => Ok(GeneratedField::HashFunction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignRawPayloadIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignRawPayloadIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignRawPayloadIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sign_with__ = None;
                let mut payload__ = None;
                let mut encoding__ = None;
                let mut hash_function__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SignWith => {
                            if sign_with__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signWith"));
                            }
                            sign_with__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Payload => {
                            if payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payload"));
                            }
                            payload__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Encoding => {
                            if encoding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encoding"));
                            }
                            encoding__ = Some(map_.next_value::<super::super::common::v1::PayloadEncoding>()? as i32);
                        }
                        GeneratedField::HashFunction => {
                            if hash_function__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashFunction"));
                            }
                            hash_function__ = Some(map_.next_value::<super::super::common::v1::HashFunction>()? as i32);
                        }
                    }
                }
                Ok(SignRawPayloadIntentV2 {
                    sign_with: sign_with__.unwrap_or_default(),
                    payload: payload__.unwrap_or_default(),
                    encoding: encoding__.unwrap_or_default(),
                    hash_function: hash_function__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignRawPayloadIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignRawPayloadResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignRawPayloadResult", len)?;
        if true {
            struct_ser.serialize_field("r", &self.r)?;
        }
        if true {
            struct_ser.serialize_field("s", &self.s)?;
        }
        if true {
            struct_ser.serialize_field("v", &self.v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignRawPayloadResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "r",
            "s",
            "v",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            R,
            S,
            V,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "r" => Ok(GeneratedField::R),
                            "s" => Ok(GeneratedField::S),
                            "v" => Ok(GeneratedField::V),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignRawPayloadResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignRawPayloadResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignRawPayloadResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r__ = None;
                let mut s__ = None;
                let mut v__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::R => {
                            if r__.is_some() {
                                return Err(serde::de::Error::duplicate_field("r"));
                            }
                            r__ = Some(map_.next_value()?);
                        }
                        GeneratedField::S => {
                            if s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("s"));
                            }
                            s__ = Some(map_.next_value()?);
                        }
                        GeneratedField::V => {
                            if v__.is_some() {
                                return Err(serde::de::Error::duplicate_field("v"));
                            }
                            v__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SignRawPayloadResult {
                    r: r__.unwrap_or_default(),
                    s: s__.unwrap_or_default(),
                    v: v__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignRawPayloadResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignRawPayloadsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignRawPayloadsIntent", len)?;
        if true {
            struct_ser.serialize_field("signWith", &self.sign_with)?;
        }
        if true {
            struct_ser.serialize_field("payloads", &self.payloads)?;
        }
        if true {
            let v = super::super::common::v1::PayloadEncoding::try_from(self.encoding)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.encoding)))?;
            struct_ser.serialize_field("encoding", &v)?;
        }
        if true {
            let v = super::super::common::v1::HashFunction::try_from(self.hash_function)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.hash_function)))?;
            struct_ser.serialize_field("hashFunction", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignRawPayloadsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sign_with",
            "signWith",
            "payloads",
            "encoding",
            "hash_function",
            "hashFunction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignWith,
            Payloads,
            Encoding,
            HashFunction,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "signWith" | "sign_with" => Ok(GeneratedField::SignWith),
                            "payloads" => Ok(GeneratedField::Payloads),
                            "encoding" => Ok(GeneratedField::Encoding),
                            "hashFunction" | "hash_function" => Ok(GeneratedField::HashFunction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignRawPayloadsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignRawPayloadsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignRawPayloadsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sign_with__ = None;
                let mut payloads__ = None;
                let mut encoding__ = None;
                let mut hash_function__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SignWith => {
                            if sign_with__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signWith"));
                            }
                            sign_with__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Payloads => {
                            if payloads__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloads"));
                            }
                            payloads__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Encoding => {
                            if encoding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encoding"));
                            }
                            encoding__ = Some(map_.next_value::<super::super::common::v1::PayloadEncoding>()? as i32);
                        }
                        GeneratedField::HashFunction => {
                            if hash_function__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hashFunction"));
                            }
                            hash_function__ = Some(map_.next_value::<super::super::common::v1::HashFunction>()? as i32);
                        }
                    }
                }
                Ok(SignRawPayloadsIntent {
                    sign_with: sign_with__.unwrap_or_default(),
                    payloads: payloads__.unwrap_or_default(),
                    encoding: encoding__.unwrap_or_default(),
                    hash_function: hash_function__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignRawPayloadsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignRawPayloadsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignRawPayloadsResult", len)?;
        if true {
            struct_ser.serialize_field("signatures", &self.signatures)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignRawPayloadsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signatures",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Signatures,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "signatures" => Ok(GeneratedField::Signatures),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignRawPayloadsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignRawPayloadsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignRawPayloadsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signatures__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Signatures => {
                            if signatures__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signatures"));
                            }
                            signatures__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SignRawPayloadsResult {
                    signatures: signatures__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignRawPayloadsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignTransactionIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignTransactionIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("unsignedTransaction", &self.unsigned_transaction)?;
        }
        if true {
            let v = super::super::common::v1::TransactionType::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignTransactionIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "unsigned_transaction",
            "unsignedTransaction",
            "type",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            UnsignedTransaction,
            Type,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "unsignedTransaction" | "unsigned_transaction" => Ok(GeneratedField::UnsignedTransaction),
                            "type" => Ok(GeneratedField::Type),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignTransactionIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignTransactionIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignTransactionIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut unsigned_transaction__ = None;
                let mut r#type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UnsignedTransaction => {
                            if unsigned_transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unsignedTransaction"));
                            }
                            unsigned_transaction__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<super::super::common::v1::TransactionType>()? as i32);
                        }
                    }
                }
                Ok(SignTransactionIntent {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    unsigned_transaction: unsigned_transaction__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignTransactionIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignTransactionIntentV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignTransactionIntentV2", len)?;
        if true {
            struct_ser.serialize_field("signWith", &self.sign_with)?;
        }
        if true {
            struct_ser.serialize_field("unsignedTransaction", &self.unsigned_transaction)?;
        }
        if true {
            let v = super::super::common::v1::TransactionType::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignTransactionIntentV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sign_with",
            "signWith",
            "unsigned_transaction",
            "unsignedTransaction",
            "type",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignWith,
            UnsignedTransaction,
            Type,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "signWith" | "sign_with" => Ok(GeneratedField::SignWith),
                            "unsignedTransaction" | "unsigned_transaction" => Ok(GeneratedField::UnsignedTransaction),
                            "type" => Ok(GeneratedField::Type),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignTransactionIntentV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignTransactionIntentV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignTransactionIntentV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sign_with__ = None;
                let mut unsigned_transaction__ = None;
                let mut r#type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SignWith => {
                            if sign_with__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signWith"));
                            }
                            sign_with__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UnsignedTransaction => {
                            if unsigned_transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unsignedTransaction"));
                            }
                            unsigned_transaction__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<super::super::common::v1::TransactionType>()? as i32);
                        }
                    }
                }
                Ok(SignTransactionIntentV2 {
                    sign_with: sign_with__.unwrap_or_default(),
                    unsigned_transaction: unsigned_transaction__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignTransactionIntentV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignTransactionResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.SignTransactionResult", len)?;
        if true {
            struct_ser.serialize_field("signedTransaction", &self.signed_transaction)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignTransactionResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "signed_transaction",
            "signedTransaction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignedTransaction,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "signedTransaction" | "signed_transaction" => Ok(GeneratedField::SignedTransaction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignTransactionResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.SignTransactionResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SignTransactionResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut signed_transaction__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SignedTransaction => {
                            if signed_transaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedTransaction"));
                            }
                            signed_transaction__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SignTransactionResult {
                    signed_transaction: signed_transaction__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.SignTransactionResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateAllowedOriginsIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateAllowedOriginsIntent", len)?;
        if true {
            struct_ser.serialize_field("allowedOrigins", &self.allowed_origins)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateAllowedOriginsIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "allowed_origins",
            "allowedOrigins",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AllowedOrigins,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "allowedOrigins" | "allowed_origins" => Ok(GeneratedField::AllowedOrigins),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateAllowedOriginsIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateAllowedOriginsIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateAllowedOriginsIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut allowed_origins__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AllowedOrigins => {
                            if allowed_origins__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowedOrigins"));
                            }
                            allowed_origins__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdateAllowedOriginsIntent {
                    allowed_origins: allowed_origins__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateAllowedOriginsIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateAllowedOriginsResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateAllowedOriginsResult", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateAllowedOriginsResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateAllowedOriginsResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateAllowedOriginsResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateAllowedOriginsResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(UpdateAllowedOriginsResult {
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateAllowedOriginsResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePolicyIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdatePolicyIntent", len)?;
        if true {
            struct_ser.serialize_field("policyId", &self.policy_id)?;
        }
        if let Some(v) = self.policy_name.as_ref() {
            struct_ser.serialize_field("policyName", v)?;
        }
        if let Some(v) = self.policy_effect.as_ref() {
            let v = super::super::common::v1::Effect::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("policyEffect", &v)?;
        }
        if let Some(v) = self.policy_condition.as_ref() {
            struct_ser.serialize_field("policyCondition", v)?;
        }
        if let Some(v) = self.policy_consensus.as_ref() {
            struct_ser.serialize_field("policyConsensus", v)?;
        }
        if let Some(v) = self.policy_notes.as_ref() {
            struct_ser.serialize_field("policyNotes", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePolicyIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_id",
            "policyId",
            "policy_name",
            "policyName",
            "policy_effect",
            "policyEffect",
            "policy_condition",
            "policyCondition",
            "policy_consensus",
            "policyConsensus",
            "policy_notes",
            "policyNotes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyId,
            PolicyName,
            PolicyEffect,
            PolicyCondition,
            PolicyConsensus,
            PolicyNotes,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyId" | "policy_id" => Ok(GeneratedField::PolicyId),
                            "policyName" | "policy_name" => Ok(GeneratedField::PolicyName),
                            "policyEffect" | "policy_effect" => Ok(GeneratedField::PolicyEffect),
                            "policyCondition" | "policy_condition" => Ok(GeneratedField::PolicyCondition),
                            "policyConsensus" | "policy_consensus" => Ok(GeneratedField::PolicyConsensus),
                            "policyNotes" | "policy_notes" => Ok(GeneratedField::PolicyNotes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdatePolicyIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdatePolicyIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePolicyIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_id__ = None;
                let mut policy_name__ = None;
                let mut policy_effect__ = None;
                let mut policy_condition__ = None;
                let mut policy_consensus__ = None;
                let mut policy_notes__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyId => {
                            if policy_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyId"));
                            }
                            policy_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PolicyName => {
                            if policy_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyName"));
                            }
                            policy_name__ = map_.next_value()?;
                        }
                        GeneratedField::PolicyEffect => {
                            if policy_effect__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyEffect"));
                            }
                            policy_effect__ = map_.next_value::<::std::option::Option<super::super::common::v1::Effect>>()?.map(|x| x as i32);
                        }
                        GeneratedField::PolicyCondition => {
                            if policy_condition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyCondition"));
                            }
                            policy_condition__ = map_.next_value()?;
                        }
                        GeneratedField::PolicyConsensus => {
                            if policy_consensus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyConsensus"));
                            }
                            policy_consensus__ = map_.next_value()?;
                        }
                        GeneratedField::PolicyNotes => {
                            if policy_notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyNotes"));
                            }
                            policy_notes__ = map_.next_value()?;
                        }
                    }
                }
                Ok(UpdatePolicyIntent {
                    policy_id: policy_id__.unwrap_or_default(),
                    policy_name: policy_name__,
                    policy_effect: policy_effect__,
                    policy_condition: policy_condition__,
                    policy_consensus: policy_consensus__,
                    policy_notes: policy_notes__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdatePolicyIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePolicyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdatePolicyResult", len)?;
        if true {
            struct_ser.serialize_field("policyId", &self.policy_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePolicyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_id",
            "policyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "policyId" | "policy_id" => Ok(GeneratedField::PolicyId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdatePolicyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdatePolicyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePolicyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyId => {
                            if policy_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyId"));
                            }
                            policy_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdatePolicyResult {
                    policy_id: policy_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdatePolicyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePrivateKeyTagIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdatePrivateKeyTagIntent", len)?;
        if true {
            struct_ser.serialize_field("privateKeyTagId", &self.private_key_tag_id)?;
        }
        if let Some(v) = self.new_private_key_tag_name.as_ref() {
            struct_ser.serialize_field("newPrivateKeyTagName", v)?;
        }
        if true {
            struct_ser.serialize_field("addPrivateKeyIds", &self.add_private_key_ids)?;
        }
        if true {
            struct_ser.serialize_field("removePrivateKeyIds", &self.remove_private_key_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePrivateKeyTagIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_tag_id",
            "privateKeyTagId",
            "new_private_key_tag_name",
            "newPrivateKeyTagName",
            "add_private_key_ids",
            "addPrivateKeyIds",
            "remove_private_key_ids",
            "removePrivateKeyIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyTagId,
            NewPrivateKeyTagName,
            AddPrivateKeyIds,
            RemovePrivateKeyIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyTagId" | "private_key_tag_id" => Ok(GeneratedField::PrivateKeyTagId),
                            "newPrivateKeyTagName" | "new_private_key_tag_name" => Ok(GeneratedField::NewPrivateKeyTagName),
                            "addPrivateKeyIds" | "add_private_key_ids" => Ok(GeneratedField::AddPrivateKeyIds),
                            "removePrivateKeyIds" | "remove_private_key_ids" => Ok(GeneratedField::RemovePrivateKeyIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdatePrivateKeyTagIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdatePrivateKeyTagIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePrivateKeyTagIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_tag_id__ = None;
                let mut new_private_key_tag_name__ = None;
                let mut add_private_key_ids__ = None;
                let mut remove_private_key_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyTagId => {
                            if private_key_tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTagId"));
                            }
                            private_key_tag_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NewPrivateKeyTagName => {
                            if new_private_key_tag_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newPrivateKeyTagName"));
                            }
                            new_private_key_tag_name__ = map_.next_value()?;
                        }
                        GeneratedField::AddPrivateKeyIds => {
                            if add_private_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addPrivateKeyIds"));
                            }
                            add_private_key_ids__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RemovePrivateKeyIds => {
                            if remove_private_key_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("removePrivateKeyIds"));
                            }
                            remove_private_key_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdatePrivateKeyTagIntent {
                    private_key_tag_id: private_key_tag_id__.unwrap_or_default(),
                    new_private_key_tag_name: new_private_key_tag_name__,
                    add_private_key_ids: add_private_key_ids__.unwrap_or_default(),
                    remove_private_key_ids: remove_private_key_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdatePrivateKeyTagIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePrivateKeyTagResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdatePrivateKeyTagResult", len)?;
        if true {
            struct_ser.serialize_field("privateKeyTagId", &self.private_key_tag_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePrivateKeyTagResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_tag_id",
            "privateKeyTagId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyTagId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "privateKeyTagId" | "private_key_tag_id" => Ok(GeneratedField::PrivateKeyTagId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdatePrivateKeyTagResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdatePrivateKeyTagResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePrivateKeyTagResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_tag_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyTagId => {
                            if private_key_tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTagId"));
                            }
                            private_key_tag_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdatePrivateKeyTagResult {
                    private_key_tag_id: private_key_tag_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdatePrivateKeyTagResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateRootQuorumIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateRootQuorumIntent", len)?;
        if true {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateRootQuorumIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "threshold",
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Threshold,
            UserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "threshold" => Ok(GeneratedField::Threshold),
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateRootQuorumIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateRootQuorumIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateRootQuorumIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut threshold__ = None;
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdateRootQuorumIntent {
                    threshold: threshold__.unwrap_or_default(),
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateRootQuorumIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateRootQuorumResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateRootQuorumResult", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateRootQuorumResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateRootQuorumResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateRootQuorumResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateRootQuorumResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(UpdateRootQuorumResult {
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateRootQuorumResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateUserIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateUserIntent", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if let Some(v) = self.user_name.as_ref() {
            struct_ser.serialize_field("userName", v)?;
        }
        if let Some(v) = self.user_email.as_ref() {
            struct_ser.serialize_field("userEmail", v)?;
        }
        if true {
            struct_ser.serialize_field("userTagIds", &self.user_tag_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateUserIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "user_name",
            "userName",
            "user_email",
            "userEmail",
            "user_tag_ids",
            "userTagIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            UserName,
            UserEmail,
            UserTagIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "userName" | "user_name" => Ok(GeneratedField::UserName),
                            "userEmail" | "user_email" => Ok(GeneratedField::UserEmail),
                            "userTagIds" | "user_tag_ids" => Ok(GeneratedField::UserTagIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateUserIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateUserIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateUserIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut user_name__ = None;
                let mut user_email__ = None;
                let mut user_tag_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserName => {
                            if user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userName"));
                            }
                            user_name__ = map_.next_value()?;
                        }
                        GeneratedField::UserEmail => {
                            if user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userEmail"));
                            }
                            user_email__ = map_.next_value()?;
                        }
                        GeneratedField::UserTagIds => {
                            if user_tag_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagIds"));
                            }
                            user_tag_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdateUserIntent {
                    user_id: user_id__.unwrap_or_default(),
                    user_name: user_name__,
                    user_email: user_email__,
                    user_tag_ids: user_tag_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateUserIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateUserResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateUserResult", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateUserResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateUserResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateUserResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateUserResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdateUserResult {
                    user_id: user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateUserResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateUserTagIntent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateUserTagIntent", len)?;
        if true {
            struct_ser.serialize_field("userTagId", &self.user_tag_id)?;
        }
        if let Some(v) = self.new_user_tag_name.as_ref() {
            struct_ser.serialize_field("newUserTagName", v)?;
        }
        if true {
            struct_ser.serialize_field("addUserIds", &self.add_user_ids)?;
        }
        if true {
            struct_ser.serialize_field("removeUserIds", &self.remove_user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateUserTagIntent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tag_id",
            "userTagId",
            "new_user_tag_name",
            "newUserTagName",
            "add_user_ids",
            "addUserIds",
            "remove_user_ids",
            "removeUserIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTagId,
            NewUserTagName,
            AddUserIds,
            RemoveUserIds,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userTagId" | "user_tag_id" => Ok(GeneratedField::UserTagId),
                            "newUserTagName" | "new_user_tag_name" => Ok(GeneratedField::NewUserTagName),
                            "addUserIds" | "add_user_ids" => Ok(GeneratedField::AddUserIds),
                            "removeUserIds" | "remove_user_ids" => Ok(GeneratedField::RemoveUserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateUserTagIntent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateUserTagIntent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateUserTagIntent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tag_id__ = None;
                let mut new_user_tag_name__ = None;
                let mut add_user_ids__ = None;
                let mut remove_user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTagId => {
                            if user_tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagId"));
                            }
                            user_tag_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NewUserTagName => {
                            if new_user_tag_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newUserTagName"));
                            }
                            new_user_tag_name__ = map_.next_value()?;
                        }
                        GeneratedField::AddUserIds => {
                            if add_user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addUserIds"));
                            }
                            add_user_ids__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RemoveUserIds => {
                            if remove_user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("removeUserIds"));
                            }
                            remove_user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdateUserTagIntent {
                    user_tag_id: user_tag_id__.unwrap_or_default(),
                    new_user_tag_name: new_user_tag_name__,
                    add_user_ids: add_user_ids__.unwrap_or_default(),
                    remove_user_ids: remove_user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateUserTagIntent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateUserTagResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UpdateUserTagResult", len)?;
        if true {
            struct_ser.serialize_field("userTagId", &self.user_tag_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateUserTagResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_tag_id",
            "userTagId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserTagId,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userTagId" | "user_tag_id" => Ok(GeneratedField::UserTagId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateUserTagResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UpdateUserTagResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateUserTagResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_tag_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserTagId => {
                            if user_tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTagId"));
                            }
                            user_tag_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UpdateUserTagResult {
                    user_tag_id: user_tag_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UpdateUserTagResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UserParams", len)?;
        if true {
            struct_ser.serialize_field("userName", &self.user_name)?;
        }
        if let Some(v) = self.user_email.as_ref() {
            struct_ser.serialize_field("userEmail", v)?;
        }
        if true {
            let v = super::super::common::v1::AccessType::try_from(self.access_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.access_type)))?;
            struct_ser.serialize_field("accessType", &v)?;
        }
        if true {
            struct_ser.serialize_field("apiKeys", &self.api_keys)?;
        }
        if true {
            struct_ser.serialize_field("authenticators", &self.authenticators)?;
        }
        if true {
            struct_ser.serialize_field("userTags", &self.user_tags)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_name",
            "userName",
            "user_email",
            "userEmail",
            "access_type",
            "accessType",
            "api_keys",
            "apiKeys",
            "authenticators",
            "user_tags",
            "userTags",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserName,
            UserEmail,
            AccessType,
            ApiKeys,
            Authenticators,
            UserTags,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userName" | "user_name" => Ok(GeneratedField::UserName),
                            "userEmail" | "user_email" => Ok(GeneratedField::UserEmail),
                            "accessType" | "access_type" => Ok(GeneratedField::AccessType),
                            "apiKeys" | "api_keys" => Ok(GeneratedField::ApiKeys),
                            "authenticators" => Ok(GeneratedField::Authenticators),
                            "userTags" | "user_tags" => Ok(GeneratedField::UserTags),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UserParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_name__ = None;
                let mut user_email__ = None;
                let mut access_type__ = None;
                let mut api_keys__ = None;
                let mut authenticators__ = None;
                let mut user_tags__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserName => {
                            if user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userName"));
                            }
                            user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserEmail => {
                            if user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userEmail"));
                            }
                            user_email__ = map_.next_value()?;
                        }
                        GeneratedField::AccessType => {
                            if access_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessType"));
                            }
                            access_type__ = Some(map_.next_value::<super::super::common::v1::AccessType>()? as i32);
                        }
                        GeneratedField::ApiKeys => {
                            if api_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeys"));
                            }
                            api_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Authenticators => {
                            if authenticators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticators"));
                            }
                            authenticators__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserTags => {
                            if user_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTags"));
                            }
                            user_tags__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UserParams {
                    user_name: user_name__.unwrap_or_default(),
                    user_email: user_email__,
                    access_type: access_type__.unwrap_or_default(),
                    api_keys: api_keys__.unwrap_or_default(),
                    authenticators: authenticators__.unwrap_or_default(),
                    user_tags: user_tags__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UserParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserParamsV2 {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.UserParamsV2", len)?;
        if true {
            struct_ser.serialize_field("userName", &self.user_name)?;
        }
        if let Some(v) = self.user_email.as_ref() {
            struct_ser.serialize_field("userEmail", v)?;
        }
        if true {
            struct_ser.serialize_field("apiKeys", &self.api_keys)?;
        }
        if true {
            struct_ser.serialize_field("authenticators", &self.authenticators)?;
        }
        if true {
            struct_ser.serialize_field("userTags", &self.user_tags)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserParamsV2 {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_name",
            "userName",
            "user_email",
            "userEmail",
            "api_keys",
            "apiKeys",
            "authenticators",
            "user_tags",
            "userTags",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserName,
            UserEmail,
            ApiKeys,
            Authenticators,
            UserTags,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userName" | "user_name" => Ok(GeneratedField::UserName),
                            "userEmail" | "user_email" => Ok(GeneratedField::UserEmail),
                            "apiKeys" | "api_keys" => Ok(GeneratedField::ApiKeys),
                            "authenticators" => Ok(GeneratedField::Authenticators),
                            "userTags" | "user_tags" => Ok(GeneratedField::UserTags),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserParamsV2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.UserParamsV2")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserParamsV2, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_name__ = None;
                let mut user_email__ = None;
                let mut api_keys__ = None;
                let mut authenticators__ = None;
                let mut user_tags__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserName => {
                            if user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userName"));
                            }
                            user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserEmail => {
                            if user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userEmail"));
                            }
                            user_email__ = map_.next_value()?;
                        }
                        GeneratedField::ApiKeys => {
                            if api_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeys"));
                            }
                            api_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Authenticators => {
                            if authenticators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticators"));
                            }
                            authenticators__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserTags => {
                            if user_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTags"));
                            }
                            user_tags__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(UserParamsV2 {
                    user_name: user_name__.unwrap_or_default(),
                    user_email: user_email__,
                    api_keys: api_keys__.unwrap_or_default(),
                    authenticators: authenticators__.unwrap_or_default(),
                    user_tags: user_tags__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.UserParamsV2", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WalletAccountParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.WalletAccountParams", len)?;
        if true {
            let v = super::super::common::v1::Curve::try_from(self.curve)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.curve)))?;
            struct_ser.serialize_field("curve", &v)?;
        }
        if true {
            let v = super::super::common::v1::PathFormat::try_from(self.path_format)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.path_format)))?;
            struct_ser.serialize_field("pathFormat", &v)?;
        }
        if true {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if true {
            let v = super::super::common::v1::AddressFormat::try_from(self.address_format)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.address_format)))?;
            struct_ser.serialize_field("addressFormat", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WalletAccountParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "curve",
            "path_format",
            "pathFormat",
            "path",
            "address_format",
            "addressFormat",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Curve,
            PathFormat,
            Path,
            AddressFormat,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "curve" => Ok(GeneratedField::Curve),
                            "pathFormat" | "path_format" => Ok(GeneratedField::PathFormat),
                            "path" => Ok(GeneratedField::Path),
                            "addressFormat" | "address_format" => Ok(GeneratedField::AddressFormat),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WalletAccountParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.WalletAccountParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WalletAccountParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut curve__ = None;
                let mut path_format__ = None;
                let mut path__ = None;
                let mut address_format__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Curve => {
                            if curve__.is_some() {
                                return Err(serde::de::Error::duplicate_field("curve"));
                            }
                            curve__ = Some(map_.next_value::<super::super::common::v1::Curve>()? as i32);
                        }
                        GeneratedField::PathFormat => {
                            if path_format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pathFormat"));
                            }
                            path_format__ = Some(map_.next_value::<super::super::common::v1::PathFormat>()? as i32);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AddressFormat => {
                            if address_format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressFormat"));
                            }
                            address_format__ = Some(map_.next_value::<super::super::common::v1::AddressFormat>()? as i32);
                        }
                    }
                }
                Ok(WalletAccountParams {
                    curve: curve__.unwrap_or_default(),
                    path_format: path_format__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    address_format: address_format__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.WalletAccountParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WalletParams {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.WalletParams", len)?;
        if true {
            struct_ser.serialize_field("walletName", &self.wallet_name)?;
        }
        if true {
            struct_ser.serialize_field("accounts", &self.accounts)?;
        }
        if let Some(v) = self.mnemonic_length.as_ref() {
            struct_ser.serialize_field("mnemonicLength", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WalletParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_name",
            "walletName",
            "accounts",
            "mnemonic_length",
            "mnemonicLength",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletName,
            Accounts,
            MnemonicLength,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletName" | "wallet_name" => Ok(GeneratedField::WalletName),
                            "accounts" => Ok(GeneratedField::Accounts),
                            "mnemonicLength" | "mnemonic_length" => Ok(GeneratedField::MnemonicLength),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WalletParams;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.WalletParams")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WalletParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_name__ = None;
                let mut accounts__ = None;
                let mut mnemonic_length__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletName => {
                            if wallet_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletName"));
                            }
                            wallet_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Accounts => {
                            if accounts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accounts"));
                            }
                            accounts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MnemonicLength => {
                            if mnemonic_length__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mnemonicLength"));
                            }
                            mnemonic_length__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(WalletParams {
                    wallet_name: wallet_name__.unwrap_or_default(),
                    accounts: accounts__.unwrap_or_default(),
                    mnemonic_length: mnemonic_length__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.WalletParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WalletResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.activity.v1.WalletResult", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WalletResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "addresses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            Addresses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "addresses" => Ok(GeneratedField::Addresses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WalletResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.activity.v1.WalletResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WalletResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut addresses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(WalletResult {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    addresses: addresses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.activity.v1.WalletResult", FIELDS, GeneratedVisitor)
    }
}
