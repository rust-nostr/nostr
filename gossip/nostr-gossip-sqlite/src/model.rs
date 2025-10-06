use sqlx::FromRow;

#[derive(FromRow)]
pub(super) struct ListRow {
    pub(super) event_created_at: Option<i64>,
    pub(super) last_checked_at: Option<i64>,
}
