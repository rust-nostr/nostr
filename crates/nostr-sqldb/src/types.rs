use deadpool::managed::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

#[cfg(feature = "postgres")]
use diesel_async::AsyncPgConnection;
#[cfg(feature = "postgres")]
pub type DbConnectionPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
#[cfg(feature = "postgres")]
pub type DbConnection = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[cfg(feature = "mysql")]
use diesel_async::AsyncMysqlConnection;
#[cfg(feature = "mysql")]
pub type DbConnectionPool = Pool<AsyncDieselConnectionManager<AsyncMysqlConnection>>;
#[cfg(feature = "mysql")]
pub type DbConnection = Object<AsyncDieselConnectionManager<AsyncMysqlConnection>>;

#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection;
#[cfg(feature = "sqlite")]
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
#[cfg(feature = "sqlite")]
pub type DbConnection = SyncConnectionWrapper<SqliteConnection>;

// #[cfg(feature = "sqlite")]
// async fn get_connection() -> Result<SyncConnectionWrapper<SqliteConnection>, DatabaseError> {
//     let mut conn = SyncConnectionWrapper::<SqliteConnection>::establish(&database_url)
//         .await
//         .unwrap();
//     pool.get().await.map_err(DatabaseError::backend)
// }
