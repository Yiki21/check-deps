use std::{
    fs,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::{anyhow, Context};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    Client as S3Client,
    config::{Credentials, Region},
    primitives::ByteStream,
};
use axum::{extract::State, Json};
use axum_valid::Valid;
use chrono::Utc;
use flate2::{Compression, write::GzEncoder};
use git2::Repository;
use packageurl::PackageUrl;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use neo4rs::{query, Graph};
use tar::Builder;
use tracing::{debug, warn};
use tokio::task;
use validator::Validate;
use walkdir::WalkDir;

use crate::{
    app::AppState,
    common::{ApiError, ApiResponse, ApiResult},
    config::{LanguagesConfig, S3Config},
    entity::{direct_dependency, package, project, scan},
    id,
};

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct NewProject {
    pub name: String,
    pub description: Option<String>,
    #[validate(url)]
    pub repo_url: String,
    pub store_sbom: Option<bool>,
    pub store_source: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
pub struct RegisterResponse {
    pub project_id: i32,
    pub scan_id: i32,
    pub sbom_path: Option<String>,
    pub source_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CycloneDxBom {
    #[serde(default)]
    components: Vec<CycloneDxComponent>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CycloneDxComponent {
    #[serde(rename = "bom-ref")]
    bom_ref: Option<String>,
    purl: Option<String>,
    version: Option<String>,
    scope: Option<String>,
    #[serde(default)]
    properties: Vec<CycloneDxProperty>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CycloneDxProperty {
    name: String,
    value: Value,
}

#[axum::debug_handler]
pub async fn register_project(
    State(AppState { db, neo4j }): State<AppState>,
    Valid(Json(payload)): Valid<Json<NewProject>>,
) -> ApiResult<ApiResponse<RegisterResponse>> {
    // 1) Prepare workspace and repo context.
    let tmp_dir = create_tmp_dir()?;
    let package_type = clone_and_detect(&payload.repo_url, &tmp_dir).await?;

    // 2) Render SBOM and persist locally.
    let sbom_local_path = tmp_dir.join("bom.json");
    let sbom = render_and_store_sbom(&payload.repo_url, package_type, &sbom_local_path).await?;
    let components = parse_cyclonedx_components(&sbom)?;

    // 3) Optionally archive source.
    let source_archive_path = maybe_archive_source(&payload, &tmp_dir).await?;

    // 4) Optionally upload artifacts.
    let (sbom_path, source_path) =
        maybe_upload_to_s3(&payload, sbom.as_bytes(), source_archive_path.as_deref()).await?;

    // 5) Persist project + scan.
    let now = Utc::now();
    let pm_string = package_type.as_str().map(str::to_string);
    let (sbom_path_to_store, source_path_to_store) = resolve_artifact_paths(
        &payload,
        &sbom_path,
        &source_path,
        &sbom_local_path,
        source_archive_path.as_deref(),
    );

    let register_response = persist_project_and_scan(
        &db,
        &payload,
        &pm_string,
        &sbom_path_to_store,
        &source_path_to_store,
        now,
    )
    .await?;

    insert_direct_dependencies(
        &db,
        register_response.project_id,
        register_response.scan_id,
        &pm_string,
        &components,
    )
    .await?;

    if let Some(graph) = neo4j.as_ref() {
        if let Err(err) = sync_dependencies_to_neo4j(
            graph,
            register_response.project_id,
            register_response.scan_id,
            &payload,
            pm_string.as_deref(),
            &components,
        )
        .await
        {
            warn!(error = ?err, "failed to sync dependencies to Neo4j");
        }
    }

    // 6) Cleanup temp dir if unused.
    cleanup_tmp_dir_if_unused(&tmp_dir, &sbom_path_to_store, &source_path_to_store);

    Ok(ApiResponse::ok(
        "project registered",
        Some(register_response),
    ))
}

fn create_tmp_dir() -> anyhow::Result<PathBuf> {
    let tmp_dir = std::env::temp_dir().join(format!("check-deps-{}", id::next()));
    fs::create_dir_all(&tmp_dir).with_context(|| "can not create temp dir")?;
    Ok(tmp_dir)
}

async fn clone_and_detect(repo_url: &str, tmp_dir: &Path) -> anyhow::Result<PackageType> {
    let repo = clone_repo(repo_url, tmp_dir).await?;
    let repo_dir = repo.workdir().unwrap_or(repo.path());
    detect_language_and_package_type(repo_dir)
}

async fn render_and_store_sbom(
    repo_url: &str,
    package_type: PackageType,
    sbom_local_path: &Path,
) -> anyhow::Result<String> {
    let sbom = request_sbom(repo_url, package_type).await?;
    fs::write(sbom_local_path, sbom.as_bytes()).with_context(|| "fail when write data")?;
    Ok(sbom)
}

async fn maybe_archive_source(
    payload: &NewProject,
    tmp_dir: &Path,
) -> anyhow::Result<Option<PathBuf>> {
    if !payload.store_source.unwrap_or(false) {
        return Ok(None);
    }

    let path = tmp_dir.join("source.tar.gz");
    archive_source_dir_to_file(tmp_dir, &path).await?;
    Ok(Some(path))
}

fn resolve_artifact_paths(
    payload: &NewProject,
    sbom_path: &Option<String>,
    source_path: &Option<String>,
    sbom_local_path: &Path,
    source_archive_path: Option<&Path>,
) -> (Option<String>, Option<String>) {
    let sbom_path_to_store = sbom_path
        .clone()
        .or_else(|| sbom_local_path.to_str().map(|s| s.to_string()));

    let mut source_path_to_store = source_path.clone();
    if payload.store_source.unwrap_or(false) && source_path_to_store.is_none() {
        if let Some(path) = source_archive_path {
            source_path_to_store = path.to_str().map(|s| s.to_string());
        }
    }

    (sbom_path_to_store, source_path_to_store)
}

fn cleanup_tmp_dir_if_unused(
    tmp_dir: &Path,
    sbom_path: &Option<String>,
    source_path: &Option<String>,
) {
    let mut should_cleanup = true;

    if let Some(path) = sbom_path {
        if Path::new(path).starts_with(tmp_dir) {
            should_cleanup = false;
        }
    }

    if let Some(path) = source_path {
        if Path::new(path).starts_with(tmp_dir) {
            should_cleanup = false;
        }
    }

    if should_cleanup {
        let _ = fs::remove_dir_all(tmp_dir);
    }
}

fn parse_cyclonedx_components(sbom_json: &str) -> ApiResult<Vec<CycloneDxComponent>> {
    let bom: CycloneDxBom = serde_json::from_str(sbom_json)
        .with_context(|| "failed to parse CycloneDX SBOM")?;
    Ok(bom.components)
}

async fn insert_direct_dependencies(
    db: &DatabaseConnection,
    project_id: i32,
    scan_id: i32,
    pm_string: &Option<String>,
    components: &[CycloneDxComponent],
) -> ApiResult<()> {
    let now_tz = Utc::now();

    for component in components {
        let Some(purl) = component.purl.as_deref() else {
            continue;
        };

        let package_id = ensure_package(db, purl).await?;

        let dep_model = direct_dependency::ActiveModel {
            scan_id: Set(scan_id),
            project_id: Set(project_id),
            package_id: Set(package_id),
            declared_constraint: Set(None),
            resolved_version: Set(component.version.clone()),
            scope: Set(component.scope.clone()),
            manager: Set(pm_string.clone()),
            registry: Set(None),
            bom_ref: Set(component.bom_ref.clone()),
            is_optional: Set(false),
            created_at: Set(now_tz),
            updated_at: Set(now_tz),
            ..Default::default()
        };

        let _ = dep_model.insert(db).await?;
    }

    Ok(())
}

async fn sync_dependencies_to_neo4j(
    graph: &Graph,
    project_id: i32,
    scan_id: i32,
    payload: &NewProject,
    pm: Option<&str>,
    components: &[CycloneDxComponent],
) -> anyhow::Result<()> {
    let mut tx = graph.start_txn().await?;

    tx.run(
        query(
            "MERGE (p:Project {id: $project_id}) \
             SET p.name = $name, \
                 p.repo_url = $repo_url, \
                 p.package_manager = $package_manager, \
                 p.updated_at = datetime()",
        )
        .param("project_id", project_id as i64)
        .param("name", payload.name.as_str())
        .param("repo_url", payload.repo_url.as_str())
        .param("package_manager", pm.unwrap_or("")),
    )
    .await?;

    for component in components {
        let Some(purl) = component.purl.as_deref() else {
            continue;
        };

        let parsed = match PackageUrl::from_str(purl) {
            Ok(p) => p,
            Err(err) => {
                warn!(error = ?err, "skip invalid purl during Neo4j sync, purl = {}", purl);
                continue;
            }
        };

        let pkg_name = parsed.name().to_string();
        let pkg_type = parsed.ty().to_string();
        let pkg_namespace = parsed
            .namespace()
            .map(|s| s.to_string())
            .unwrap_or_default();

        tx.run(
            query(
                "MERGE (pkg:Package {purl: $purl}) \
                 SET pkg.name = $name, \
                     pkg.type = $type, \
                     pkg.namespace = $namespace, \
                     pkg.updated_at = datetime()",
            )
            .param("purl", purl)
            .param("name", pkg_name)
            .param("type", pkg_type)
            .param("namespace", pkg_namespace),
        )
        .await?;

        tx.run(
            query(
                "MATCH (p:Project {id: $project_id}), (pkg:Package {purl: $purl}) \
                 MERGE (p)-[r:DEPENDS_ON {scan_id: $scan_id, purl: $purl}]->(pkg) \
                 SET r.scope = $scope, \
                     r.resolved_version = $resolved_version, \
                     r.manager = $manager, \
                     r.bom_ref = $bom_ref, \
                     r.updated_at = datetime()",
            )
            .param("project_id", project_id as i64)
            .param("scan_id", scan_id as i64)
            .param("purl", purl)
            .param("scope", component.scope.clone().unwrap_or_default())
            .param("resolved_version", component.version.clone().unwrap_or_default())
            .param("manager", pm.unwrap_or(""))
            .param("bom_ref", component.bom_ref.clone().unwrap_or_default()),
        )
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn ensure_package(db: &DatabaseConnection, purl: &str) -> ApiResult<i32> {
    if let Some(existing) = package::Entity::find()
        .filter(package::Column::Purl.eq(purl))
        .one(db)
        .await? {
        return Ok(existing.id);
    }

    let parsed = PackageUrl::from_str(purl)
        .map_err(|e| ApiError::Biz(format!("invalid purl: {e}")))?;

    let qualifiers = serde_json::to_value(parsed.qualifiers()).ok();

    let now_tz = Utc::now();

    let pkg = package::ActiveModel {
        purl: Set(purl.to_string()),
        purl_type: Set(parsed.ty().to_string()),
        namespace: Set(parsed.namespace().map(|s| s.to_string())),
        name: Set(parsed.name().to_string()),
        qualifiers: Set(qualifiers),
        created_at: Set(now_tz),
        updated_at: Set(now_tz),
        ..Default::default()
    };

    let inserted = pkg.insert(db).await?;
    Ok(inserted.id)
}

async fn persist_project_and_scan(
    db: &DatabaseConnection,
    payload: &NewProject,
    pm_string: &Option<String>,
    sbom_path_to_store: &Option<String>,
    source_path_to_store: &Option<String>,
    now: chrono::DateTime<Utc>,
) -> ApiResult<RegisterResponse> {
    if project::Entity::find()
        .filter(project::Column::Name.eq(payload.name.clone()))
        .one(db)
        .await?
        .is_some()
    {
        return Err(ApiError::Biz("project name already exists".into()));
    }

    let project_model = project::ActiveModel {
        name: Set(payload.name.clone()),
        description: Set(payload.description.clone()),
        repo_url: Set(Some(payload.repo_url.clone())),
        homepage: Set(None),
        purl: Set(None),
        default_branch: Set(None),
        revision: Set(None),
        package_manager: Set(pm_string.clone()),
        manifest_path: Set(None),
        lockfile_path: Set(None),
        sbom_path: Set(sbom_path_to_store.clone()),
        source_path: Set(source_path_to_store.clone()),
        sbom_format: Set(Some("cyclonedx-json".to_string())),
        last_scanned_at: Set(Some(now)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let project = project_model.insert(db).await?;

    let scan_model = scan::ActiveModel {
        project_id: Set(project.id),
        package_manager: Set(pm_string.clone()),
        manifest_path: Set(None),
        lockfile_path: Set(None),
        branch: Set(None),
        revision: Set(None),
        source_path: Set(source_path_to_store.clone()),
        sbom_path: Set(sbom_path_to_store.clone()),
        sbom_format: Set(Some("cyclonedx-json".to_string())),
        scanner: Set(Some("cdxgen".to_string())),
        sbom_hash: Set(None),
        status: Set(Some("success".to_string())),
        started_at: Set(Some(now)),
        completed_at: Set(Some(now)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let scan = scan_model.insert(db).await?;

    Ok(RegisterResponse {
        project_id: project.id,
        scan_id: scan.id,
        sbom_path: sbom_path_to_store.clone(),
        source_path: source_path_to_store.clone(),
    })
}

async fn clone_repo(repo_url: &str, dest: &Path) -> anyhow::Result<Repository> {
    debug!("cloning repo {} into {:?}", repo_url, dest);

    let repo =
        Repository::clone(repo_url, dest).with_context(|| "failed to clone the target repo")?;

    Ok(repo)
}

async fn archive_source_dir_to_file(dir: &Path, output_file: &Path) -> anyhow::Result<()> {
    let dir = dir.to_path_buf();
    let output_file = output_file.to_path_buf();

    task::spawn_blocking(move || -> anyhow::Result<()> {
        let tar_gz = File::create(&output_file)
            .with_context(|| format!("failed to create output file {:?}", output_file))?;
        let encoder = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(encoder);

        tar.append_dir_all(".", &dir)
            .with_context(|| format!("failed to archive source directory {:?}", dir))?;

        tar.finish().context("failed to finish tar stream")?;
        // GzEncoder completes compression on drop
        Ok(())
    })
    .await??;

    Ok(())
}

async fn request_sbom(repo_url: &str, package_type: PackageType) -> anyhow::Result<String> {
    let cfg = crate::config::get();
    let languages_cfg = cfg.languages();

    let base_url = resolve_cdxgen_base_url(languages_cfg, package_type)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(languages_cfg.timeout_seconds()))
        .build()?;

    let endpoint = format!("{}/sbom", base_url.trim_end_matches('/'));

    let mut req: reqwest::RequestBuilder = client
        .get(endpoint)
        .query(&[("url", repo_url), ("multiProject", "true")]);

    if let Some(cdx_type) = package_type.cdxgen_type() {
        req = req.query(&[("type", cdx_type)]);
    }

    let resp = req.send().await?.error_for_status()?;

    Ok(resp.text().await?)
}

fn resolve_cdxgen_base_url(
    languages_cfg: &LanguagesConfig,
    package_type: PackageType,
) -> anyhow::Result<String> {
    let candidates = package_type.cdxgen_profiles();

    if let Some(url) = languages_cfg.resolve_base_url(candidates) {
        return Ok(url.to_string());
    }

    Err(anyhow!(format!(
        "cdxgen url not configured for profiles: {}",
        candidates.join(", ")
    )))
}

async fn maybe_upload_to_s3(
    payload: &NewProject,
    sbom_bytes: &[u8],
    source_archive: Option<&Path>,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    let cfg = crate::config::get().s3();

    if !cfg.enabled() {
        return Ok((None, None));
    }

    let bucket = cfg
        .bucket()
        .filter(|b| !b.is_empty())
        .ok_or_else(|| anyhow!("s3 bucket is required when s3.enabled=true"))?;
    let region = cfg
        .region()
        .filter(|r| !r.is_empty())
        .ok_or_else(|| anyhow!("s3 region is required when s3.enabled=true"))?;

    let client = build_s3_client(cfg, region).await?;

    let prefix = cfg.prefix().unwrap_or("").trim_matches('/');
    let safe_name = payload.name.replace(' ', "-");
    let base_key = if prefix.is_empty() {
        format!("{}-{}", safe_name, id::next())
    } else {
        format!("{}/{}-{}", prefix, safe_name, id::next())
    };

    let mut sbom_path = None;
    let mut source_path = None;

    if payload.store_sbom.unwrap_or(false) {
        let key = format!("{}/bom.json", base_key);
        sbom_path = Some(upload_bytes_to_s3(&client, bucket, &key, sbom_bytes.to_vec()).await?);
    }

    if payload.store_source.unwrap_or(false) {
        if let Some(path) = source_archive {
            let key = format!("{}/source.tar.gz", base_key);
            source_path = Some(upload_file_to_s3(&client, bucket, &key, path).await?);
        }
    }

    Ok((sbom_path, source_path))
}

async fn build_s3_client(cfg: &S3Config, region: &str) -> anyhow::Result<S3Client> {
    let mut loader =
        aws_config::defaults(BehaviorVersion::latest()).region(Region::new(region.to_string()));

    if let Some(endpoint) = cfg.endpoint() {
        loader = loader.endpoint_url(endpoint);
    }

    let shared = loader.load().await;
    let mut builder = aws_sdk_s3::config::Builder::from(&shared);

    if let (Some(access_key), Some(secret)) = (cfg.access_key_id(), cfg.secret_access_key()) {
        builder = builder
            .credentials_provider(Credentials::new(access_key, secret, None, None, "static"));
    }

    if let Some(endpoint) = cfg.endpoint() {
        builder = builder.endpoint_url(endpoint);
    }

    Ok(S3Client::from_conf(builder.build()))
}

async fn upload_bytes_to_s3(
    client: &S3Client,
    bucket: &str,
    key: &str,
    body: Vec<u8>,
) -> anyhow::Result<String> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(body))
        .send()
        .await?;

    Ok(format!("s3://{}/{}", bucket, key))
}

async fn upload_file_to_s3(
    client: &S3Client,
    bucket: &str,
    key: &str,
    path: &Path,
) -> anyhow::Result<String> {
    let body = ByteStream::from_path(path)
        .await
        .with_context(|| format!("failed to open path {:?} for upload", path))?;

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await?;

    Ok(format!("s3://{}/{}", bucket, key))
}

#[derive(Debug, Clone, Copy)]
enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Poetry,
    Maven,
    Gradle,
}

#[derive(Debug, Clone, Copy)]
enum PackageType {
    Rust,
    JavaScript(PackageManager),
    Python(PackageManager),
    Java(PackageManager),
    Go,
    Unknown,
}

impl PackageType {
    fn cdxgen_type(&self) -> Option<&'static str> {
        match *self {
            PackageType::Rust => Some("rust"),
            PackageType::JavaScript(_) => Some("nodejs"),
            PackageType::Python(_) => Some("python"),
            PackageType::Java(_) => Some("java"),
            PackageType::Go => Some("go"),
            PackageType::Unknown => None,
        }
    }

    fn cdxgen_profiles(&self) -> &'static [&'static str] {
        match *self {
            PackageType::JavaScript(_) => &["node", "full"],
            PackageType::Python(_) => &["python", "full"],
            PackageType::Java(_) => &["java", "full"],
            PackageType::Rust | PackageType::Go | PackageType::Unknown => &["full"],
        }
    }

    fn as_str(&self) -> Option<&'static str> {
        match *self {
            PackageType::Rust => Some("rust"),
            PackageType::JavaScript(PackageManager::Npm) => Some("npm"),
            PackageType::JavaScript(PackageManager::Yarn) => Some("yarn"),
            PackageType::JavaScript(PackageManager::Pnpm) => Some("pnpm"),
            PackageType::Python(PackageManager::Poetry) => Some("poetry"),
            PackageType::Java(PackageManager::Maven) => Some("maven"),
            PackageType::Java(PackageManager::Gradle) => Some("gradle"),
            PackageType::Go => Some("go"),
            _ => None,
        }
    }
}

fn detect_language_and_package_type(dir: &Path) -> anyhow::Result<PackageType> {
    const RULES: &[(&str, PackageType)] = &[
        ("Cargo.toml", PackageType::Rust),
        (
            "package-lock.json",
            PackageType::JavaScript(PackageManager::Npm),
        ),
        ("yarn.lock", PackageType::JavaScript(PackageManager::Yarn)),
        (
            "pnpm-lock.yaml",
            PackageType::JavaScript(PackageManager::Pnpm),
        ),
        ("package.json", PackageType::JavaScript(PackageManager::Npm)),
        (
            "pyproject.toml",
            PackageType::Python(PackageManager::Poetry),
        ),
        ("pom.xml", PackageType::Java(PackageManager::Maven)),
        ("build.gradle", PackageType::Java(PackageManager::Gradle)),
        ("go.mod", PackageType::Go),
    ];

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let file_name = entry.file_name().to_string_lossy();

        if let Some((_, pkg_type)) = RULES.iter().find(|(name, _)| *name == file_name) {
            return Ok(*pkg_type);
        }
    }

    Ok(PackageType::Unknown)
}
