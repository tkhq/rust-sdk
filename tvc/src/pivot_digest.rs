use anyhow::{anyhow, bail, Context, Result};
use oci_client::client::{linux_amd64_resolver, ClientConfig};
use oci_client::config::ConfigFile;
use oci_client::manifest::{
    OciDescriptor, IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE, IMAGE_DOCKER_LAYER_TAR_MEDIA_TYPE,
    IMAGE_LAYER_GZIP_MEDIA_TYPE, IMAGE_LAYER_MEDIA_TYPE,
};
use oci_client::secrets::RegistryAuth;
use oci_client::{Client, Reference};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use tar::Archive;
use tempfile::TempDir;

const DEBUG_ENV: &str = "TVC_DEBUG_PIVOT_DIGEST";
const MAX_CANDIDATE_PATHS: usize = 8;
const MAX_SAMPLE_PATHS: usize = 6;

#[derive(Debug, Clone)]
pub struct PivotDigestSource {
    pub image_url: String,
    pub pivot_path: String,
}

#[derive(Debug, Clone)]
pub struct PivotDigestResult {
    pub image_url: String,
    pub pivot_path: String,
    pub digest: String,
}

#[derive(Debug, Deserialize)]
struct DockerConfig {
    #[serde(default)]
    auths: HashMap<String, DockerAuthEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct DockerAuthEntry {
    #[serde(default)]
    auth: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default, rename = "identitytoken")]
    identity_token: Option<String>,
    #[serde(default, rename = "registrytoken")]
    registry_token: Option<String>,
}

pub async fn compute_pivot_digest(
    source: &PivotDigestSource,
    pull_secret_path: Option<&Path>,
) -> Result<PivotDigestResult> {
    let reference: Reference = source
        .image_url
        .parse()
        .with_context(|| format!("invalid image reference: {}", source.image_url))?;
    let target = normalize_pivot_path(&source.pivot_path)?;
    debug_log(format!(
        "computing pivot digest for image '{}' and target '{}'",
        source.image_url,
        target.display()
    ));
    let auth = registry_auth_for_reference(&reference, pull_secret_path)?;
    let client = build_client();
    let (manifest, manifest_digest, config_json) = client
        .pull_manifest_and_config(&reference, &auth)
        .await
        .with_context(|| {
            format!(
                "failed to pull image manifest and config for {}",
                source.image_url
            )
        })?;
    debug_log(format!(
        "resolved manifest {} with {} layer(s)",
        manifest_digest,
        manifest.layers.len()
    ));
    log_image_config(&config_json);

    let temp_dir = TempDir::new().context("failed to create temporary directory for layers")?;
    let mut current_contents: Option<Vec<u8>> = None;
    let mut candidate_paths = Vec::new();

    for (index, layer) in manifest.layers.iter().enumerate() {
        debug_log(format!(
            "inspecting layer {} digest={} media_type={}",
            index, layer.digest, layer.media_type
        ));
        let layer_path = temp_dir.path().join(format!("layer-{index}.blob"));
        let layer_file = tokio::fs::File::create(&layer_path)
            .await
            .with_context(|| {
                format!("failed to create temp layer file: {}", layer_path.display())
            })?;

        client
            .pull_blob(&reference, layer, layer_file)
            .await
            .with_context(|| {
                format!(
                    "failed to pull layer {} for {}",
                    layer.digest, source.image_url
                )
            })?;

        let inspection = apply_layer(&layer_path, layer, &target, &mut current_contents)
            .with_context(|| {
                format!(
                    "failed to inspect pivot path '{}' in layer {}",
                    source.pivot_path, layer.digest
                )
            })?;

        if inspection.found_exact_match {
            debug_log(format!(
                "layer {} contained the pivot path '{}'",
                index,
                target.display()
            ));
        }

        if inspection.cleared_target {
            debug_log(format!(
                "layer {} removed or shadowed the target path '{}'",
                index,
                target.display()
            ));
        }

        if !inspection.sample_paths.is_empty() {
            debug_log(format!(
                "layer {} sample paths: {}",
                index,
                inspection.sample_paths.join(", ")
            ));
        }

        for candidate in inspection.candidate_paths {
            if candidate_paths.len() < MAX_CANDIDATE_PATHS && !candidate_paths.contains(&candidate)
            {
                candidate_paths.push(candidate);
            }
        }
    }

    let contents = current_contents.ok_or_else(|| {
        let hint = if candidate_paths.is_empty() {
            format!(
                "pivot path '{}' was not found in image {}. Set {}=1 for per-layer debug logs.",
                source.pivot_path, source.image_url, DEBUG_ENV
            )
        } else {
            format!(
                "pivot path '{}' was not found in image {}. Candidate paths seen while inspecting layers: {}. Set {}=1 for per-layer debug logs.",
                source.pivot_path,
                source.image_url,
                candidate_paths.join(", "),
                DEBUG_ENV
            )
        };
        anyhow!(hint)
    })?;

    Ok(PivotDigestResult {
        image_url: source.image_url.clone(),
        pivot_path: source.pivot_path.clone(),
        digest: format!("{:x}", Sha256::digest(&contents)),
    })
}

