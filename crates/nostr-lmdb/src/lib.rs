// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! LMDB storage backend for nostr apps
//!
//! Fork of [Pocket](https://github.com/mikedilger/pocket) database.

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(clippy::mutable_key_type)]

use std::collections::HashSet;
use std::path::Path;

use nostr_database::prelude::*;

mod store;

use self::store::Store;

// pub struct LmdbTransaction<'a> {
//     db: Store,
//     txn: RoTxn<'a>,
// }

/// LMDB Nostr Database
#[derive(Debug)]
pub struct NostrLMDB {
    db: Store,
    // TODO: Temporary use memory database to store seen event IDs
    // until decide if continue to store them in `NostrDatabase`
    // or somewhere else
    temp: MemoryDatabase,
}

impl NostrLMDB {
    /// Open LMDB database
    #[inline]
    pub fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            db: Store::open(path).map_err(DatabaseError::backend)?,
            temp: MemoryDatabase::with_opts(MemoryDatabaseOptions {
                events: false,
                max_events: Some(100_000),
            }),
        })
    }
}

#[async_trait]
impl NostrDatabase for NostrLMDB {
    #[inline]
    fn backend(&self) -> Backend {
        Backend::LMDB
    }

    #[inline]
    async fn wipe(&self) -> Result<(), DatabaseError> {
        self.db.wipe().await.map_err(DatabaseError::backend)
    }
}

// #[async_trait]
// impl<'lmdb> NostrEventsDatabaseTransaction for LmdbTransaction<'lmdb> {
//     async fn query<'a>(&'a self, filters: Vec<Filter>) -> Result<QueryEvents<'a>, DatabaseError> {
//         self.db.query()
//         Ok(QueryEvents::Set(events))
//     }
// }

#[async_trait]
impl NostrEventsDatabase for NostrLMDB {
    #[inline]
    async fn save_event(&self, event: &Event) -> Result<SaveEventStatus, DatabaseError> {
        self.db
            .save_event(event)
            .await
            .map_err(DatabaseError::backend)
    }

    async fn check_id(&self, event_id: &EventId) -> Result<DatabaseEventStatus, DatabaseError> {
        if self
            .db
            .event_is_deleted(*event_id)
            .await
            .map_err(DatabaseError::backend)?
        {
            Ok(DatabaseEventStatus::Deleted)
        } else if self
            .db
            .has_event(event_id)
            .await
            .map_err(DatabaseError::backend)?
        {
            Ok(DatabaseEventStatus::Saved)
        } else {
            Ok(DatabaseEventStatus::NotExistent)
        }
    }

    async fn has_coordinate_been_deleted(
        &self,
        coordinate: &Coordinate,
        timestamp: &Timestamp,
    ) -> Result<bool, DatabaseError> {
        if let Some(t) = self
            .db
            .when_is_coordinate_deleted(coordinate.clone())
            .await
            .map_err(DatabaseError::backend)?
        {
            Ok(&t >= timestamp)
        } else {
            Ok(false)
        }
    }

    #[inline]
    async fn event_id_seen(
        &self,
        event_id: EventId,
        relay_url: RelayUrl,
    ) -> Result<(), DatabaseError> {
        self.temp.event_id_seen(event_id, relay_url).await
    }

    #[inline]
    async fn event_seen_on_relays(
        &self,
        event_id: &EventId,
    ) -> Result<Option<HashSet<RelayUrl>>, DatabaseError> {
        self.temp.event_seen_on_relays(event_id).await
    }

    #[inline]
    async fn event_by_id(&self, event_id: &EventId) -> Result<Option<Event>, DatabaseError> {
        self.db
            .get_event_by_id(event_id)
            .await
            .map_err(DatabaseError::backend)
    }

    #[inline]
    async fn count(&self, filters: Vec<Filter>) -> Result<usize, DatabaseError> {
        self.db.count(filters).await.map_err(DatabaseError::backend)
    }

