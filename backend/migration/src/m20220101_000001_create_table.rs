use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("projects")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("id")
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new("name")
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new("description").string().null())
                    .col(ColumnDef::new("repo_url").string().null())
                    .col(ColumnDef::new("homepage").string().null())
                    .col(ColumnDef::new("purl").string().null())
                    .col(ColumnDef::new("default_branch").string().null())
                    .col(ColumnDef::new("revision").string().null())
                    .col(ColumnDef::new("package_manager").string().null())
                    .col(ColumnDef::new("manifest_path").string().null())
                    .col(ColumnDef::new("lockfile_path").string().null())
                    .col(ColumnDef::new("sbom_path").string().null())
                    .col(ColumnDef::new("source_path").string().null())
                    .col(ColumnDef::new("sbom_format").string().null())
                    .col(ColumnDef::new("last_scanned_at").timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new("created_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new("updated_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("packages")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("id")
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new("purl")
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new("purl_type").string().not_null())
                    .col(ColumnDef::new("namespace").string().null())
                    .col(ColumnDef::new("name").string().not_null())
                    .col(ColumnDef::new("qualifiers").json().null())
                    .col(
                        ColumnDef::new("created_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new("updated_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("scans")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("id")
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new("project_id").integer().not_null())
                    .col(ColumnDef::new("package_manager").string().null())
                    .col(ColumnDef::new("manifest_path").string().null())
                    .col(ColumnDef::new("lockfile_path").string().null())
                    .col(ColumnDef::new("branch").string().null())
                    .col(ColumnDef::new("revision").string().null())
                    .col(ColumnDef::new("source_path").string().null())
                    .col(ColumnDef::new("sbom_path").string().null())
                    .col(ColumnDef::new("sbom_format").string().null())
                    .col(ColumnDef::new("scanner").string().null())
                    .col(ColumnDef::new("sbom_hash").string().null())
                    .col(ColumnDef::new("status").string().null())
                    .col(ColumnDef::new("started_at").timestamp_with_time_zone().null())
                    .col(ColumnDef::new("completed_at").timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new("created_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new("updated_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-scans-project_id")
                            .from("scans", "project_id")
                            .to("projects", "id"),
                    )
                    .to_owned(),
            ).await?;

        manager
            .create_table(
                Table::create()
                    .table("direct_dependencies")
                    .if_not_exists()
                    .col(
                        ColumnDef::new("id")
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new("scan_id").integer().not_null())
                    .col(ColumnDef::new("project_id").integer().not_null())
                    .col(ColumnDef::new("package_id").integer().not_null())
                    .col(ColumnDef::new("declared_constraint").string().null())
                    .col(ColumnDef::new("resolved_version").string().null())
                    .col(ColumnDef::new("scope").string().null())
                    .col(ColumnDef::new("manager").string().null())
                    .col(ColumnDef::new("registry").string().null())
                    .col(ColumnDef::new("bom_ref").string().null())
                    .col(
                        ColumnDef::new("is_optional")
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new("created_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new("updated_at")
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-direct-dependencies-scan_id")
                            .from("direct_dependencies", "scan_id")
                            .to("scans", "id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-direct-dependencies-project_id")
                            .from("direct_dependencies", "project_id")
                            .to("projects", "id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-direct-dependencies-package_id")
                            .from("direct_dependencies", "package_id")
                            .to("packages", "id"),
                    )
                    .to_owned(),
            )
            .await?;



        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        manager
            .drop_table(Table::drop().table("direct_dependencies").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table("scans").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table("packages").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table("projects").to_owned())
            .await?;

        Ok(())
    }
}
