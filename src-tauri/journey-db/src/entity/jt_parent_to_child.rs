use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "jt_parent_to_child")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub parent_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub child_id: Uuid,
    #[sea_orm(belongs_to, from = "parent_id", to = "uuid")]
    pub parent: BelongsTo<super::media_items::Entity>,
    #[sea_orm(belongs_to, relation_enum = "Child", from = "child_id", to = "uuid")]
    pub child: BelongsTo<super::media_items::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
