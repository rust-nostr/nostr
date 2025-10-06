use sqlx::FromRow;

#[derive(FromRow)]
pub(super) struct PublicKeyRow {
    pub(super) last_nip17_update: Option<i64>,
    pub(super) last_nip65_update: Option<i64>,
}
