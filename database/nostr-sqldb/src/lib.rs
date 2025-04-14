mod migrations;
#[allow(dead_code)] // TODO: to remove when also SQLite and MySQL are implemented
mod model;
#[cfg(feature = "postgres")]
mod postgres;
#[allow(dead_code)] // TODO: to remove when also SQLite and MySQL are implemented
mod query;
mod schema;

#[cfg(feature = "mysql")]
pub use migrations::mysql::run_migrations;
#[cfg(feature = "postgres")]
pub use migrations::postgres::run_migrations;
#[cfg(feature = "sqlite")]
pub use migrations::sqlite::run_migrations;
#[cfg(feature = "postgres")]
pub use postgres::{postgres_connection_pool, NostrPostgres};
