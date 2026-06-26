//! Deploy commands.

use crate::config::deploy::DeployConfig;

pub mod approve;
pub mod create;
pub mod debug_logs;
pub mod delete;
pub mod get_status;
pub mod init;
pub mod post_share;
pub mod provisioning_details;
pub mod restore;
pub mod status;

pub(crate) const PORT_GUIDANCE: &str = r#"Port guidance:
  publicIngressPort is the port inside your container that receives public app
  traffic.
  healthCheckPort is the port TVC probes to decide whether the deployment is
  healthy.
  Use the same port for both unless your binary exposes health checks on a
  separate listener. Generated configs default both to 3000."#;

pub(crate) fn format_port_summary(config: &DeployConfig) -> String {
    let public_port = config.public_ingress_port;
    let health_port = config.health_check_port;

    if public_port == health_port {
        return format!(
            r#"Ports:
  Public ingress and health checks both use container port {public_port}.
  This is the common case when one listener serves app traffic and health checks."#
        );
    }

    format!(
        r#"Ports:
  Public ingress uses container port {public_port}.
  Health checks use container port {health_port}.
  Separate ports are for binaries that expose health checks on a separate listener."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_summary_explains_shared_port() {
        let config = DeployConfig::template(None);

        let summary = format_port_summary(&config);

        assert!(
            summary.contains("both use container port 3000"),
            "{summary}"
        );
        assert!(summary.contains("common case"), "{summary}");
    }

    #[test]
    fn port_summary_explains_split_ports() {
        let mut config = DeployConfig::template(None);
        config.public_ingress_port = 8080;
        config.health_check_port = 9090;

        let summary = format_port_summary(&config);

        assert!(
            summary.contains("Public ingress uses container port 8080"),
            "{summary}"
        );
        assert!(
            summary.contains("Health checks use container port 9090"),
            "{summary}"
        );
        assert!(summary.contains("separate listener"), "{summary}");
    }
}
