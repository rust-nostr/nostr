use rusqlite::Row;

pub(super) struct ListRow {
    pub(super) event_created_at: Option<i64>,
    pub(super) last_checked_at: Option<i64>,
}

impl ListRow {
    pub(crate) fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            event_created_at: row.get("event_created_at")?,
            last_checked_at: row.get("last_checked_at")?,
        })
    }
}
