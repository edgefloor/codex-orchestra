use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod release;

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ProductError {
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProductPins {
    pub product: ProductIdentity,
    pub sources: BTreeMap<String, String>,
    pub schemas: SchemaPins,
    pub evaluator: EvaluatorPins,
    pub capabilities: Vec<String>,
    pub limits: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProductIdentity {
    pub version: String,
    pub minimum_macos: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SchemaPins {
    pub protocol: String,
    pub snapshot: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all(serialize = "camelCase"))]
pub struct EvaluatorPins {
    pub revision: String,
    #[serde(alias = "adapterAbi")]
    pub adapter_abi: String,
    pub canonicalizer: String,
    #[serde(alias = "issueFormat")]
    pub issue_format: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactInput {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactIdentity {
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnsignedReleaseManifest {
    schema_version: u32,
    product_version: String,
    minimum_macos: String,
    target: String,
    sources: BTreeMap<String, String>,
    schemas: BTreeMap<String, SchemaIdentity>,
    evaluator: EvaluatorPins,
    capabilities: Vec<String>,
    limits: BTreeMap<String, u64>,
    artifacts: BTreeMap<String, ArtifactIdentity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaIdentity {
    pub identity: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub product_version: String,
    pub minimum_macos: String,
    pub target: String,
    pub sources: BTreeMap<String, String>,
    pub schemas: BTreeMap<String, SchemaIdentity>,
    pub evaluator: EvaluatorPins,
    pub capabilities: Vec<String>,
    pub limits: BTreeMap<String, u64>,
    pub artifacts: BTreeMap<String, ArtifactIdentity>,
    pub manifest_sha256: String,
}

pub fn read_pins(root: &Path) -> Result<ProductPins, ProductError> {
    let path = root.join("product/pins.toml");
    Ok(toml::from_str(&fs::read_to_string(&path).map_err(
        |error| ProductError::Message(format!("failed to read {}: {error}", path.display())),
    )?)?)
}

pub fn build_manifest(
    pins: ProductPins,
    target: impl Into<String>,
    artifact_inputs: &[ArtifactInput],
) -> Result<ReleaseManifest, ProductError> {
    let mut capabilities = pins.capabilities;
    capabilities.sort();
    capabilities.dedup();
    if capabilities.is_empty() {
        return Err(ProductError::Message(
            "Product release must declare at least one capability".into(),
        ));
    }
    let target = target.into();
    if target.trim().is_empty() {
        return Err(ProductError::Message("target must not be empty".into()));
    }

    let schemas = BTreeMap::from([
        ("protocol".into(), schema_identity(pins.schemas.protocol)),
        ("snapshot".into(), schema_identity(pins.schemas.snapshot)),
    ]);
    let mut artifacts = BTreeMap::new();
    for input in artifact_inputs {
        if input.name.trim().is_empty() {
            return Err(ProductError::Message(
                "artifact name must not be empty".into(),
            ));
        }
        if artifacts.contains_key(&input.name) {
            return Err(ProductError::Message(format!(
                "duplicate artifact name `{}`",
                input.name
            )));
        }
        let bytes = fs::read(&input.path).map_err(|error| {
            ProductError::Message(format!(
                "failed to read artifact {} at {}: {error}",
                input.name,
                input.path.display()
            ))
        })?;
        artifacts.insert(
            input.name.clone(),
            ArtifactIdentity {
                bytes: bytes.len() as u64,
                sha256: sha256(&bytes),
            },
        );
    }

    let unsigned = UnsignedReleaseManifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        product_version: pins.product.version,
        minimum_macos: pins.product.minimum_macos,
        target,
        sources: pins.sources,
        schemas,
        evaluator: pins.evaluator,
        capabilities,
        limits: pins.limits,
        artifacts,
    };
    let manifest_sha256 = sha256(&serde_json::to_vec(&unsigned)?);
    Ok(ReleaseManifest {
        schema_version: unsigned.schema_version,
        product_version: unsigned.product_version,
        minimum_macos: unsigned.minimum_macos,
        target: unsigned.target,
        sources: unsigned.sources,
        schemas: unsigned.schemas,
        evaluator: unsigned.evaluator,
        capabilities: unsigned.capabilities,
        limits: unsigned.limits,
        artifacts: unsigned.artifacts,
        manifest_sha256,
    })
}

pub fn write_manifest(path: &Path, manifest: &ReleaseManifest) -> Result<(), ProductError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    let mut bytes = serde_json::to_vec_pretty(manifest)?;
    bytes.push(b'\n');
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, path)?;
    Ok(())
}

pub fn verify_manifest_identity(manifest: &ReleaseManifest) -> Result<(), ProductError> {
    let unsigned = UnsignedReleaseManifest {
        schema_version: manifest.schema_version,
        product_version: manifest.product_version.clone(),
        minimum_macos: manifest.minimum_macos.clone(),
        target: manifest.target.clone(),
        sources: manifest.sources.clone(),
        schemas: manifest.schemas.clone(),
        evaluator: manifest.evaluator.clone(),
        capabilities: manifest.capabilities.clone(),
        limits: manifest.limits.clone(),
        artifacts: manifest.artifacts.clone(),
    };
    let actual = sha256(&serde_json::to_vec(&unsigned)?);
    if actual != manifest.manifest_sha256 {
        return Err(ProductError::Message(format!(
            "release manifest identity mismatch: expected {}, calculated {actual}",
            manifest.manifest_sha256
        )));
    }
    Ok(())
}

pub fn verify_manifest_artifact(
    manifest: &ReleaseManifest,
    name: &str,
    path: &Path,
) -> Result<(), ProductError> {
    verify_manifest_identity(manifest)?;
    let expected = manifest.artifacts.get(name).ok_or_else(|| {
        ProductError::Message(format!("release manifest has no artifact `{name}`"))
    })?;
    let bytes = fs::read(path).map_err(|error| {
        ProductError::Message(format!(
            "failed to read artifact `{name}` at {}: {error}",
            path.display()
        ))
    })?;
    let actual_sha256 = sha256(&bytes);
    if bytes.len() as u64 != expected.bytes || actual_sha256 != expected.sha256 {
        return Err(ProductError::Message(format!(
            "artifact `{name}` does not match the sealed release manifest"
        )));
    }
    Ok(())
}

pub fn verify_repository(root: &Path, pins: &ProductPins) -> Result<(), ProductError> {
    let codex = required_revision(pins, "orchestra_codex")?;
    required_revision(pins, "orchestra_codex_tree")?;
    let codex_upstream = required_revision(pins, "codex_upstream")?;
    required_revision(pins, "codex_upstream_tree")?;
    required_revision(pins, "orchestra_core_revision")?;
    required_revision(pins, "orchestra_core_tree")?;
    let desktop = required_revision(pins, "orchestra_desktop")?;
    required_revision(pins, "orchestra_desktop_tree")?;
    let t3code_upstream = required_revision(pins, "t3code_upstream")?;
    required_revision(pins, "t3code_upstream_tree")?;
    required_repository(pins, "orchestra_codex_repository")?;
    required_repository(pins, "codex_upstream_repository")?;
    required_repository(pins, "orchestra_core_repository")?;
    required_repository(pins, "orchestra_desktop_repository")?;
    required_repository(pins, "t3code_upstream_repository")?;
    required_revision(pins, "protocol_tree")?;
    required_digest(pins, "protocol_digest")?;
    required_revision(pins, "bun")?;
    required_repository(pins, "bun_repository")?;
    required_revision(pins, "zod")?;
    required_repository(pins, "zod_repository")?;
    required_revision(pins, "zod_package_revision")?;
    required_revision(pins, "zod_package_shasum")?;
    let worker_source_digest = required_digest(pins, "evaluator_worker_source_sha256")?;
    let lock_digest = required_digest(pins, "evaluator_lock_sha256")?;
    let package_digest = required_digest(pins, "evaluator_package_sha256")?;
    let bun_version = required_source(pins, "bun_version")?;
    let zod_version = required_source(pins, "zod_version")?;
    let zod_integrity = required_source(pins, "zod_package_integrity")?;
    let protocol_file_count = pins
        .sources
        .get("protocol_file_count")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|count| *count > 0);
    if pins
        .sources
        .get("protocol_digest_algorithm")
        .map(String::as_str)
        != Some("sha256-relative-path-nul-file-sha256-lf-v1")
        || protocol_file_count.is_none()
    {
        return Err(ProductError::Message(
            "generated protocol digest algorithm or file count is invalid".into(),
        ));
    }
    if codex == codex_upstream || desktop == t3code_upstream {
        return Err(ProductError::Message(
            "fork revisions must be distinct from their upstream base revisions".into(),
        ));
    }
    if pins.evaluator.revision != format!("bun-{bun_version}-zod-{zod_version}-sealed-2")
        || pins.evaluator.adapter_abi != "orchestra-evaluator-abi-v1"
        || pins.evaluator.canonicalizer != "rfc8785-jcs-v1"
        || pins.evaluator.issue_format != "orchestra-validation-issues-v1"
    {
        return Err(ProductError::Message(
            "evaluator revision, ABI, canonicalizer, or issue format is not the sealed Product identity"
                .into(),
        ));
    }
    for limit in [
        "validation_request_bytes",
        "validation_response_bytes",
        "validation_bundle_bytes",
        "validation_value_bytes",
        "validation_value_depth",
        "validation_value_nodes",
        "validation_collection_entries",
        "validation_string_bytes",
        "validation_issue_count",
        "validation_issue_text_bytes",
        "validation_wall_ms",
    ] {
        if !pins.limits.contains_key(limit) {
            return Err(ProductError::Message(format!(
                "sealed evaluator limit `{limit}` is missing"
            )));
        }
    }

    let worker_source = fs::read(root.join("evaluator/worker.ts"))?;
    let lock_source = fs::read(root.join("evaluator/bun.lock"))?;
    let package_source = fs::read(root.join("evaluator/package.json"))?;
    for (name, bytes, expected) in [
        (
            "worker source",
            worker_source.as_slice(),
            worker_source_digest,
        ),
        ("lockfile", lock_source.as_slice(), lock_digest),
        (
            "package manifest",
            package_source.as_slice(),
            package_digest,
        ),
    ] {
        if sha256(bytes) != expected {
            return Err(ProductError::Message(format!(
                "sealed evaluator {name} digest does not match Product provenance"
            )));
        }
    }
    let worker_source = String::from_utf8(worker_source)
        .map_err(|_| ProductError::Message("evaluator worker source is not UTF-8".into()))?;
    if !worker_source.contains(&format!(
        "const EVALUATOR_REVISION = \"{}\"",
        pins.evaluator.revision
    )) {
        return Err(ProductError::Message(
            "evaluator worker does not embed its sealed Product revision".into(),
        ));
    }
    let lock_source = String::from_utf8(lock_source)
        .map_err(|_| ProductError::Message("evaluator lockfile is not UTF-8".into()))?;
    if !lock_source.contains(&format!("zod@{zod_version}")) || !lock_source.contains(zod_integrity)
    {
        return Err(ProductError::Message(
            "evaluator lockfile does not contain the sealed Zod package identity".into(),
        ));
    }
    let package: serde_json::Value = serde_json::from_slice(&package_source)?;
    if package
        .pointer("/dependencies/zod")
        .and_then(serde_json::Value::as_str)
        != Some(zod_version)
    {
        return Err(ProductError::Message(
            "evaluator package does not pin the sealed Zod version".into(),
        ));
    }

    for retired in [
        "integration/codex",
        "integration/t3code",
        "scripts/codex-integration.sh",
        "scripts/t3code-integration.sh",
    ] {
        if root.join(retired).exists() {
            return Err(ProductError::Message(format!(
                "retired patch assembly artifact still exists: {retired}"
            )));
        }
    }
    for script in [
        "scripts/product-source-prepare.sh",
        "scripts/product-source-verify.sh",
        "scripts/product-dev-build.sh",
        "scripts/product-release.sh",
        "scripts/product-dogfood.sh",
    ] {
        let source = fs::read_to_string(root.join(script))?;
        for forbidden in [
            "git apply",
            "codex-integration.sh",
            "t3code-integration.sh",
            ".patch",
        ] {
            if source.contains(forbidden) {
                return Err(ProductError::Message(format!(
                    "direct-fork Product script {script} contains retired assembly token `{forbidden}`"
                )));
            }
        }
    }

    let plugin: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(".codex-plugin/plugin.json"))?)?;
    let plugin_version = plugin
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ProductError::Message("plugin version is missing".into()))?;
    let short_codex = &codex[..codex.len().min(8)];
    if !plugin_version.contains(short_codex) {
        return Err(ProductError::Message(format!(
            "plugin version `{plugin_version}` does not contain pinned Codex `{short_codex}`"
        )));
    }
    if pins
        .capabilities
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(ProductError::Message(
            "capabilities must not contain empty values".into(),
        ));
    }
    Ok(())
}

