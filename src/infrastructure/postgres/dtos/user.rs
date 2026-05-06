use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct UserDbRow {
    pub id: Uuid,
    pub username: String,
    pub passwd: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
}