use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "packages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    /// Full Package URL. Keep unique so we can reuse the same record across scans.
    #[sea_orm(unique, column_type = "Text")]
    pub purl: String,
    /// Extracted type from PURL (e.g. npm, maven, pypi) to speed lookups.
    pub purl_type: String,
    /// Optional namespace/group/id from PURL.
    pub namespace: Option<String>,
    /// Extracted name (artifact/module) from PURL.
    pub name: String,
    /// Qualifiers from PURL stored as JSON for flexible filtering.
    pub qualifiers: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::direct_dependency::Entity")]
    DirectDependency,
}

impl Related<super::direct_dependency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DirectDependency.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}