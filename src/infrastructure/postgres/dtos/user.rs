use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct UserDbRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
}

#[derive(sqlx::FromRow)]
pub struct UserAuthenticatorDbRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_id: Option<String>,
    pub passwd: Option<String>,
}