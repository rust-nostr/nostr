use diesel::dsl::{AsSelect, Eq, Filter as DieselFilter, InnerJoin, IntoBoxed, SqlTypeOf};
use diesel::expression::SqlLiteral;
use diesel::prelude::*;
use diesel::sql_types::Binary;
use nostr::event::*;
use nostr::filter::Filter;
use nostr_database::*;

use super::model::EventDb;
#[cfg(feature = "mysql")]
use super::schema::mysql::{event_tags, events};
#[cfg(feature = "postgres")]
use super::schema::postgres::{event_tags, events};
#[cfg(feature = "sqlite")]
use super::schema::sqlite::{event_tags, events};

// filter type of a join query.
type QuerySetJoinTypeDb<'a, DB> = IntoBoxed<
    'a,
    DieselFilter<
        InnerJoin<events::table, event_tags::table>,
        Eq<event_tags::event_id, SqlLiteral<Binary>>,
    >,
    DB,
>;
type SelectEventTypeDb<DB> = SqlTypeOf<AsSelect<EventDb, DB>>;
type BoxedEventQueryDb<'a, DB> = events::BoxedQuery<'a, DB, SelectEventTypeDb<DB>>;

#[cfg(feature = "postgres")]
type QuerySetJoinType<'a> = QuerySetJoinTypeDb<'a, diesel::pg::Pg>;
#[cfg(feature = "postgres")]
type BoxedEventQuery<'a> = BoxedEventQueryDb<'a, diesel::pg::Pg>;
#[cfg(feature = "sqlite")]
type QuerySetJoinType<'a> = QuerySetJoinTypeDb<'a, diesel::sqlite::Sqlite>;
#[cfg(feature = "sqlite")]
type BoxedEventQuery<'a> = BoxedEventQueryDb<'a, diesel::sqlite::Sqlite>;
#[cfg(feature = "mysql")]
type QuerySetJoinType<'a> = QuerySetJoinTypeDb<'a, diesel::mysql::Mysql>;
#[cfg(feature = "mysql")]
type BoxedEventQuery<'a> = BoxedEventQueryDb<'a, diesel::mysql::Mysql>;

pub fn build_filter_query<'a>(filter: Filter) -> QuerySetJoinType<'a> {
    let mut query = events::table
        .distinct()
        .inner_join(event_tags::table)
        .filter(events::deleted.eq(false))
        .order_by(events::created_at.desc())
        .into_boxed();

    if let Some(limit) = filter.limit {
        query = query.limit(limit as i64);
    }

    if !has_filters(&filter) {
        return query;
    }

    if let Some(ids) = filter.ids.clone() {
        let values = ids
            .iter()
            .map(|id| id.as_bytes().to_vec())
            .collect::<Vec<_>>();
        query = query.filter(events::id.eq_any(values));
    }

    if let Some(authors) = filter.authors.clone() {
        let values = authors
            .iter()
            .map(|a| a.as_bytes().to_vec())
            .collect::<Vec<_>>();
        query = query.filter(events::pubkey.eq_any(values));
    }

    if let Some(kinds) = filter.kinds.clone() {
        let values = kinds.iter().map(|k| k.as_u16() as i64).collect::<Vec<_>>();
        query = query.filter(events::kind.eq_any(values));
    }

    if let Some(since) = filter.since {
        query = query.filter(events::created_at.ge(since.as_u64() as i64));
    }

    if let Some(until) = filter.until {
        query = query.filter(events::created_at.le(until.as_u64() as i64));
    }

    if !filter.generic_tags.is_empty() {
        for (tag, values) in filter.generic_tags.into_iter() {
            let values = values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
            query = query.filter(
                event_tags::tag
                    .eq(tag.to_string())
                    .and(event_tags::tag_value.eq_any(values)),
            );
        }
    }

    query
}

/// sets the given default limit on a Nostr filter if not set
pub fn with_limit(filter: Filter, default_limit: usize) -> Filter {
    if filter.limit.is_none() {
        return filter.limit(default_limit);
    }
    filter
}

pub fn event_by_id<'a>(event_id: &EventId) -> BoxedEventQuery<'a> {
    let event_id = event_id.as_bytes().to_vec();
    events::table
        .select(EventDb::as_select())
        .filter(events::id.eq(event_id))
        .into_boxed()
}

// determine if the filter has any filters set
fn has_filters(filter: &Filter) -> bool {
    filter.ids.is_some()
        || filter.authors.is_some()
        || filter.kinds.is_some()
        || filter.since.is_some()
        || filter.until.is_some()
        || !filter.generic_tags.is_empty()
        || filter.limit.is_some()
}
