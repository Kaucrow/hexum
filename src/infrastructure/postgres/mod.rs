pub mod adapter;
mod dtos;
mod user;
mod queries;

use std::sync::OnceLock;
use queries::Queries;

static QUERIES: OnceLock<Queries> = OnceLock::new();

pub use adapter::PostgresAdapter;