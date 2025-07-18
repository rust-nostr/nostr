// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

//! # NostrLMDB
//!
//! A Nostr database implementation using LMDB.
//!
//! Fork of [Pocket](https://github.com/mikedilger/pocket) database.

#![warn(missing_docs)]
#![warn(rustdoc::bare_urls)]
#![allow(unknown_lints)]

use std::path::{Path, PathBuf};

use nostr::prelude::*;
use nostr::util::BoxedFuture;
use nostr_database::{
    Backend, DatabaseError, DatabaseEventStatus, Events, NostrDatabase, SaveEventStatus,
};

mod store;

use self::store::Store;

// 64-bit
#[cfg(target_pointer_width = "64")]
const MAP_SIZE: usize = 1024 * 1024 * 1024 * 32; // 32GB

// 32-bit
#[cfg(target_pointer_width = "32")]
const MAP_SIZE: usize = 0xFFFFF000; // 4GB (2^32-4096)

/// Nostr LMDB database builder
#[derive(Debug, Clone)]
pub struct NostrLmdbBuilder {
    /// Database path
    pub path: PathBuf,
    /// Custom map size
    ///
    /// By default, the following map size is used:
    /// - 32GB for 64-bit arch
    /// - 4GB for 32-bit arch
    pub map_size: Option<usize>,
    /// Maximum number of readers
    pub max_readers: Option<u32>,
    /// Maximum number of named databases
    pub max_dbs: Option<u32>,
}

impl NostrLmdbBuilder {
    /// New LMDb builder
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            map_size: None,
            max_readers: None,
            max_dbs: None,
        }
    }

    /// Custom map size
    pub fn map_size(mut self, map_size: usize) -> Self {
        self.map_size = Some(map_size);
        self
    }

    /// Maximum number of readers
    pub fn max_readers(mut self, readers: u32) -> Self {
        self.max_readers = Some(readers);
        self
    }

    /// Maximum number of named databases
    pub fn max_dbs(mut self, dbs: u32) -> Self {
        self.max_dbs = Some(dbs);
        self
    }

    /// Build
    pub fn build(self) -> Result<NostrLMDB, DatabaseError> {
        let map_size: usize = self.map_size.unwrap_or(MAP_SIZE);
        // LMDB defaults: max_readers=126, max_dbs=0
        // We use 126 readers (LMDB default) and 20 dbs (reasonable for our use case)
        let max_readers: u32 = self.max_readers.unwrap_or(126);
        let max_dbs: u32 = self.max_dbs.unwrap_or(10);

        let db: Store = Store::open(self.path, map_size, max_readers, max_dbs)
            .map_err(DatabaseError::backend)?;

        Ok(NostrLMDB { db })
    }
}

/// LMDB Nostr Database
#[derive(Debug)]
pub struct NostrLMDB {
    db: Store,
}

impl NostrLMDB {
    /// Open LMDB database with default configuration
    #[inline]
    pub fn open<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        Self::builder(path).build()
    }

    /// Get a new builder
    #[inline]
    pub fn builder<P>(path: P) -> NostrLmdbBuilder
    where
        P: AsRef<Path>,
    {
        NostrLmdbBuilder::new(path)
    }
}

