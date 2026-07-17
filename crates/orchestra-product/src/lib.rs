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
    let codex = required_source(pins, "codex")?;
    let t3code = required_source(pins, "t3code")?;
    verify_text(&root.join("integration/codex/UPSTREAM_REVISION"), codex)?;
    verify_text(&root.join("integration/t3code/UPSTREAM_REVISION"), t3code)?;

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

fn required_source<'a>(pins: &'a ProductPins, name: &str) -> Result<&'a str, ProductError> {
    pins.sources
        .get(name)
        .map(String::as_str)
        .filter(|value| value.len() >= 8)
        .ok_or_else(|| ProductError::Message(format!("missing or invalid source pin `{name}`")))
}

fn verify_text(path: &Path, expected: &str) -> Result<(), ProductError> {
    let actual = fs::read_to_string(path)?.trim().to_owned();
    if actual != expected {
        return Err(ProductError::Message(format!(
            "{} pins `{actual}`, expected `{expected}`",
            path.display()
        )));
    }
    Ok(())
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
                ("codex".into(), "aaaaaaaa".into()),
                ("t3code".into(), "bbbbbbbb".into()),
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
}
