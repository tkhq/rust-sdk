/// A known Turnkey API environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnkeyEnvironment {
    Production,
    Preproduction,
    Development,
    Local,
}

impl TurnkeyEnvironment {
    /// Recognizes only the exact API base URL of a known environment.
    pub fn from_api_base_url(api_base_url: &str) -> Option<Self> {
        [
            Self::Production,
            Self::Preproduction,
            Self::Development,
            Self::Local,
        ]
        .into_iter()
        .find(|environment| environment.api_base_url() == api_base_url)
    }

    /// Returns the canonical API base URL.
    pub const fn api_base_url(self) -> &'static str {
        match self {
            Self::Production => "https://api.turnkey.com",
            Self::Preproduction => "https://api.preprod.turnkey.engineering",
            Self::Development => "https://api.dev.turnkey.engineering",
            Self::Local => "http://localhost:8081",
        }
    }

    /// Returns the dashboard for this environment. Local uses the production dashboard.
    pub const fn dashboard_base_url(self) -> &'static str {
        match self {
            Self::Production | Self::Local => "https://app.turnkey.com",
            Self::Preproduction => "https://app.preprod.turnkey.engineering",
            Self::Development => "https://app.dev.turnkey.engineering",
        }
    }
}
