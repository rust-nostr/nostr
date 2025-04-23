mod migrations;
mod model;
mod query;
mod schema;

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "mysql")]
pub use migrations::mysql::run_migrations;
#[cfg(feature = "postgres")]
pub use migrations::postgres::run_migrations;
#[cfg(feature = "sqlite")]
pub use migrations::sqlite::run_migrations;
#[cfg(feature = "postgres")]
pub use postgres::{postgres_connection_pool, NostrPostgres};
