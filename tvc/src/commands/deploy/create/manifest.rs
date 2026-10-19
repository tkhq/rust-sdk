//! Client-side construction of a deployment's QOS manifest.
//!
//! When the CLI builds the manifest, the coordinator stores the submitted
//! bytes unchanged and checks only the fields it decides or already knows: the
//! namespace, the pivot, the bridge rules, the operator sets, and the enclave
//! measurements (with an empty `qosCommit`). Those fields are derived here the
//! way the coordinator's own generator derives them. QOS validates everything
//! else, so nothing more is checked here.

use crate::client::AuthenticatedClient;
use crate::config::deploy::DeployConfig;
use crate::errors::MissingResource;
use anyhow::{Context, Result, bail};
use hex::FromHex;
use qos_core::protocol::{
    Hash256,
    services::boot::{
        BridgeConfig, DnsConfig, ManifestBuilder, ManifestSet, Namespace, NitroConfig,
        QuorumMember, RestartPolicy, ShareSet,
    },
};
use std::net::IpAddr;
use turnkey_client::generated::{
    GetTvcManifestGenerationInfoRequest, GetTvcManifestGenerationInfoResponse,
    external::data::v1::{TvcApp, TvcDeployment, TvcOperatorSet},
};

/// Bind address for every server bridge rule.
const ALL_INTERFACES: &str = "0.0.0.0";

/// A manifest built by the CLI, with the nonce it was built for.
#[derive(Debug)]
pub(super) struct ClientManifest {
    pub(super) nonce: u32,
    pub(super) bytes: Vec<u8>,
}

/// Parse the expected pivot digest into the manifest's pivot hash.
pub(super) fn parse_pivot_digest(digest: &str) -> Result<Hash256> {
    Hash256::from_hex(digest).with_context(|| {
        format!(
            "expected pivot digest {digest:?} must be 64 hex characters with no `sha256:` prefix \
             to build the QOS manifest locally"
        )
    })
}

/// Fetch the values the coordinator decides or already knows for the
/// deployment's QOS version and build the manifest from them and the
/// deployment config.
pub(super) async fn generate(
    auth: &AuthenticatedClient,
    deploy_config: &DeployConfig,
    app: TvcApp,
    pivot_hash: Hash256,
) -> Result<ClientManifest> {
    let info = auth
        .client
        .get_tvc_manifest_generation_info(GetTvcManifestGenerationInfoRequest {
            organization_id: auth.org_id.clone(),
            app_id: deploy_config.app_id.clone(),
            qos_version: deploy_config.qos_version.clone(),
        })
        .await
        .with_context(|| {
            format!(
                "failed to fetch TVC manifest generation info for QOS version {}",
                deploy_config.qos_version
            )
        })?;

    let nonce = info.nonce;
    let bytes = build_manifest(app, info, deploy_config, pivot_hash)?;

    Ok(ClientManifest { nonce, bytes })
}

