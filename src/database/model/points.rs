use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "points")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub randomized_id: i64,
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub spd: f64,
    pub azm: f64,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub timestamp: Option<DateTime>
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