    #[inline]
    async fn query(&self, filters: Vec<Filter>) -> Result<Events, DatabaseError> {
        self.db.query(filters).await.map_err(DatabaseError::backend)
    }

    #[inline]
    async fn negentropy_items(
        &self,
        filter: Filter,
    ) -> Result<Vec<(EventId, Timestamp)>, DatabaseError> {
        self.db
            .negentropy_items(filter)
            .await
            .map_err(DatabaseError::backend)
    }

    #[inline]
    async fn delete(&self, filter: Filter) -> Result<(), DatabaseError> {
        self.db.delete(filter).await.map_err(DatabaseError::backend)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use std::time::Duration;

    use tempfile::TempDir;

    use super::*;

    const EVENTS: [&str; 14] = [
        r#"{"id":"b7b1fb52ad8461a03e949820ae29a9ea07e35bcd79c95c4b59b0254944f62805","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644581,"kind":1,"tags":[],"content":"Text note","sig":"ed73a8a4e7c26cd797a7b875c634d9ecb6958c57733305fed23b978109d0411d21b3e182cb67c8ad750884e30ca383b509382ae6187b36e76ee76e6a142c4284"}"#,
        r#"{"id":"7296747d91c53f1d71778ef3e12d18b66d494a41f688ef244d518abf37c959b6","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644586,"kind":32121,"tags":[["d","id-1"]],"content":"Empty 1","sig":"8848989a8e808f7315e950f871b231c1dff7752048f8957d4a541881d2005506c30e85c7dd74dab022b3e01329c88e69c9d5d55d961759272a738d150b7dbefc"}"#,
        r#"{"id":"ec6ea04ba483871062d79f78927df7979f67545b53f552e47626cb1105590442","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644591,"kind":32122,"tags":[["d","id-1"]],"content":"Empty 2","sig":"89946113a97484850fe35fefdb9120df847b305de1216dae566616fe453565e8707a4da7e68843b560fa22a932f81fc8db2b5a2acb4dcfd3caba9a91320aac92"}"#,
        r#"{"id":"63b8b829aa31a2de870c3a713541658fcc0187be93af2032ec2ca039befd3f70","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644596,"kind":32122,"tags":[["d","id-2"]],"content":"","sig":"607b1a67bef57e48d17df4e145718d10b9df51831d1272c149f2ab5ad4993ae723f10a81be2403ae21b2793c8ed4c129e8b031e8b240c6c90c9e6d32f62d26ff"}"#,
        r#"{"id":"6fe9119c7db13ae13e8ecfcdd2e5bf98e2940ba56a2ce0c3e8fba3d88cd8e69d","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644601,"kind":32122,"tags":[["d","id-3"]],"content":"","sig":"d07146547a726fc9b4ec8d67bbbe690347d43dadfe5d9890a428626d38c617c52e6945f2b7144c4e0c51d1e2b0be020614a5cadc9c0256b2e28069b70d9fc26e"}"#,
        r#"{"id":"a82f6ebfc709f4e7c7971e6bf738e30a3bc112cfdb21336054711e6779fd49ef","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644606,"kind":32122,"tags":[["d","id-1"]],"content":"","sig":"96d3349b42ed637712b4d07f037457ab6e9180d58857df77eb5fa27ff1fd68445c72122ec53870831ada8a4d9a0b484435f80d3ff21a862238da7a723a0d073c"}"#,
        r#"{"id":"8ab0cb1beceeb68f080ec11a3920b8cc491ecc7ec5250405e88691d733185832","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644611,"kind":32122,"tags":[["d","id-1"]],"content":"Test","sig":"49153b482d7110e2538eb48005f1149622247479b1c0057d902df931d5cea105869deeae908e4e3b903e3140632dc780b3f10344805eab77bb54fb79c4e4359d"}"#,
        r#"{"id":"63dc49a8f3278a2de8dc0138939de56d392b8eb7a18c627e4d78789e2b0b09f2","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704644616,"kind":5,"tags":[["a","32122:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:"]],"content":"","sig":"977e54e5d57d1fbb83615d3a870037d9eb5182a679ca8357523bbf032580689cf481f76c88c7027034cfaf567ba9d9fe25fc8cd334139a0117ad5cf9fe325eef"}"#,
        r#"{"id":"6975ace0f3d66967f330d4758fbbf45517d41130e2639b54ca5142f37757c9eb","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704644621,"kind":5,"tags":[["a","32122:aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4:id-2"]],"content":"","sig":"9bb09e4759899d86e447c3fa1be83905fe2eda74a5068a909965ac14fcdabaed64edaeb732154dab734ca41f2fc4d63687870e6f8e56e3d9e180e4a2dd6fb2d2"}"#,
        r#"{"id":"33f5b4e6a38e107638c20f4536db35191d4b8651ba5a2cefec983b9ec2d65084","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704645586,"kind":0,"tags":[],"content":"{\"name\":\"Key A\"}","sig":"285d090f45a6adcae717b33771149f7840a8c27fb29025d63f1ab8d95614034a54e9f4f29cee9527c4c93321a7ebff287387b7a19ba8e6f764512a40e7120429"}"#,
        r#"{"id":"90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704645606,"kind":0,"tags":[],"content":"{\"name\":\"key-a\",\"display_name\":\"Key A\",\"lud16\":\"keya@ln.address\"}","sig":"ec8f49d4c722b7ccae102d49befff08e62db775e5da43ef51b25c47dfdd6a09dc7519310a3a63cbdb6ec6b3250e6f19518eb47be604edeb598d16cdc071d3dbc"}"#,
        r#"{"id":"a295422c636d3532875b75739e8dae3cdb4dd2679c6e4994c9a39c7ebf8bc620","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704646569,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"d4dc8368a4ad27eef63cacf667345aadd9617001537497108234fc1686d546c949cbb58e007a4d4b632c65ea135af4fbd7a089cc60ab89b6901f5c3fc6a47b29"}"#, // Invalid event deletion
        r#"{"id":"999e3e270100d7e1eaa98fcfab4a98274872c1f2dfdab024f32e42a5a12d5b5e","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704646606,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"4f3a33fd52784cea7ca8428fd35d94d65049712e9aa11a70b1a16a1fcd761c7b7e27afac325728b1c00dfa11e33e78b2efd0430a7e4b28f4ede5b579b3f32614"}"#,
        r#"{"id":"99a022e6d61c4e39c147d08a2be943b664e8030c0049325555ac1766429c2832","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1705241093,"kind":30333,"tags":[["d","multi-id"],["p","aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4"]],"content":"Multi-tags","sig":"0abfb2b696a7ed7c9e8e3bf7743686190f3f1b3d4045b72833ab6187c254f7ed278d289d52dfac3de28be861c1471421d9b1bfb5877413cbc81c84f63207a826"}"#,
    ];

    struct TempDatabase {
        db: NostrLMDB,
        // Needed to avoid the drop and deletion of temp folder
        _temp: TempDir,
    }

    impl Deref for TempDatabase {
        type Target = NostrLMDB;

        fn deref(&self) -> &Self::Target {
            &self.db
        }
    }

    impl TempDatabase {
        fn new() -> Self {
            let path = tempfile::tempdir().unwrap();
            Self {
                db: NostrLMDB::open(&path).unwrap(),
                _temp: path,
            }
        }

        // Return the number of added events
        async fn add_random_events(&self) -> usize {
            let keys_a = Keys::generate();
            let keys_b = Keys::generate();

            let events = vec![
                EventBuilder::text_note("Text Note A")
                    .sign_with_keys(&keys_a)
                    .unwrap(),
                EventBuilder::text_note("Text Note B")
                    .sign_with_keys(&keys_b)
                    .unwrap(),
                EventBuilder::metadata(
                    &Metadata::new().name("account-a").display_name("Account A"),
                )
                .sign_with_keys(&keys_a)
                .unwrap(),
                EventBuilder::metadata(
                    &Metadata::new().name("account-b").display_name("Account B"),
                )
                .sign_with_keys(&keys_b)
                .unwrap(),
                EventBuilder::new(Kind::Custom(33_333), "")
                    .tag(Tag::identifier("my-id-a"))
                    .sign_with_keys(&keys_a)
                    .unwrap(),
                EventBuilder::new(Kind::Custom(33_333), "")
                    .tag(Tag::identifier("my-id-b"))
                    .sign_with_keys(&keys_b)
                    .unwrap(),
            ];

            // Store
            for event in events.iter() {
                self.db.save_event(event).await.unwrap();
            }

            events.len()
        }

        async fn add_event(&self, builder: EventBuilder) -> (Keys, Event) {
            let keys = Keys::generate();
            let event = builder.sign_with_keys(&keys).unwrap();
            self.db.save_event(&event).await.unwrap();
            (keys, event)
        }

        async fn add_event_with_keys(
            &self,
            builder: EventBuilder,
            keys: &Keys,
        ) -> (Event, SaveEventStatus) {
            let event = builder.sign_with_keys(keys).unwrap();
            let status = self.db.save_event(&event).await.unwrap();
            (event, status)
        }

        async fn count_all(&self) -> usize {
            self.db.count(vec![Filter::new()]).await.unwrap()
        }
    }

    #[tokio::test]
    async fn test_event_by_id() {
        let db = TempDatabase::new();

        let added_events: usize = db.add_random_events().await;

        let (_keys, expected_event) = db.add_event(EventBuilder::text_note("Test")).await;

        let event = db.event_by_id(&expected_event.id).await.unwrap().unwrap();
        assert_eq!(event, expected_event);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1)
    }

