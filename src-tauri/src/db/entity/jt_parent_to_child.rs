use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "jt_parent_to_child")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub parent_id: uuid::Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub child_id: uuid::Uuid,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: Option<super::media_items::Entity>,
    #[sea_orm(belongs_to, relation_enum = "Child", from = "child_id", to = "uuid")]
    pub child: Option<super::media_items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
