pub mod config;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

pub struct AppState {
    pub pool: sqlx::PgPool,
}