impl NostrDatabase for NostrLMDB {
    #[inline]
    fn backend(&self) -> Backend {
        Backend::LMDB
    }
    fn save_event<'a>(
        &'a self,
        event: &'a Event,
    ) -> BoxedFuture<'a, Result<SaveEventStatus, DatabaseError>> {
        Box::pin(async move {
            self.db
                .save_event(event)
                .await
                .map_err(DatabaseError::backend)
        })
    }

    fn check_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<DatabaseEventStatus, DatabaseError>> {
        Box::pin(async move {
            if self
                .db
                .has_event(event_id)
                .map_err(DatabaseError::backend)?
            {
                if self
                    .db
                    .event_is_deleted(event_id)
                    .map_err(DatabaseError::backend)?
                {
                    Ok(DatabaseEventStatus::Deleted)
                } else {
                    Ok(DatabaseEventStatus::Saved)
                }
            } else {
                Ok(DatabaseEventStatus::NotExistent)
            }
        })
    }

    fn event_by_id<'a>(
        &'a self,
        event_id: &'a EventId,
    ) -> BoxedFuture<'a, Result<Option<Event>, DatabaseError>> {
        Box::pin(async move {
            self.db
                .get_event_by_id(event_id)
                .map_err(DatabaseError::backend)
        })
    }

    fn count(&self, filter: Filter) -> BoxedFuture<Result<usize, DatabaseError>> {
        Box::pin(async move { self.db.count(filter).map_err(DatabaseError::backend) })
    }

    fn query(&self, filter: Filter) -> BoxedFuture<Result<Events, DatabaseError>> {
        Box::pin(async move { self.db.query(filter).map_err(DatabaseError::backend) })
    }

    fn negentropy_items(
        &self,
        filter: Filter,
    ) -> BoxedFuture<Result<Vec<(EventId, Timestamp)>, DatabaseError>> {
        Box::pin(async move {
            self.db
                .negentropy_items(filter)
                .map_err(DatabaseError::backend)
        })
    }

    fn delete(&self, filter: Filter) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { self.db.delete(filter).await.map_err(DatabaseError::backend) })
    }

    fn wipe(&self) -> BoxedFuture<Result<(), DatabaseError>> {
        Box::pin(async move { self.db.wipe().map_err(DatabaseError::backend) })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nostr::nips::nip01::Coordinate;
    use nostr::nips::nip09::EventDeletionRequest;
    use nostr::{
        Event, EventBuilder, EventId, Filter, JsonUtil, Keys, Kind, Metadata, Tag, Timestamp,
    };
    use tempfile::TempDir;

    use crate::{DatabaseEventStatus, NostrDatabase, NostrLMDB, SaveEventStatus};

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
        r#"{"id":"a295422c636d3532875b75739e8dae3cdb4dd2679c6e4994c9a39c7ebf8bc620","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1704646569,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"d4dc8368a4ad27eef63cacf667345aadd9617001537497108234fc1686d546c949cbb58e007a4d4b632c65ea135af4fbd7a089cc60ab89b6901f5c3fc6a47b29"}"#,
        r#"{"id":"999e3e270100d7e1eaa98fcfab4a98274872c1f2dfdab024f32e42a5a12d5b5e","pubkey":"aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4","created_at":1704646606,"kind":5,"tags":[["e","90a761aec9b5b60b399a76826141f529db17466deac85696a17e4a243aa271f9"]],"content":"","sig":"4f3a33fd52784cea7ca8428fd35d94d65049712e9aa11a70b1a16a1fcd761c7b7e27afac325728b1c00dfa11e33e78b2efd0430a7e4b28f4ede5b579b3f32614"}"#,
        r#"{"id":"99a022e6d61c4e39c147d08a2be943b664e8030c0049325555ac1766429c2832","pubkey":"79dff8f82963424e0bb02708a22e44b4980893e3a4be0fa3cb60a43b946764e3","created_at":1705241093,"kind":30333,"tags":[["d","multi-id"],["p","aa4fc8665f5696e33db7e1a572e3b0f5b3d615837b0f362dcb1c8068b098c7b4"]],"content":"Multi-tags","sig":"0abfb2b696a7ed7c9e8e3bf7743686190f3f1b3d4045b72833ab6187c254f7ed278d289d52dfac3de28be861c1471421d9b1bfb5877413cbc81c84f63207a826"}"#,
    ];

    fn decode_events() -> Vec<Event> {
        EVENTS
            .iter()
            .map(|e| Event::from_json(e).expect("Failed to parse event"))
            .collect()
    }

    async fn setup_db() -> (NostrLMDB, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db = NostrLMDB::open(temp_dir.path()).expect("Failed to open database");
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_query() {
        let (db, _temp_dir) = setup_db().await;
        let events = decode_events();

        // Save all events (some will be rejected due to invalid deletions)
        for (i, event) in events.iter().enumerate() {
            let status = db.save_event(event).await.expect("Failed to save event");
            if i == 7 || i == 11 {
                // These should be rejected for invalid deletions
                assert!(!status.is_success());
            } else {
                assert!(matches!(status, SaveEventStatus::Success));
            }

            // NOTE: Sleep prevents automatic batching - events in the same batch share
            // a database snapshot and can't see each other's changes. Deletion events
            // (7,11) must "see" target events, and replaceable events must observe
            // earlier events to replace them. Sleep forces sequential processing.
            // Use this pattern when event N must observe changes from event N-1.
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Query all events
        let saved_events = db.query(Filter::new()).await.expect("Failed to query");
        // Expected: 8 events after applying coordinate deletion validation
        assert_eq!(saved_events.len(), 8);
    }

    #[tokio::test]
    async fn test_save_duplicate() {
        let (db, _temp_dir) = setup_db().await;
        let events = decode_events();
        let event = &events[0];

        // Save event first time
        let status = db.save_event(event).await.expect("Failed to save event");
        assert!(matches!(status, SaveEventStatus::Success));

        // Try to save again
        let status = db.save_event(event).await.expect("Failed to save event");
        assert!(matches!(
            status,
            SaveEventStatus::Rejected(nostr_database::RejectedReason::Duplicate)
        ));
    }

    #[tokio::test]
    async fn test_query_by_filter() {
        let (db, _temp_dir) = setup_db().await;
        let events = decode_events();

        // Save all events
        for event in &events {
            db.save_event(event).await.expect("Failed to save event");
        }

        // Query by author
        let author_filter = Filter::new().author(events[0].pubkey);
        let author_events = db.query(author_filter).await.expect("Failed to query");
        assert!(!author_events.is_empty());
        assert!(author_events.iter().all(|e| e.pubkey == events[0].pubkey));

        // Query by kind
        let kind_filter = Filter::new().kind(Kind::TextNote);
        let kind_events = db.query(kind_filter).await.expect("Failed to query");
        assert!(!kind_events.is_empty());
        assert!(kind_events.iter().all(|e| e.kind == Kind::TextNote));

        // Query by time range
        let since = Timestamp::from_secs(1704644590);
        let until = Timestamp::from_secs(1704644620);
        let time_filter = Filter::new().since(since).until(until);
        let time_events = db.query(time_filter).await.expect("Failed to query");
        assert!(!time_events.is_empty());
        assert!(time_events
            .iter()
            .all(|e| e.created_at >= since && e.created_at <= until));
    }

    #[tokio::test]
    async fn test_delete_by_filter() {
        let (db, _temp_dir) = setup_db().await;
        let events = decode_events();

        // Save all events
        for event in &events {
            db.save_event(event).await.expect("Failed to save event");
        }

        // Count before delete (8 visible after processing deletions/replacements)
        let count_before = db
            .count(Filter::new())
            .await
            .expect("Failed to count events");
        assert_eq!(count_before, 8);

        // Delete text notes
        let delete_filter = Filter::new().kind(Kind::TextNote);
        db.delete(delete_filter)
            .await
            .expect("Failed to delete events");

        // Count after delete (text notes: indices 0,4,13 - but 0 is deleted = 2 visible text notes deleted)
        let count_after = db
            .count(Filter::new())
            .await
            .expect("Failed to count events");
        assert_eq!(count_after, 7); // 8 - 1 text note = 7

        // Verify no text notes remain
        let text_notes = db
            .query(Filter::new().kind(Kind::TextNote))
            .await
            .expect("Failed to query");
        assert_eq!(text_notes.len(), 0);
    }

    #[tokio::test]
    async fn test_replaceable_events() {
        let (db, _temp_dir) = setup_db().await;
        let keys = Keys::generate();

        // Create first replaceable event (kind 0 - metadata)
        let metadata1 = Metadata::new().name("First");
        let event1 = EventBuilder::metadata(&metadata1)
            .custom_created_at(Timestamp::from_secs(1000))
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        db.save_event(&event1).await.expect("Failed to save event");

        // Create newer replaceable event with later timestamp
        let metadata2 = Metadata::new().name("Second");
        let event2 = EventBuilder::metadata(&metadata2)
            .custom_created_at(Timestamp::from_secs(2000))
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        db.save_event(&event2).await.expect("Failed to save event");

        // Query metadata events
        let filter = Filter::new().author(keys.public_key()).kind(Kind::Metadata);
        let results = db.query(filter).await.expect("Failed to query");

        // Should only have the newer event
        assert_eq!(results.len(), 1);
        // Verify it's the newer event by content
        let result_event = results.first().unwrap();
        assert!(result_event.content.contains("Second"));
    }

    #[tokio::test]
    async fn test_addressable_events() {
        let (db, _temp_dir) = setup_db().await;
        let keys = Keys::generate();

        // Create first addressable event
        let event1 = EventBuilder::new(Kind::from(32121), "Content 1")
            .tag(Tag::identifier("test-id"))
            .custom_created_at(Timestamp::from_secs(1000))
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        db.save_event(&event1).await.expect("Failed to save event");

        // Create newer addressable event with same identifier
        let event2 = EventBuilder::new(Kind::from(32121), "Content 2")
            .tag(Tag::identifier("test-id"))
            .custom_created_at(Timestamp::from_secs(2000))
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        db.save_event(&event2).await.expect("Failed to save event");

        // Query addressable events
        let filter = Filter::new()
            .author(keys.public_key())
            .kind(Kind::from(32121));
        let results = db.query(filter).await.expect("Failed to query");

        // Should only have the newer event
        assert_eq!(results.len(), 1);
        // Verify it's the newer event by content
        let result_event = results.first().unwrap();
        assert_eq!(result_event.content, "Content 2");
    }

    #[tokio::test]
    async fn test_event_deletion() {
        let (db, _temp_dir) = setup_db().await;
        let keys = Keys::generate();

        // Create events to delete
        let event1 = EventBuilder::text_note("To be deleted 1")
            .sign_with_keys(&keys)
            .expect("Failed to sign");
        let event2 = EventBuilder::text_note("To be deleted 2")
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        db.save_event(&event1).await.expect("Failed to save event");
        db.save_event(&event2).await.expect("Failed to save event");

        // Create deletion event
        let deletion =
            EventBuilder::delete(EventDeletionRequest::new().id(event1.id).id(event2.id))
                .sign_with_keys(&keys)
                .expect("Failed to sign");

        db.save_event(&deletion)
            .await
            .expect("Failed to save deletion");

        // Check events are marked as deleted
        let status1 = db
            .check_id(&event1.id)
            .await
            .expect("Failed to check event");
        let status2 = db
            .check_id(&event2.id)
            .await
            .expect("Failed to check event");

        // Events should be marked as deleted after processing deletion event
        assert_eq!(status1, DatabaseEventStatus::Deleted);
        assert_eq!(status2, DatabaseEventStatus::Deleted);
    }

    #[tokio::test]
    async fn test_wipe_database() {
        let (db, _temp_dir) = setup_db().await;
        let events = decode_events();

        // Save all events
        for event in &events {
            db.save_event(event).await.expect("Failed to save event");
        }

        // Verify events exist (7 visible after processing)
        let count = db
            .count(Filter::new())
            .await
            .expect("Failed to count events");
        assert_eq!(count, 8);

        // Wipe database
        db.wipe().await.expect("Failed to wipe database");

        // Verify database is empty
        let count_after = db
            .count(Filter::new())
            .await
            .expect("Failed to count events");
        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_negentropy_items() {
        let (db, _temp_dir) = setup_db().await;
        let events = decode_events();

        // Save all events
        for event in &events {
            db.save_event(event).await.expect("Failed to save event");
        }

        // Get negentropy items (7 visible events)
        let items = db
            .negentropy_items(Filter::new())
            .await
            .expect("Failed to get negentropy items");

        assert_eq!(items.len(), 8);

        // Verify items are from the original events
        let event_ids: std::collections::HashSet<EventId> = events.iter().map(|e| e.id).collect();

        for (id, _timestamp) in items {
            assert!(
                event_ids.contains(&id),
                "Unexpected event ID in negentropy items"
            );
        }
    }

    // Test infrastructure from upstream/master
    struct TempDatabase {
        db: NostrLMDB,
        // Needed to avoid the drop and deletion of temp folder
        _temp: TempDir,
    }

    impl std::ops::Deref for TempDatabase {
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
            self.db.count(Filter::new()).await.unwrap()
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
            .query(Filter::new().author(keys.public_key).kind(Kind::Metadata))
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
            .query(Filter::new().author(keys.public_key).kind(Kind::Metadata))
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
        let events = db.query(coordinate.clone().into()).await.unwrap();
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
        let events = db.query(coordinate.into()).await.unwrap();
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

        let events = db.query(Filter::new().search("Account A")).await.unwrap();
        assert_eq!(events.len(), 1);

        let events = db.query(Filter::new().search("account a")).await.unwrap();
        assert_eq!(events.len(), 1);

        let events = db.query(Filter::new().search("text note")).await.unwrap();
        assert_eq!(events.len(), 2);

        let events = db.query(Filter::new().search("notes")).await.unwrap();
        assert_eq!(events.len(), 0);

        let events = db.query(Filter::new().search("hola")).await.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_expected_query_result() {
        let db = TempDatabase::new();

        // Save events individually to avoid batching issues during test
        for (i, event_str) in EVENTS.into_iter().enumerate() {
            let event = Event::from_json(event_str).unwrap();
            let status = db.save_event(&event).await.unwrap();
            println!(
                "Event {}: {} - Kind: {:?}, Status: {:?}",
                i, event.id, event.kind, status
            );

            // Invalid deletions (Event 7 and 11) should be rejected
            if i == 7 || i == 11 {
                assert!(!status.is_success(), "Event {} should be rejected", i);
            }

            // Add a small delay to ensure each event is processed individually
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Expected output after applying NIP-09 deletion validation
        // Events 7 and 11 are rejected for invalid deletion attempts
        let expected_output = vec![
            Event::from_json(EVENTS[13]).unwrap(), // Kind:30333 latest
            Event::from_json(EVENTS[12]).unwrap(), // Kind:5 deletion
            Event::from_json(EVENTS[8]).unwrap(),  // Kind:5 coordinate deletion
            Event::from_json(EVENTS[6]).unwrap(),  // Kind:32122 latest
            Event::from_json(EVENTS[5]).unwrap(),  // Kind:32122 from different author
            Event::from_json(EVENTS[4]).unwrap(),  // Kind:32122 from different author
            Event::from_json(EVENTS[1]).unwrap(),  // Kind:32121
            Event::from_json(EVENTS[0]).unwrap(),  // Kind:1 text note
        ];

        let actual = db.query(Filter::new()).await.unwrap().to_vec();

        // Debug: print which events are missing from expected
        println!(
            "Actual has {} events, expected has {} events",
            actual.len(),
            expected_output.len()
        );
        for event in &expected_output {
            if !actual.iter().any(|e| e.id == event.id) {
                println!("Expected event {} is missing from actual", event.id);
            }
        }

        assert_eq!(actual, expected_output);
        assert_eq!(db.count_all().await, 8); // 8 events after deletion validation
    }

    #[tokio::test]
    async fn test_kind5_deletion_query_bug_fix() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db = NostrLMDB::open(temp_dir.path()).unwrap();
        let keys = Keys::generate();

        // Create and save an event
        let event = EventBuilder::text_note("Test event")
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        let status = db.save_event(&event).await.expect("Failed to save event");
        assert!(matches!(status, SaveEventStatus::Success));

        // Sleep to ensure it's committed
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify it exists with ID filter
        let before_by_id = db
            .query(Filter::new().id(event.id))
            .await
            .expect("Failed to query");
        assert_eq!(before_by_id.len(), 1);

        // Verify it exists with author-kind filter
        let before_by_author = db
            .query(Filter::new().author(keys.public_key()).kind(Kind::TextNote))
            .await
            .expect("Failed to query");
        assert_eq!(before_by_author.len(), 1);

        // Create and save a Kind 5 deletion event
        let deletion_event = EventBuilder::new(Kind::EventDeletion, "")
            .tag(Tag::event(event.id))
            .sign_with_keys(&keys)
            .expect("Failed to sign");

        let del_status = db
            .save_event(&deletion_event)
            .await
            .expect("Failed to save deletion");
        assert!(matches!(del_status, SaveEventStatus::Success));

        // Sleep to ensure deletion is processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Query for the deleted event by ID - should be empty after fix
        let after_by_id = db
            .query(Filter::new().id(event.id))
            .await
            .expect("Failed to query");
        assert_eq!(
            after_by_id.len(),
            0,
            "Deleted event should not be returned in ID query"
        );

        // Query for the deleted event by author-kind - should be empty after fix
        let after_by_author = db
            .query(Filter::new().author(keys.public_key()).kind(Kind::TextNote))
            .await
            .expect("Failed to query");
        assert_eq!(
            after_by_author.len(),
            0,
            "Deleted event should not be returned in author-kind query"
        );

        // The deletion event itself should still be queryable
        let deletion_events = db
            .query(Filter::new().kind(Kind::EventDeletion))
            .await
            .expect("Failed to query");
        assert_eq!(
            deletion_events.len(),
            1,
            "Deletion event should remain queryable"
        );
    }
}
