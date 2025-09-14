use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Points::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Points::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Points::RandomizedId).big_integer().not_null())
                    .col(ColumnDef::new(Points::Lat).double().not_null())
                    .col(ColumnDef::new(Points::Lng).double().not_null())
                    .col(ColumnDef::new(Points::Alt).double().not_null())
                    .col(ColumnDef::new(Points::Spd).double().not_null())
                    .col(ColumnDef::new(Points::Azm).double().not_null())
                    .col(
                        ColumnDef::new(Points::Timestamp)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Points::Anomaly).boolean())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Points::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Points {
    Table,
    Id,
    RandomizedId,
    Lat,
    Lng,
    Alt,
    Spd,
    Azm,
    Timestamp,
    Anomaly
}
