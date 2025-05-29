use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use nostr_database::DatabaseError;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/sqlite");

/// programatically run the db migrations
pub fn run_migrations(connection_string: &str) -> Result<(), DatabaseError> {
    info!("Running db migrations in sqlite database",);
    let mut connection =
        SqliteConnection::establish(connection_string).map_err(DatabaseError::backend)?;

    let res = connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(DatabaseError::Backend)?;
    info!("Successfully executed sqlite db migrations {:?}", res);
    Ok(())
}