/// Build the v2 manifest and serialize it with QOS canonical JSON.
///
/// Mirrors the coordinator's generator: the pivot always restarts, its
/// environment is empty, the bridge has a server rule for the health check
/// port, one for the public ingress port when it differs, and a transparent
/// client rule when the app has egress, which is also when DNS resolvers apply.
fn build_manifest(
    app: TvcApp,
    info: GetTvcManifestGenerationInfoResponse,
    deploy_config: &DeployConfig,
    pivot_hash: Hash256,
) -> Result<Vec<u8>> {
    let TvcApp {
        id: app_id,
        organization_id: _,
        name: _,
        quorum_public_key,
        manifest_set,
        share_set,
        enable_egress,
        created_at: _,
        updated_at: _,
        live_deployment_id: _,
        public_domain: _,
        enable_debug_mode_deployments: _,
    } = app;
    let GetTvcManifestGenerationInfoResponse {
        nonce,
        namespace,
        pcr0,
        pcr1,
        pcr2,
        pcr3,
        aws_root_certificate,
        default_dns_resolvers,
    } = info;

    let (manifest_threshold, manifest_members) = operator_set(
        manifest_set.ok_or_else(|| MissingResource::new("manifest set", app_id.as_str()))?,
        "manifest set",
    )?;
    let (share_threshold, share_members) = operator_set(
        share_set.ok_or_else(|| MissingResource::new("share set", app_id.as_str()))?,
        "share set",
    )?;

    let server = |port| BridgeConfig::Server {
        port,
        host: ALL_INTERFACES.to_string(),
    };
    let mut bridge_config = vec![server(deploy_config.health_check_port)];
    if deploy_config.public_ingress_port != deploy_config.health_check_port {
        bridge_config.push(server(deploy_config.public_ingress_port));
    }

    let mut builder = ManifestBuilder::v2()
        .namespace(Namespace {
            name: namespace,
            nonce,
            quorum_key: decode_hex("app quorum public key", &quorum_public_key)?,
        })
        .pivot_hash(pivot_hash)
        .restart_policy(RestartPolicy::Always)
        .debug_mode(deploy_config.dangerous_deploy_debug_mode)
        .pivot_args(deploy_config.pivot_args.clone())
        .manifest_set(ManifestSet {
            threshold: manifest_threshold,
            members: manifest_members,
        })
        .share_set(ShareSet {
            threshold: share_threshold,
            members: share_members,
        })
        .enclave(NitroConfig {
            pcr0: decode_hex("pcr0", &pcr0)?,
            pcr1: decode_hex("pcr1", &pcr1)?,
            pcr2: decode_hex("pcr2", &pcr2)?,
            pcr3: decode_hex("pcr3", &pcr3)?,
            aws_root_certificate: decode_hex("AWS root certificate", &aws_root_certificate)?,
            // The coordinator requires this to be empty.
            qos_commit: String::new(),
        });

    if enable_egress {
        bridge_config.push(BridgeConfig::Client {
            port: 0,
            host: None,
        });

        let resolvers = default_dns_resolvers
            .iter()
            .map(|resolver| {
                resolver.parse::<IpAddr>().with_context(|| {
                    format!("DNS resolver {resolver:?} from the coordinator is not an IP address")
                })
            })
            .collect::<Result<Vec<_>>>()?;
        if !resolvers.is_empty() {
            builder = builder.dns(DnsConfig { resolvers });
        }
    }

    let bytes = builder
        .bridge_config(bridge_config)
        .build()
        .context("failed to build the QOS manifest")?
        .to_storage_vec()
        .context("failed to serialize the QOS manifest")?;

    Ok(bytes)
}

/// Convert an operator set to its threshold and quorum members, aliasing each
/// member by operator name.
fn operator_set(set: TvcOperatorSet, set_name: &str) -> Result<(u32, Vec<QuorumMember>)> {
    let TvcOperatorSet {
        id: _,
        name: _,
        organization_id: _,
        operators,
        threshold,
        created_at: _,
        updated_at: _,
    } = set;

    let members = operators
        .into_iter()
        .map(|operator| {
            Ok(QuorumMember {
                pub_key: decode_hex(
                    &format!("{set_name} operator {:?} public key", operator.name),
                    &operator.public_key,
                )?,
                alias: operator.name,
            })
        })
        .collect::<Result<_>>()?;

    Ok((threshold, members))
}

fn decode_hex(field: &str, value: &str) -> Result<Vec<u8>> {
    hex::decode(value).with_context(|| format!("{field} is not valid hex"))
}

