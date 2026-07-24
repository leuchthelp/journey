use anyhow::Result;
use sea_orm::*;
use tokio::sync::OnceCell;

pub static DB_CLIENT: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn init_db() -> Result<()> {
    if DB_CLIENT.initialized() {
        return Ok(());
    }

    let db = Database::connect("sqlite::memory:").await?;

    db.get_schema_registry("journey-db::entity::*")
        .sync(&db)
        .await?;

    DB_CLIENT.set(db)?;
    Ok(())
}

pub fn get_conn() -> &'static DatabaseConnection {
    return DB_CLIENT.get().unwrap();
}