pub async fn resolve_pinned_image_url(
    image_url: &str,
    pull_secret_path: Option<&Path>,
) -> Result<String> {
    if image_url.contains('@') {
        return Ok(image_url.to_string());
    }

    let reference: Reference = image_url
        .parse()
        .with_context(|| format!("invalid image reference: {image_url}"))?;
    let auth = registry_auth_for_reference(&reference, pull_secret_path)?;
    let client = build_client();
    let (_, digest) = client
        .pull_image_manifest(&reference, &auth)
        .await
        .with_context(|| format!("failed to resolve image digest for {image_url}"))?;
    debug_log(format!(
        "resolved pinned digest {digest} for image '{image_url}'"
    ));

    Ok(format!("{image_url}@{digest}"))
}

pub fn validate_expected_digest(actual_digest: &str, expected_digest: &str) -> Result<()> {
    let expected = normalize_expected_digest(expected_digest)?;
    let actual = normalize_expected_digest(actual_digest)?;

    if actual != expected {
        bail!("pivot digest mismatch: expected {expected}, got {actual}");
    }

    Ok(())
}

fn registry_auth_for_reference(
    reference: &Reference,
    pull_secret_path: Option<&Path>,
) -> Result<RegistryAuth> {
    let Some(path) = pull_secret_path else {
        debug_log("using anonymous registry auth");
        return Ok(RegistryAuth::Anonymous);
    };

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read pull secret file: {}", path.display()))?;

    if content.trim().is_empty() {
        bail!(
            "pull secret file is empty after trimming whitespace: {}",
            path.display()
        );
    }

    let config: DockerConfig = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse pull secret JSON: {}", path.display()))?;
    let registry = normalize_registry(reference.resolve_registry());
    debug_log(format!(
        "resolving registry credentials for '{}' from '{}'",
        registry,
        path.display()
    ));

    let entry = config
        .auths
        .into_iter()
        .find(|(key, _)| normalize_registry(key) == registry)
        .map(|(_, entry)| entry)
        .ok_or_else(|| {
            anyhow!(
                "no credentials found for registry '{}' in pull secret file {}",
                registry,
                path.display()
            )
        })?;

    if let Some(token) = entry
        .identity_token
        .filter(|value| !value.trim().is_empty())
        .or(entry
            .registry_token
            .filter(|value| !value.trim().is_empty()))
    {
        debug_log(format!("using bearer auth for registry '{registry}'"));
        return Ok(RegistryAuth::Bearer(token));
    }

    if let Some(auth) = entry.auth.filter(|value| !value.trim().is_empty()) {
        use base64::Engine;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(auth)
            .context("failed to decode base64 auth entry in pull secret")?;
        let decoded = String::from_utf8(decoded)
            .context("decoded pull secret auth entry is not valid UTF-8")?;
        let (username, password) = decoded
            .split_once(':')
            .ok_or_else(|| anyhow!("pull secret auth entry must decode to 'username:password'"))?;
        debug_log(format!(
            "using basic auth from encoded credentials for registry '{registry}'"
        ));
        return Ok(RegistryAuth::Basic(
            username.to_string(),
            password.to_string(),
        ));
    }

    match (entry.username, entry.password) {
        (Some(username), Some(password)) if !username.trim().is_empty() => {
            debug_log(format!(
                "using basic auth from explicit username/password for registry '{registry}'"
            ));
            Ok(RegistryAuth::Basic(username, password))
        }
        _ => bail!(
            "pull secret entry for registry '{}' does not contain usable credentials",
            registry
        ),
    }
}

