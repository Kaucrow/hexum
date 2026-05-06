use crate::prelude::*;
use super::QUERIES;

#[derive(Deserialize, Debug)]
pub struct Queries {
    pub user: User,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub create: String,
    pub get_by_id: String,
    pub get_by_username: String,
    pub get_by_email: String,
    pub insert: String,
    pub activate_by_id: String,
    pub delete_by_id: String,
}

pub fn init() -> anyhow::Result<()> {
    let queries = get_queries()?;
    QUERIES.set(queries).expect("Failed to set global queries.");
    Ok(())
}

pub fn get_queries() -> Result<Queries, config::ConfigError> {
    let base_path = get_base_path();

    let queries_directory = base_path.join("postgres");

    let filename = "queries.yaml";

    let settings = config::Config::builder()
        .add_source(config::File::from(
            queries_directory.join(filename),
        ))
        .build()?;

    settings.try_deserialize::<Queries>()
}