fn required_revision<'a>(pins: &'a ProductPins, name: &str) -> Result<&'a str, ProductError> {
    pins.sources
        .get(name)
        .map(String::as_str)
        .filter(|value| {
            value.len() == 40
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| ProductError::Message(format!("missing or invalid source pin `{name}`")))
}

fn required_repository<'a>(pins: &'a ProductPins, name: &str) -> Result<&'a str, ProductError> {
    pins.sources
        .get(name)
        .map(String::as_str)
        .filter(|value| value.starts_with("https://github.com/") && value.ends_with(".git"))
        .ok_or_else(|| ProductError::Message(format!("missing or invalid source pin `{name}`")))
}

fn required_source<'a>(pins: &'a ProductPins, name: &str) -> Result<&'a str, ProductError> {
    pins.sources
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProductError::Message(format!("missing or invalid source pin `{name}`")))
}

fn required_digest<'a>(pins: &'a ProductPins, name: &str) -> Result<&'a str, ProductError> {
    pins.sources
        .get(name)
        .map(String::as_str)
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| ProductError::Message(format!("missing or invalid digest pin `{name}`")))
}

fn schema_identity(identity: String) -> SchemaIdentity {
    SchemaIdentity {
        sha256: sha256(identity.as_bytes()),
        identity,
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn pins() -> ProductPins {
        ProductPins {
            product: ProductIdentity {
                version: "0.2.0-dev".into(),
                minimum_macos: "13.0".into(),
            },
            sources: BTreeMap::from([
                ("orchestra_codex".into(), "a".repeat(40)),
                (
                    "orchestra_codex_repository".into(),
                    "https://github.com/edgefloor/orchestra-codex.git".into(),
                ),
                ("codex_upstream".into(), "b".repeat(40)),
                (
                    "codex_upstream_repository".into(),
                    "https://github.com/openai/codex.git".into(),
                ),
                ("orchestra_desktop".into(), "c".repeat(40)),
                (
                    "orchestra_desktop_repository".into(),
                    "https://github.com/edgefloor/orchestra-desktop.git".into(),
                ),
                ("t3code_upstream".into(), "d".repeat(40)),
                (
                    "t3code_upstream_repository".into(),
                    "https://github.com/pingdotgg/t3code.git".into(),
                ),
            ]),
            schemas: SchemaPins {
                protocol: "protocol-v1".into(),
                snapshot: "snapshot-v1".into(),
            },
            evaluator: EvaluatorPins {
                revision: "evaluator-v1".into(),
                adapter_abi: "adapter-v1".into(),
                canonicalizer: "jcs-v1".into(),
                issue_format: "issues-v1".into(),
            },
            capabilities: vec!["z".into(), "a".into(), "a".into()],
            limits: BTreeMap::from([("frame_bytes".into(), 1024)]),
        }
    }

    #[test]
    fn identical_inputs_produce_identical_manifest_identity() {
        let temp = tempdir().unwrap();
        let artifact = temp.path().join("host");
        fs::write(&artifact, b"host bytes").unwrap();
        let inputs = [ArtifactInput {
            name: "host".into(),
            path: artifact,
        }];
        let first = build_manifest(pins(), "arm64-apple-darwin", &inputs).unwrap();
        let second = build_manifest(pins(), "arm64-apple-darwin", &inputs).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.capabilities, ["a", "z"]);
        assert_eq!(first.manifest_sha256.len(), 64);
    }

    #[test]
    fn artifact_content_changes_release_identity() {
        let temp = tempdir().unwrap();
        let artifact = temp.path().join("host");
        fs::write(&artifact, b"first").unwrap();
        let inputs = [ArtifactInput {
            name: "host".into(),
            path: artifact.clone(),
        }];
        let first = build_manifest(pins(), "arm64-apple-darwin", &inputs).unwrap();
        fs::write(artifact, b"second").unwrap();
        let second = build_manifest(pins(), "arm64-apple-darwin", &inputs).unwrap();
        assert_ne!(first.manifest_sha256, second.manifest_sha256);
    }

    #[test]
    fn manifest_identity_rejects_tampering() {
        let temp = tempdir().unwrap();
        let artifact = temp.path().join("host");
        fs::write(&artifact, b"host bytes").unwrap();
        let mut manifest = build_manifest(
            pins(),
            "arm64-apple-darwin",
            &[ArtifactInput {
                name: "host".into(),
                path: artifact,
            }],
        )
        .unwrap();
        verify_manifest_identity(&manifest).unwrap();
        manifest.product_version = "tampered".into();
        assert!(verify_manifest_identity(&manifest).is_err());
    }

    #[test]
    fn manifest_artifact_verification_rejects_substitution() {
        let temp = tempdir().unwrap();
        let artifact = temp.path().join("host");
        fs::write(&artifact, b"host bytes").unwrap();
        let manifest = build_manifest(
            pins(),
            "arm64-apple-darwin",
            &[ArtifactInput {
                name: "host".into(),
                path: artifact.clone(),
            }],
        )
        .unwrap();
        verify_manifest_artifact(&manifest, "host", &artifact).unwrap();
        fs::write(&artifact, b"substituted").unwrap();
        assert!(verify_manifest_artifact(&manifest, "host", &artifact).is_err());
    }

    #[test]
    fn repository_verification_rejects_evaluator_material_drift() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let mut pins = read_pins(root).unwrap();
        verify_repository(root, &pins).unwrap();
        pins.sources
            .insert("evaluator_worker_source_sha256".into(), "0".repeat(64));
        let error = verify_repository(root, &pins).unwrap_err().to_string();
        assert!(error.contains("worker source digest"));
    }
}