/// Check that the coordinator stored the manifest the CLI sent for the
/// deployment, byte for byte.
///
/// A mismatch means the server ignored the client's manifest, typically because
/// an older release predates the intent field that carries it and silently
/// drops that field, so the server generates its own manifest.
pub(super) fn ensure_stored(deployment: TvcDeployment, sent: &[u8]) -> Result<()> {
    let TvcDeployment { id, manifest, .. } = deployment;

    if manifest
        .as_ref()
        .map(|manifest| manifest.manifest.as_slice())
        != Some(sent)
    {
        bail!(
            "deployment {id} was created, but the server did not store the client-generated \
             QOS manifest: the stored manifest differs from the one sent. An older Turnkey API \
             release ignores the client-generated manifest and generates its own. Review the \
             stored manifest before approving this deployment"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use turnkey_client::generated::external::data::v1::TvcOperator;

    const APP_ID: &str = "7d4ea3b5-3f5c-4d7b-9f26-51d4d2c1c7a2";
    const NAMESPACE: &str = "dev/tvc/7d4ea3b5-3f5c-4d7b-9f26-51d4d2c1c7a2";
    const PIVOT_DIGEST: &str = "dba4f54cd573b81d724b0002c4e761c76eb21f881f31d48b972b3e4c101f2098";
    const PCR: &str = "50a89cedc3c4ebd18f55544173b3002013b0fa8009c9558819bfad1fd16ac07ebe0a8d21dffb3360e589db3ff629a5dd";
    const NONCE: u32 = 1_790_000_000;

    fn tvc_operator_set(threshold: u32, operators: &[(&str, &str)]) -> TvcOperatorSet {
        TvcOperatorSet {
            id: "set-id".to_string(),
            name: "set".to_string(),
            organization_id: "org-id".to_string(),
            operators: operators
                .iter()
                .map(|(name, public_key)| TvcOperator {
                    id: format!("{name}-id"),
                    name: name.to_string(),
                    public_key: public_key.to_string(),
                    created_at: None,
                    updated_at: None,
                    encrypt_public_key: String::new(),
                    sign_public_key: String::new(),
                    key_source: None,
                })
                .collect(),
            threshold,
            created_at: None,
            updated_at: None,
        }
    }

    fn app(enable_egress: bool) -> TvcApp {
        TvcApp {
            id: APP_ID.to_string(),
            organization_id: "org-id".to_string(),
            name: "app".to_string(),
            quorum_public_key: "04aa01".to_string(),
            manifest_set: Some(tvc_operator_set(2, &[("alice", "0a0b"), ("bob", "0C0D")])),
            share_set: Some(tvc_operator_set(1, &[("carol", "0e0f")])),
            enable_egress,
            created_at: None,
            updated_at: None,
            live_deployment_id: None,
            public_domain: "app.example".to_string(),
            enable_debug_mode_deployments: false,
        }
    }

    fn info(dns_resolvers: &[&str]) -> GetTvcManifestGenerationInfoResponse {
        GetTvcManifestGenerationInfoResponse {
            nonce: NONCE,
            namespace: NAMESPACE.to_string(),
            pcr0: PCR.to_string(),
            pcr1: PCR.to_string(),
            pcr2: PCR.to_string(),
            pcr3: PCR.to_string(),
            aws_root_certificate: "3082021130820196a0".to_string(),
            default_dns_resolvers: dns_resolvers.iter().map(ToString::to_string).collect(),
        }
    }

    fn deploy_config(health_check_port: u16, public_ingress_port: u16) -> DeployConfig {
        let mut config = DeployConfig::template(None);
        config.pivot_args = vec!["--host".to_string(), "0.0.0.0".to_string()];
        config.health_check_port = health_check_port;
        config.public_ingress_port = public_ingress_port;
        config
    }

    fn build(
        app: TvcApp,
        info: GetTvcManifestGenerationInfoResponse,
        config: &DeployConfig,
    ) -> Result<Vec<u8>> {
        build_manifest(app, info, config, parse_pivot_digest(PIVOT_DIGEST).unwrap())
    }

    /// The manifest carries exactly the values the coordinator compares it
    /// against, in QOS canonical JSON, with an empty `qosCommit`.
    #[test]
    fn manifest_has_the_fields_the_coordinator_requires() {
        let bytes = build(app(true), info(&["10.0.0.2"]), &deploy_config(3000, 4000)).unwrap();

        assert_eq!(
            serde_json::from_slice::<Value>(&bytes).unwrap(),
            json!({
                "version": "v2",
                "namespace": {"name": NAMESPACE, "nonce": NONCE.to_string(), "quorumKey": "04aa01"},
                "pivot": {
                    "hash": PIVOT_DIGEST,
                    "restart": "Always",
                    "args": ["--host", "0.0.0.0"],
                    "bridgeConfig": [
                        {"type": "server", "port": "3000", "host": "0.0.0.0"},
                        {"type": "server", "port": "4000", "host": "0.0.0.0"},
                        {"type": "client", "port": "0"},
                    ],
                    "debugMode": false,
                },
                "manifestSet": {
                    "threshold": "2",
                    "members": [
                        {"alias": "alice", "pubKey": "0a0b"},
                        {"alias": "bob", "pubKey": "0c0d"},
                    ],
                },
                "shareSet": {"threshold": "1", "members": [{"alias": "carol", "pubKey": "0e0f"}]},
                "enclave": {
                    "pcr0": PCR,
                    "pcr1": PCR,
                    "pcr2": PCR,
                    "pcr3": PCR,
                    "awsRootCertificate": "3082021130820196a0",
                    "qosCommit": "",
                },
                "dns": {"resolvers": ["10.0.0.2"]},
            })
        );
    }

    /// Without egress there is no client rule and no DNS, and one port is
    /// bridged once.
    #[test]
    fn manifest_without_egress_has_one_server_rule_and_no_dns() {
        let bytes = build(app(false), info(&[]), &deploy_config(3000, 3000)).unwrap();

        let manifest: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(
            manifest["pivot"]["bridgeConfig"],
            json!([{"type": "server", "port": "3000", "host": "0.0.0.0"}])
        );
        assert_eq!(manifest.get("dns"), None);
    }
}