    #[tokio::test]
    async fn test_replaceable_event() {
        let db = TempDatabase::new();

        let added_events: usize = db.add_random_events().await;

        let now = Timestamp::now();
        let metadata = Metadata::new()
            .name("my-account")
            .display_name("My Account");

        let (keys, expected_event) = db
            .add_event(
                EventBuilder::metadata(&metadata).custom_created_at(now - Duration::from_secs(120)),
            )
            .await;

        // Test event by ID
        let event = db.event_by_id(&expected_event.id).await.unwrap().unwrap();
        assert_eq!(event, expected_event);

        // Test filter query
        let events = db
            .query(vec![Filter::new()
                .author(keys.public_key)
                .kind(Kind::Metadata)])
            .await
            .unwrap();
        assert_eq!(events.to_vec(), vec![expected_event.clone()]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);

        // Replace previous event
        let (new_expected_event, status) = db
            .add_event_with_keys(
                EventBuilder::metadata(&metadata).custom_created_at(now),
                &keys,
            )
            .await;
        assert!(status.is_success());

        // Test event by ID (MUST be None because replaced)
        assert!(db.event_by_id(&expected_event.id).await.unwrap().is_none());

        // Test event by ID
        let event = db
            .event_by_id(&new_expected_event.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event, new_expected_event);

        // Test filter query
        let events = db
            .query(vec![Filter::new()
                .author(keys.public_key)
                .kind(Kind::Metadata)])
            .await
            .unwrap();
        assert_eq!(events.to_vec(), vec![new_expected_event]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);
    }

