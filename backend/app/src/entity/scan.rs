use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

/// Records each SBOM generation run (usually via CycloneDX toolchain) per project.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "scans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    /// Package manager/ecosystem for this scan (npm, pnpm, pip, maven, gradle, cargo, etc.).
    pub package_manager: Option<String>,
    /// Relative manifest path used for this run.
    pub manifest_path: Option<String>,
    /// Relative lockfile path used for this run.
    pub lockfile_path: Option<String>,
    /// Branch used for the scan; keep alongside revision to reproduce.
    pub branch: Option<String>,
    /// Commit/tag/sha used for the SBOM generation.
    pub revision: Option<String>,
    /// Local checkout path where the scan was executed.
    pub source_path: Option<String>,
    /// Path to generated SBOM artifact.
    pub sbom_path: Option<String>,
    /// SBOM format (cyclonedx-json, cyclonedx-xml, etc.).
    pub sbom_format: Option<String>,
    /// Scanner/CLI version (e.g., cyclonedx-cli@X.Y.Z).
    pub scanner: Option<String>,
    /// Hash of SBOM file to detect duplicates.
    pub sbom_hash: Option<String>,
    /// Run status: pending/running/success/failed.
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::project::Entity", from = "Column::ProjectId", to = "super::project::Column::Id")]
    Project,
    #[sea_orm(has_many = "super::direct_dependency::Entity")]
    DirectDependency,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::direct_dependency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectDependency.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}