fn build_client() -> Client {
    Client::new(ClientConfig {
        platform_resolver: Some(Box::new(linux_amd64_resolver)),
        ..Default::default()
    })
}

#[derive(Default)]
struct LayerInspection {
    found_exact_match: bool,
    cleared_target: bool,
    candidate_paths: Vec<String>,
    sample_paths: Vec<String>,
}

fn normalize_registry(registry: &str) -> String {
    let registry = registry
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .trim_end_matches("/v1")
        .trim_end_matches("/v2");

    match registry {
        "docker.io" | "registry-1.docker.io" => "index.docker.io".to_string(),
        other => other.to_string(),
    }
}

fn normalize_pivot_path(path: &str) -> Result<PathBuf> {
    if path.trim().is_empty() {
        bail!("pivot path cannot be empty");
    }

    let mut normalized = PathBuf::new();

    for component in Path::new(path).components() {
        match component {
            Component::Prefix(_) | Component::ParentDir => {
                bail!("pivot path must not contain parent-directory components: {path}");
            }
            Component::RootDir | Component::CurDir => {}
            Component::Normal(segment) => normalized.push(segment),
        }
    }

    if normalized.as_os_str().is_empty() {
        bail!("pivot path must resolve to a file inside the image: {path}");
    }

    Ok(normalized)
}

fn normalize_archive_path(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::ParentDir => {
                bail!(
                    "archive entry path contains unsupported traversal: {}",
                    path.display()
                );
            }
            Component::RootDir | Component::CurDir => {}
            Component::Normal(segment) => normalized.push(segment),
        }
    }

    Ok(normalized)
}

fn apply_layer(
    layer_path: &Path,
    layer: &OciDescriptor,
    target: &Path,
    current_contents: &mut Option<Vec<u8>>,
) -> Result<LayerInspection> {
    let media_type = layer.media_type.as_str();
    match media_type {
        IMAGE_LAYER_MEDIA_TYPE | IMAGE_DOCKER_LAYER_TAR_MEDIA_TYPE => {
            let file = File::open(layer_path)
                .with_context(|| format!("failed to open layer blob: {}", layer_path.display()))?;
            apply_archive(Archive::new(file), target, current_contents)
        }
        IMAGE_LAYER_GZIP_MEDIA_TYPE | IMAGE_DOCKER_LAYER_GZIP_MEDIA_TYPE => {
            let file = File::open(layer_path)
                .with_context(|| format!("failed to open layer blob: {}", layer_path.display()))?;
            let reader = flate2::read::GzDecoder::new(file);
            apply_archive(Archive::new(reader), target, current_contents)
        }
        unsupported => bail!("unsupported layer media type: {unsupported}"),
    }
}

