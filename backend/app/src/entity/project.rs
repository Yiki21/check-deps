use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Human readable name for the service/repo. Keep unique to avoid duplicates.
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    /// Repository HTTP(S) URL for cloning. Was `url`, keep separate to avoid clashes with app URL.
    pub repo_url: Option<String>,
    /// Optional public/project homepage URL.
    pub homepage: Option<String>,
    /// PURL for the project itself when applicable (e.g., published artifact id).
    pub purl: Option<String>,
    /// Default branch name (e.g., main/master) used for scheduled scans.
    pub default_branch: Option<String>,
    /// Last known commit or tag used for SBOM generation.
    pub revision: Option<String>,
    /// Package manager ecosystem (npm, pnpm, pip, maven, gradle, cargo, etc.).
    pub package_manager: Option<String>,
    /// Relative path to manifest (package.json, pom.xml, Cargo.toml...).
    pub manifest_path: Option<String>,
    /// Relative path to lockfile (package-lock.json, pnpm-lock.yaml, Cargo.lock...).
    pub lockfile_path: Option<String>,
    /// Path to the last generated SBOM artifact.
    pub sbom_path: Option<String>,
    /// Where the repo is checked out locally for scanning.
    pub source_path: Option<String>,
    /// SBOM format (cyclonedx-json, cyclonedx-xml, spdx-json...).
    pub sbom_format: Option<String>,
    /// When the project was last scanned successfully.
    pub last_scanned_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::scan::Entity")]
    Scan,
    #[sea_orm(has_many = "super::direct_dependency::Entity")]
    DirectDependency,
}

impl Related<super::scan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Scan.def()
    }
}

impl Related<super::direct_dependency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectDependency.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}