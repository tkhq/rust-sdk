// Unsure why prost-build doesn't generate file structure to match the module
// structure - hence this manual cruft

pub mod immutable {
    pub mod common {
        pub mod v1 {
            include!("immutable.common.v1.rs");
            include!("immutable.common.v1.serde.rs");
        }
    }
    pub mod activity {
        pub mod v1 {
            include!("immutable.activity.v1.rs");
            include!("immutable.activity.v1.serde.rs");
        }
    }
    pub mod data {
        pub mod v1 {
            include!("immutable.data.v1.rs");
            include!("immutable.data.v1.serde.rs");
        }
    }
    pub mod webauthn {
        pub mod v1 {
            include!("immutable.webauthn.v1.rs");
            include!("immutable.webauthn.v1.serde.rs");
        }
    }
}

pub mod external {
    pub mod options {
        pub mod v1 {
            include!("external.options.v1.rs");
            include!("external.options.v1.serde.rs");
        }
    }
    pub mod activity {
        pub mod v1 {
            include!("external.activity.v1.rs");
            include!("external.activity.v1.serde.rs");
        }
    }
    pub mod data {
        pub mod v1 {
            include!("external.data.v1.rs");
            include!("external.data.v1.serde.rs");
        }
    }
    pub mod webauthn {
        pub mod v1 {
            include!("external.webauthn.v1.rs");
            include!("external.webauthn.v1.serde.rs");
        }
    }
}

pub mod google {
    pub mod api {
        include!("google.api.rs");
        include!("google.api.serde.rs");
    }
    pub mod rpc {
        include!("google.rpc.rs");
        include!("google.rpc.serde.rs");
    }
}

pub mod services {
    pub mod coordinator {
        pub mod public {
            pub mod v1 {
                include!("services.coordinator.public.v1.rs");
                include!("services.coordinator.public.v1.serde.rs");
            }
        }
    }
}
