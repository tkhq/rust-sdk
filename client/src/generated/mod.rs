pub mod external {
    pub mod activity {
        pub mod v1 {
            include!("external.activity.v1.rs");
        }
    }
    pub mod data {
        pub mod v1 {
            include!("external.data.v1.rs");
        }
    }
    pub mod options {
        pub mod v1 {
            include!("external.options.v1.rs");
        }
    }
    pub mod webauthn {
        pub mod v1 {
            include!("external.webauthn.v1.rs");
        }
    }
}
pub mod google {
    pub mod api {
        include!("google.api.rs");
    }
    pub mod rpc {
        include!("google.rpc.rs");
    }
}
pub mod grpc {
    pub mod gateway {
        pub mod protoc_gen_openapiv2 {
            pub mod options {
                include!("grpc.gateway.protoc_gen_openapiv2.options.rs");
            }
        }
    }
}
pub mod immutable {
    pub mod activity {
        pub mod api {
            include!("immutable.activity.api.rs");
        }
        pub mod billing {
            include!("immutable.activity.billing.rs");
        }
        pub mod v1 {
            include!("immutable.activity.v1.rs");
        }
    }
    pub mod common {
        pub mod v1 {
            include!("immutable.common.v1.rs");
        }
    }
    pub mod data {
        pub mod v1 {
            include!("immutable.data.v1.rs");
        }
    }
    pub mod webauthn {
        pub mod v1 {
            include!("immutable.webauthn.v1.rs");
        }
    }
}
pub mod services {
    pub mod coordinator {
        pub mod public {
            pub mod v1 {
                include!("services.coordinator.public.v1.rs");
            }
        }
    }
}
mod client;
pub use services::coordinator::public::v1::*;
pub use external::activity::v1::*;
pub use immutable::activity::v1::*;
