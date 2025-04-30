use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use nostr_database::DatabaseError;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/postgres");

/// programatically run the db migrations
pub fn run_migrations(connection_string: &str) -> Result<(), DatabaseError> {
    info!("Running db migrations in postgres database",);
    let mut connection =
        PgConnection::establish(connection_string).map_err(DatabaseError::backend)?;

    let res = connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(DatabaseError::Backend)?;
    info!("Successfully executed postgres db migrations {:?}", res);
    Ok(())
}
