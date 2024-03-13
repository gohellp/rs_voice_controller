use crate::schema::voices_info;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = voices_info)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct VoicesInfo {
    pub id: i32,
    pub channel_id: String,
    pub owner_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = voices_info)]
pub struct NewVoicesInfo<'a> {
    pub channel_id: &'a str,
    pub owner_id: &'a str,
}