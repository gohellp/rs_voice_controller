use sqlx::{query_as, SqlitePool};

#[derive(Debug, sqlx::FromRow)]
pub struct VoicesInfo {
    pub id: i32,
    pub channel_id: String,
    pub owner_id: String,
}

impl VoicesInfo {
    pub async fn new(channel_id: String, owner_id: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("INSERT INTO voices_info(channel_id, owner_id) VALUES($1, $2) RETURNING *")
            .bind(channel_id)
            .bind(owner_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }
    
    pub async fn get_by_id(id: i32, pool: &SqlitePool)-> Self {
        query_as::<_,Self>("SELECT * FROM voices_info WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    pub async fn get_by_channel_id(channel_id: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("SELECT * FROM voices_info WHERE channel_id = $1")
            .bind(channel_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[inline]
    pub async fn change_owner(&self, new_owner: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("UPDATE voices_info SET owner_id = $1 WHERE id = $2 RETURNING *")
            .bind(new_owner)
            .bind(self.id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[inline]
    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM voices_info WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }
}