    #[tokio::test]
    async fn test_param_replaceable_event() {
        let db = TempDatabase::new();

        let added_events: usize = db.add_random_events().await;

        let now = Timestamp::now();

        let (keys, expected_event) = db
            .add_event(
                EventBuilder::new(Kind::Custom(33_333), "")
                    .tag(Tag::identifier("my-id-a"))
                    .custom_created_at(now - Duration::from_secs(120)),
            )
            .await;
        let coordinate = Coordinate::new(Kind::from(33_333), keys.public_key).identifier("my-id-a");

        // Test event by ID
        let event = db.event_by_id(&expected_event.id).await.unwrap().unwrap();
        assert_eq!(event, expected_event);

        // Test filter query
        let events = db.query(vec![coordinate.clone().into()]).await.unwrap();
        assert_eq!(events.to_vec(), vec![expected_event.clone()]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);

        // Replace previous event
        let (new_expected_event, status) = db
            .add_event_with_keys(
                EventBuilder::new(Kind::Custom(33_333), "Test replace")
                    .tag(Tag::identifier("my-id-a"))
                    .custom_created_at(now),
                &keys,
            )
            .await;
        assert!(status.is_success());

        // Test event by ID (MUST be None` because replaced)
        assert!(db.event_by_id(&expected_event.id).await.unwrap().is_none());

        // Test event by ID
        let event = db
            .event_by_id(&new_expected_event.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event, new_expected_event);

        // Test filter query
        let events = db.query(vec![coordinate.into()]).await.unwrap();
        assert_eq!(events.to_vec(), vec![new_expected_event]);

        // Check if number of events in database match the expected
        assert_eq!(db.count_all().await, added_events + 1);

        // Trey to add param replaceable event with older timestamp (MUSTN'T be stored)
        let (_, status) = db
            .add_event_with_keys(
                EventBuilder::new(Kind::Custom(33_333), "Test replace 2")
                    .tag(Tag::identifier("my-id-a"))
                    .custom_created_at(now - Duration::from_secs(2000)),
                &keys,
            )
            .await;
        assert!(!status.is_success());
    }