fn apply_archive<R: Read>(
    mut archive: Archive<R>,
    target: &Path,
    current_contents: &mut Option<Vec<u8>>,
) -> Result<LayerInspection> {
    let mut inspection = LayerInspection::default();

    for entry in archive
        .entries()
        .context("failed to read tar archive entries")?
    {
        let mut entry = entry.context("failed to read tar archive entry")?;
        let path = normalize_archive_path(
            &entry
                .path()
                .context("failed to read tar archive entry path")?,
        )?;

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if file_name == ".wh..wh..opq" {
            let opaque_dir = path.parent().unwrap_or_else(|| Path::new(""));
            if target.starts_with(opaque_dir) {
                *current_contents = None;
                inspection.cleared_target = true;
                debug_log(format!(
                    "opaque whiteout removed target via directory '{}'",
                    opaque_dir.display()
                ));
            }
            continue;
        }

        if let Some(whiteout_target) = file_name.strip_prefix(".wh.") {
            let deleted_path = path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(whiteout_target);
            if target == deleted_path || target.starts_with(&deleted_path) {
                *current_contents = None;
                inspection.cleared_target = true;
                debug_log(format!(
                    "whiteout removed target via path '{}'",
                    deleted_path.display()
                ));
            }
            continue;
        }

        if looks_like_candidate(&path, target)
            && inspection.candidate_paths.len() < MAX_CANDIDATE_PATHS
        {
            inspection.candidate_paths.push(path.display().to_string());
        }

        if inspection.sample_paths.len() < MAX_SAMPLE_PATHS {
            inspection.sample_paths.push(path.display().to_string());
        }

        if path != target {
            continue;
        }

        if !entry.header().entry_type().is_file() {
            bail!(
                "pivot path '{}' resolved to a non-regular file in the image layer",
                target.display()
            );
        }

        let mut contents = Vec::new();
        entry
            .read_to_end(&mut contents)
            .context("failed to read pivot file contents from image layer")?;
        *current_contents = Some(contents);
        inspection.found_exact_match = true;
    }

    Ok(inspection)
}

fn looks_like_candidate(path: &Path, target: &Path) -> bool {
    let target_name = target.file_name();
    let path_name = path.file_name();

    path_name.is_some()
        && (target_name == path_name
            || path
                .display()
                .to_string()
                .contains(target.to_string_lossy().as_ref()))
}

fn debug_log(message: impl AsRef<str>) {
    if std::env::var_os(DEBUG_ENV).is_some() {
        eprintln!("[pivot-digest] {}", message.as_ref());
    }
}

fn log_image_config(config_json: &str) {
    let Ok(config) = serde_json::from_str::<ConfigFile>(config_json) else {
        debug_log("failed to parse image config JSON for debug output");
        return;
    };

    debug_log(format!(
        "image platform from config: os={} arch={}",
        config.os, config.architecture
    ));

    if let Some(runtime) = config.config {
        if let Some(entrypoint) = runtime.entrypoint.filter(|value| !value.is_empty()) {
            debug_log(format!("image entrypoint: {}", entrypoint.join(" ")));
        }

        if let Some(cmd) = runtime.cmd.filter(|value| !value.is_empty()) {
            debug_log(format!("image cmd: {}", cmd.join(" ")));
        }

        if let Some(working_dir) = runtime.working_dir.filter(|value| !value.is_empty()) {
            debug_log(format!("image working dir: {working_dir}"));
        }
    }
}

fn normalize_expected_digest(value: &str) -> Result<String> {
    let digest = value
        .trim()
        .strip_prefix("sha256:")
        .unwrap_or(value.trim())
        .to_ascii_lowercase();

    if digest.len() != 64 || !digest.chars().all(|ch| ch.is_ascii_hexdigit()) {
        bail!("pivot digest must be 64 hexadecimal characters, optionally prefixed with 'sha256:'");
    }

    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::normalize_expected_digest;

    #[test]
    fn normalize_expected_digest_accepts_prefixed_value() {
        let digest = normalize_expected_digest(
            "sha256:cbe01169428f144086bfaef348bbf3db70f9217628996cafd2ecb85d5f2b47a1",
        )
        .unwrap();

        assert_eq!(
            digest,
            "cbe01169428f144086bfaef348bbf3db70f9217628996cafd2ecb85d5f2b47a1"
        );
    }

    #[test]
    fn normalize_expected_digest_accepts_raw_value() {
        let digest = normalize_expected_digest(
            "CBE01169428F144086BFAEF348BBF3DB70F9217628996CAFD2ECB85D5F2B47A1",
        )
        .unwrap();

        assert_eq!(
            digest,
            "cbe01169428f144086bfaef348bbf3db70f9217628996cafd2ecb85d5f2b47a1"
        );
    }

    #[test]
    fn normalize_expected_digest_rejects_invalid_length() {
        let error = normalize_expected_digest("abc").unwrap_err().to_string();

        assert!(error.contains("64 hexadecimal characters"));
    }
}
