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

pub async fn insert(item: media_items::ActiveModel) -> media_items::Model {
    let db = init_db().await.unwrap();

    let res = item.insert(&db).await.unwrap();

    db.close().await.unwrap();
    return res;
}

pub async fn select() -> MediaItemsDTO {
    let db = init_db().await.unwrap();

    let res = MediaItems::find()
        .one(&db)
        .await
        .unwrap()
        .unwrap()
        .into_ex();

    let res = MediaItemsDTO::from_model(res);
    db.close().await.unwrap();
    return res;
}