    #[tokio::test]
    async fn test_full_text_search() {
        let db = TempDatabase::new();

        let _added_events: usize = db.add_random_events().await;

        let events = db
            .query(vec![Filter::new().search("Account A")])
            .await
            .unwrap();
        assert_eq!(events.len(), 1);

        let events = db
            .query(vec![Filter::new().search("account a")])
            .await
            .unwrap();
        assert_eq!(events.len(), 1);

        let events = db
            .query(vec![Filter::new().search("text note")])
            .await
            .unwrap();
        assert_eq!(events.len(), 2);

        let events = db.query(vec![Filter::new().search("notes")]).await.unwrap();
        assert_eq!(events.len(), 0);

        let events = db.query(vec![Filter::new().search("hola")]).await.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_expected_query_result() {
        let db = TempDatabase::new();

        for event in EVENTS.into_iter() {
            let event = Event::from_json(event).unwrap();
            let _ = db.save_event(&event).await;
        }

        // Test expected output
        let expected_output = vec![
            Event::from_json(EVENTS[13]).unwrap(),
            Event::from_json(EVENTS[12]).unwrap(),
            // Event 11 is invalid deletion
            // Event 10 deleted by event 12
            // Event 9 replaced by event 10
            Event::from_json(EVENTS[8]).unwrap(),
            // Event 7 is an invalid deletion
            Event::from_json(EVENTS[6]).unwrap(),
            Event::from_json(EVENTS[5]).unwrap(),
            Event::from_json(EVENTS[4]).unwrap(),
            // Event 3 deleted by Event 8
            // Event 2 replaced by Event 6
            Event::from_json(EVENTS[1]).unwrap(),
            Event::from_json(EVENTS[0]).unwrap(),
        ];
        assert_eq!(
            db.query(vec![Filter::new()]).await.unwrap().to_vec(),
            expected_output
        );
        assert_eq!(db.count_all().await, 8);
    }

    #[tokio::test]
    async fn test_delete_events_with_filter() {
        let db = TempDatabase::new();

        let added_events: usize = db.add_random_events().await;

        assert_eq!(db.count_all().await, added_events);

        // Delete all kinds except text note
        let filter = Filter::new().kinds([Kind::Metadata, Kind::Custom(33_333)]);
        db.delete(filter).await.unwrap();

        assert_eq!(db.count_all().await, 2);
    }
}
