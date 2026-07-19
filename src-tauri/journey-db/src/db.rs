use sea_orm::*;

use crate::entity::{
    MediaItems, MediaItemsDTO,
    media_items::{self, ConvertableMediaItems},
};

async fn init_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    db.get_schema_registry("journey-db::entity::*")
        .sync(&db)
        .await?;

    Ok(db)
}

pub async fn insert(item: media_items::ActiveModel) -> Result<media_items::Model, DbErr> {
    let db = init_db().await?;
    let res = item.insert(&db).await?;
    db.close().await?;

    Ok(res)
}

pub async fn select() -> Result<MediaItemsDTO, DbErr> {
    let db = init_db().await?;
    let res = MediaItems::find().one(&db).await?.unwrap().into_ex();
    let res = MediaItemsDTO::from_model(res);
    db.close().await?;

    Ok(res)
}
