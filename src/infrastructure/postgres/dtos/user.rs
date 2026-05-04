#[derive(sqlx::FromRow)]
pub struct UserDbRow {
    pub id: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
}