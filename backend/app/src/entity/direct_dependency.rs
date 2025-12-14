use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

/// Represents a direct dependency of a project on a package.
/// Indirect deps stay in Neo4j; keep direct ones here for fast filtering and history.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "direct_dependencies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub scan_id: i32,
    pub project_id: i32,
    pub package_id: i32,
    /// Version constraint declared in manifest (e.g., ^1.2.3 or [1.0,2.0)).
    pub declared_constraint: Option<String>,
    /// Resolved version from lock/SBOM component entry.
    pub resolved_version: Option<String>,
    /// e.g., prod/dev/test scope from CycloneDX.
    pub scope: Option<String>,
    /// Package manager or ecosystem that produced the dependency.
    pub manager: Option<String>,
    /// Registry/source URI for the package (npm registry, Maven repo, etc.).
    pub registry: Option<String>,
    /// CycloneDX bom-ref to map back into the SBOM graph.
    pub bom_ref: Option<String>,
    /// Optional flag when dependency is marked optional in manifest.
    #[sea_orm(default_value = false)]
    pub is_optional: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::scan::Entity", from = "Column::ScanId", to = "super::scan::Column::Id")]
    Scan,
    #[sea_orm(belongs_to = "super::project::Entity", from = "Column::ProjectId", to = "super::project::Column::Id")]
    Project,
    #[sea_orm(belongs_to = "super::package::Entity", from = "Column::PackageId", to = "super::package::Column::Id")]
    Package,
}

impl Related<super::scan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Scan.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::package::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Package.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
