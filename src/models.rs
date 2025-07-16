use sqlx::{query_as, SqlitePool};

#[derive(Debug, sqlx::FromRow)]
pub struct VoiceInfo {
    pub voice_id: String,
    pub owner_id: String,
}

#[allow(dead_code)]
impl VoiceInfo {
    pub async fn new(voice_id: String, owner_id: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("INSERT INTO voices_info(voice_id, owner_id) VALUES($1, $2) RETURNING *")
            .bind(voice_id)
            .bind(owner_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    pub async fn get_by_channel_id(voice_id: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("SELECT * FROM voices_info WHERE voice_id = $1")
            .bind(voice_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    pub async fn get_by_owner_id(owner_id: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("SELECT * FROM voices_info WHERE owner_id = $1")
            .bind(owner_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[inline]
    pub async fn change_owner(&self, new_owner: String, pool: &SqlitePool) -> Self {
        query_as::<_,Self>("UPDATE voices_info SET owner_id = $1 WHERE voice_id = $2 RETURNING *")
            .bind(new_owner)
            .bind(&self.voice_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[inline]
    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM voices_info WHERE voice_id = $1")
            .bind(&self.voice_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    #[allow(dead_code)]
    pub id: String,
    pub return_to_owned_channel: bool
}
