#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ActivateBillingTierIntent {
    /// @inject_tag: validate:"required"
    pub product_id: ::prost::alloc::string::String,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct ActivateBillingTierResult {
    pub product_id: ::prost::alloc::string::String,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePaymentMethodIntent {
    #[serde(default)]
    pub payment_method_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct DeletePaymentMethodResult {
    pub payment_method_id: ::prost::alloc::string::String,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetPaymentMethodIntent {
    /// @inject_tag: validate:"required,max=16,numeric"
    pub number: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,max=4,numeric"
    pub cvv: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,numeric,len=2"
    pub expiry_month: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,numeric,len=4"
    pub expiry_year: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,email,tk_email"
    pub card_holder_email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,tk_label_length"
    pub card_holder_name: ::prost::alloc::string::String,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetPaymentMethodIntentV2 {
    /// @inject_tag: validate:"required,max=256"
    pub payment_method_id: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,email,tk_email"
    pub card_holder_email: ::prost::alloc::string::String,
    /// @inject_tag: validate:"required,tk_label_length"
    pub card_holder_name: ::prost::alloc::string::String,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct SetPaymentMethodResult {
    pub last_four: ::prost::alloc::string::String,
    pub card_holder_name: ::prost::alloc::string::String,
    pub card_holder_email: ::prost::alloc::string::String,
}
