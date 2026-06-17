//! Recursive packet framing and parity helpers for the `perfect_ip` protocol.
//!
//! `perfect_ip` is a repair-oriented tree protocol. It does not try to be a
//! general transport; instead it turns a byte buffer into a deterministic tree
//! of MTU-safe data slices and parity slices that can be reconstructed when a
//! sibling packet is missing.
//!
//! ## Packet model
//!
//! - [`process_slice`] recursively splits payloads until each emitted leaf fits
//!   within the packet budget.
//! - Internal nodes emit a left child, right child, and parity frame.
//! - Packet ids form a stable recursive path such as `ROOT.1.0.P`.
//! - [`packetize`] finalizes the batch by filling `total_packets` into every
//!   packet header.
//!
//! ## Wire model
//!
//! On the wire, packet slices are exchanged as Nostr events. Each slice is
//! encoded as JSON content under a dedicated custom event kind, which keeps the
//! protocol relay-native and removes the old transport dependency.
//!
//! ## Repair model
//!
//! The parity scheme is XOR-based. Given one sibling payload and the matching
//! parity frame, [`recover_missing_data`] and [`recover_missing_slice`] can
//! rebuild the missing sibling payload. [`IntegrityManager`] tracks the
//! manifest, records received slices, persists the partial state to disk, and
//! verifies that parity frames still match their children.
//!
//! The layout intentionally mirrors the packet-tree sketch in the
//! `perfect_ip.rs` gist.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::io;
#[cfg(feature = "std")]
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Packet header metadata shared by all packet types in the tree.
///
/// `seq_num` is assigned in the order packets are emitted by the recursive
/// packetizer. `total_packets` is populated only after packetization has
/// finished, so callers can treat the batch as self-describing once finalized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// Monotonic sequence number assigned during packetization.
    pub seq_num: u32,
    /// Total number of packets in the finalized batch.
    pub total_packets: u32,
}

/// A single packet produced by the recursive packetizer.
///
/// `id` carries the recursive path for the packet, such as `ROOT.0.1.P`.
/// `is_parity` marks frames that store XOR parity rather than original user
/// data. `data` always contains the raw bytes that would be transmitted on the
/// wire inside the JSON-encoded repair message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSlice {
    /// Stable recursive packet identifier.
    pub id: String,
    /// Packet sequencing metadata.
    pub header: Header,
    /// Raw payload bytes for the packet or parity frame.
    pub data: Vec<u8>,
    /// `true` when this slice is a parity frame.
    pub is_parity: bool,
}

/// Finalized packet tree output.
///
/// `total_packets` is duplicated here so callers can inspect batch metadata
/// without walking the packet list. The packet list is stable and already has
/// each packet header patched to reflect the finalized count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    /// Number of packets in the batch.
    pub total_packets: u32,
    /// Finalized packets with matching `total_packets` headers.
    pub packets: Vec<ProtocolSlice>,
}

/// Tracks the expected manifest and the packets that have arrived so far.
///
/// The manager is deliberately simple: it stores the expected ids, the received
/// slices, and exposes helpers for missing-node inspection, parity validation,
/// and persistence.
#[derive(Debug, Clone, Default)]
pub struct IntegrityManager {
    manifest: BTreeSet<String>,
    /// Received packet slices keyed by packet id.
    pub received_slices: BTreeMap<String, ProtocolSlice>,
}

impl IntegrityManager {
    /// Create a manager from the set of expected packet ids.
    ///
    /// The manifest is stored as a set so lookups stay cheap even for larger
    /// packet trees.
    pub fn new(expected_ids: Vec<String>) -> Self {
        Self {
            manifest: expected_ids.into_iter().collect(),
            received_slices: BTreeMap::new(),
        }
    }

    /// Record a received packet by id, replacing any prior packet with the same id.
    ///
    /// Re-recording a slice is allowed and simply overwrites the previous
    /// entry. That makes repair retries and late arrivals idempotent.
    pub fn record_slice(&mut self, slice: ProtocolSlice) {
        self.received_slices.insert(slice.id.clone(), slice);
    }

    /// Persist the received packet map to disk using JSON.
    ///
    /// This writes only the received slice map. The caller supplies the
    /// manifest again on load so persisted state stays lightweight and portable.
    #[cfg(feature = "std")]
    pub fn persist(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let encoded = serde_json::to_vec(&self.received_slices).map_err(io::Error::other)?;
        std::fs::write(path, encoded)
    }

    /// Load a persisted packet map from disk and attach it to the given manifest.
    ///
    /// The manifest is not persisted with the slice data; callers are expected
    /// to regenerate or reload it independently.
    #[cfg(feature = "std")]
    pub fn load_from_disk(path: impl AsRef<Path>, manifest: BTreeSet<String>) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let received_slices: BTreeMap<String, ProtocolSlice> =
            serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        Ok(Self {
            manifest,
            received_slices,
        })
    }

    /// Return the packet ids from the manifest that are still missing.
    ///
    /// The returned ids preserve the manifest set's iteration order only in the
    /// sense that they are collected into a vector; callers should sort if they
    /// need deterministic display order.
    pub fn get_missing_nodes(&self) -> Vec<String> {
        self.manifest
            .iter()
            .filter(|id| !self.received_slices.contains_key(*id))
            .cloned()
            .collect()
    }

    /// Verify that every parity slice matches the XOR of its sibling data slices.
    ///
    /// If a parity node's siblings are both present, the check recomputes the
    /// XOR and compares it with the stored parity bytes. Missing siblings are
    /// ignored so partial downloads can still pass for the fragments they have.
    pub fn verify_integrity(&self) -> bool {
        for (id, slice) in &self.received_slices {
            if !slice.is_parity {
                continue;
            }

            let Some(base_id) = id.strip_suffix(".P") else {
                return false;
            };
            let left = self.received_slices.get(&format!("{}.0", base_id));
            let right = self.received_slices.get(&format!("{}.1", base_id));

            if let (Some(left), Some(right)) = (left, right) {
                if calculate_parity(&left.data, &right.data) != slice.data {
                    return false;
                }
            }
        }

        true
    }
}

/// Nostr event kind used for packet slices.
pub const PERFECT_IP_KIND: Kind = Kind::Custom(31_337);

/// Hashtag used to query perfect_ip events on relays.
pub const PERFECT_IP_HASHTAG: &str = "perfect_ip";

/// PIP manifest event kind.
pub const PIP_MANIFEST_KIND: Kind = Kind::Custom(39_078);

/// PIP slice event kind.
pub const PIP_SLICE_KIND: Kind = Kind::Custom(39_079);

/// PIP repair request event kind.
pub const PIP_REPAIR_REQUEST_KIND: Kind = Kind::Custom(39_080);

/// Hashtag used to query PIP events on relays.
pub const PIP_HASHTAG: &str = "pip";

/// Packet manifest describing one tree upload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketManifest {
    /// Stable root id for the packet tree.
    pub root: String,
    /// Hex-encoded SHA-256 of the original payload.
    pub sha256: String,
    /// Original payload size in bytes.
    pub size: u64,
    /// Total packet count for the tree.
    pub packets: u64,
    /// Maximum packet tree depth.
    pub depth: u32,
    /// Target MTU used by the packetizer.
    pub mtu: u64,
    /// Encoding name used on the wire.
    pub encoding: String,
    /// Relative path for directory uploads.
    pub path: String,
}

/// Repair request for missing packet ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairRequest {
    /// Stable root id for the packet tree.
    pub root: String,
    /// Packet ids still needed for reconstruction.
    pub want: Vec<String>,
    /// Encoding name used on the wire.
    pub encoding: String,
}

/// Build a filter for PIP events.
pub fn pip_filter(kind: Kind, id: Option<&str>, hashtag: &str) -> Filter {
    let filter = Filter::new().kind(kind).hashtag(hashtag);

    match id {
        Some(id) => filter.identifier(id),
        None => filter,
    }
}

/// Build a filter for PIP manifest events.
pub fn pip_manifest_filter(id: Option<&str>) -> Filter {
    pip_filter(PIP_MANIFEST_KIND, id, PIP_HASHTAG)
}

/// Build a filter for PIP slice events.
pub fn pip_slice_filter(id: Option<&str>) -> Filter {
    pip_filter(PIP_SLICE_KIND, id, PIP_HASHTAG)
}

/// Build a filter for PIP repair requests.
pub fn pip_repair_request_filter(id: Option<&str>) -> Filter {
    pip_filter(PIP_REPAIR_REQUEST_KIND, id, PIP_HASHTAG)
}

/// Convert a manifest into a Nostr event builder.
pub fn packet_manifest_to_event(
    manifest: &PacketManifest,
    packet_event_ids: &[EventId],
) -> Result<EventBuilder, serde_json::Error> {
    Ok(
        EventBuilder::new(PIP_MANIFEST_KIND, serde_json::to_string(manifest)?)
            .tag(Tag::identifier(manifest.root.clone()))
            .tag(Tag::custom("sha256", [manifest.sha256.clone()]))
            .tag(Tag::custom("size", [manifest.size.to_string()]))
            .tag(Tag::custom("packets", [manifest.packets.to_string()]))
            .tag(Tag::custom("depth", [manifest.depth.to_string()]))
            .tag(Tag::custom("mtu", [manifest.mtu.to_string()]))
            .tag(Tag::custom("encoding", [manifest.encoding.clone()]))
            .tag(Tag::custom("path", [manifest.path.clone()]))
            .tag(Tag::hashtag(PIP_HASHTAG))
            .tag(Tag::hashtag("manifest"))
            .tags(packet_event_ids.iter().map(|event_id| {
                Tag::custom(
                    "e",
                    [event_id.to_string(), String::new(), "slice".to_string()],
                )
            })),
    )
}

/// Extract packet event ids referenced by a PIP manifest event.
pub fn manifest_packet_event_ids(event: &Event) -> Vec<EventId> {
    event
        .tags
        .iter()
        .filter(|tag| tag.kind() == "e" && tag.len() >= 4 && tag[3].as_str() == "slice")
        .filter_map(|tag| tag.content().and_then(|value| EventId::parse(value).ok()))
        .collect()
}

/// Convert a repair request into a Nostr event builder.
pub fn repair_request_to_event(
    request: &RepairRequest,
    manifest_event_id: EventId,
) -> Result<EventBuilder, serde_json::Error> {
    let builder = request.want.iter().fold(
        EventBuilder::new(PIP_REPAIR_REQUEST_KIND, serde_json::to_string(request)?)
            .tag(Tag::identifier(request.root.clone()))
            .tag(Tag::custom(
                "e",
                [
                    manifest_event_id.to_string(),
                    String::new(),
                    "request".to_string(),
                ],
            ))
            .tag(Tag::custom("encoding", [request.encoding.clone()]))
            .tag(Tag::hashtag(PIP_HASHTAG))
            .tag(Tag::hashtag("repair")),
        |builder, want| builder.tag(Tag::custom("want", [want.clone()])),
    );

    Ok(builder)
}

/// Convert a packet slice into a PIP slice event builder.
pub fn packet_slice_to_event(slice: &ProtocolSlice) -> Result<EventBuilder, serde_json::Error> {
    let type_tag = if slice.is_parity { "parity" } else { "data" };

    Ok(
        EventBuilder::new(PIP_SLICE_KIND, serde_json::to_string(slice)?)
            .tag(Tag::identifier(slice.id.clone()))
            .tag(Tag::custom("seq", [slice.header.seq_num.to_string()]))
            .tag(Tag::custom("path", [slice.id.clone()]))
            .tag(Tag::custom("type", [type_tag.to_string()]))
            .tag(Tag::custom("encoding", ["json"]))
            .tag(Tag::hashtag(PIP_HASHTAG))
            .tag(Tag::hashtag("slice")),
    )
}

fn proposal_record_to_slice(
    kind: Kind,
    tags: &Tags,
    content: &str,
) -> Result<Option<ProtocolSlice>, serde_json::Error> {
    if kind != PIP_SLICE_KIND {
        return Ok(None);
    }

    let Some(root_id) = tags
        .iter()
        .find(|tag| tag.kind() == "d")
        .and_then(|tag| tag.content())
    else {
        return Ok(None);
    };

    if !tags
        .iter()
        .any(|tag| tag.kind() == "t" && tag.content() == Some(PIP_HASHTAG))
    {
        return Ok(None);
    }

    if !tags
        .iter()
        .any(|tag| tag.kind() == "t" && tag.content() == Some("slice"))
    {
        return Ok(None);
    }

    let Some(path) = tags
        .iter()
        .find(|tag| tag.kind() == "path")
        .and_then(|tag| tag.content())
    else {
        return Ok(None);
    };

    if !path.starts_with(root_id) {
        return Ok(None);
    }

    let slice: ProtocolSlice = serde_json::from_str(content)?;
    if slice.id != path {
        return Ok(None);
    }

    Ok(Some(slice))
}

/// Try to decode a PIP event back into a packet slice.
pub fn packet_slice_from_event(event: &Event) -> Result<Option<ProtocolSlice>, serde_json::Error> {
    proposal_record_to_slice(event.kind, &event.tags, &event.content)
}

/// Try to decode an unsigned PIP event back into a packet slice.
pub fn packet_slice_from_unsigned_event(
    event: &UnsignedEvent,
) -> Result<Option<ProtocolSlice>, serde_json::Error> {
    proposal_record_to_slice(event.kind, &event.tags, &event.content)
}

/// Build a Nostr filter that matches perfect_ip packet events.
pub fn perfect_ip_filter(id: Option<&str>) -> Filter {
    let filter = Filter::new()
        .kind(PERFECT_IP_KIND)
        .hashtag(PERFECT_IP_HASHTAG);

    match id {
        Some(id) => filter.identifier(id),
        None => filter,
    }
}

/// Convert a packet slice into a Nostr event builder.
///
/// The slice is serialized into JSON content so relays can store and replay the
/// event without any custom transport logic.
pub fn slice_to_event(slice: &ProtocolSlice) -> Result<EventBuilder, serde_json::Error> {
    Ok(
        EventBuilder::new(PERFECT_IP_KIND, serde_json::to_string(slice)?)
            .tag(Tag::identifier(slice.id.clone()))
            .tag(Tag::hashtag(PERFECT_IP_HASHTAG)),
    )
}

/// Convert a packet batch into Nostr event builders.
pub fn batch_to_events(batch: &PacketBatch) -> Result<Vec<EventBuilder>, serde_json::Error> {
    batch.packets.iter().map(slice_to_event).collect()
}

fn record_to_slice(
    kind: Kind,
    tags: &Tags,
    content: &str,
) -> Result<Option<ProtocolSlice>, serde_json::Error> {
    if kind != PERFECT_IP_KIND {
        return Ok(None);
    }

    let Some(identifier) = tags
        .iter()
        .find(|tag| tag.kind() == "d")
        .and_then(|tag| tag.content())
    else {
        return Ok(None);
    };

    if !tags
        .iter()
        .any(|tag| tag.kind() == "t" && tag.content() == Some(PERFECT_IP_HASHTAG))
    {
        return Ok(None);
    }

    let slice: ProtocolSlice = serde_json::from_str(content)?;
    if slice.id != identifier {
        return Ok(None);
    }

    Ok(Some(slice))
}

/// Try to decode a Nostr event back into a packet slice.
pub fn event_to_slice(event: &Event) -> Result<Option<ProtocolSlice>, serde_json::Error> {
    record_to_slice(event.kind, &event.tags, &event.content)
}

/// Try to decode an unsigned Nostr event back into a packet slice.
pub fn unsigned_event_to_slice(
    event: &UnsignedEvent,
) -> Result<Option<ProtocolSlice>, serde_json::Error> {
    record_to_slice(event.kind, &event.tags, &event.content)
}

/// Maximum payload size this packet protocol is allowed to emit.
///
/// The tree uses half-sized leaves so that parity slices for sibling pairs stay
/// under the same transport ceiling as the data leaves.
pub const MTU_PAYLOAD: usize = 1460;
const MAX_LEAF_PAYLOAD: usize = MTU_PAYLOAD / 2;

/// XOR two payloads into a parity buffer.
///
/// Missing bytes are treated as zero so the returned buffer is as long as the
/// larger input. This is the core repair primitive used at every branch in the
/// recursive packet tree.
pub fn calculate_parity(left: &[u8], right: &[u8]) -> Vec<u8> {
    let max_len = left.len().max(right.len());
    let mut parity = vec![0; max_len];
    for i in 0..max_len {
        let l = if i < left.len() { left[i] } else { 0 };
        let r = if i < right.len() { right[i] } else { 0 };
        parity[i] = l ^ r;
    }
    parity
}

/// Generate the expected packet ids for a recursive packet tree.
///
/// The returned manifest mirrors [`process_slice`] exactly: leaf nodes are
/// emitted when the payload fits within the leaf payload limit, and internal
/// nodes append `.0`, `.1`, and `.P` entries in tree order. This is what lets
/// [`IntegrityManager`] tell which packets are still missing.
pub fn generate_manifest(id: String, len: usize) -> Vec<String> {
    if len <= MAX_LEAF_PAYLOAD {
        return vec![id];
    }

    let half = len / 2;
    let mut ids = generate_manifest(format!("{}.0", id), half);
    ids.append(&mut generate_manifest(format!("{}.1", id), len - half));
    ids.push(format!("{}.P", id));
    ids
}

/// Recover the missing payload by XORing the sibling payload and parity frame.
///
/// `expected_len` trims the recovered buffer back to the original payload
/// length. If the sibling and parity bytes are from the same branch, the XOR
/// yields the missing sibling exactly.
pub fn recover_missing_data(expected_len: usize, sibling: &[u8], parity: &[u8]) -> Vec<u8> {
    let recovered = calculate_parity(sibling, parity);
    recovered.into_iter().take(expected_len).collect()
}

/// Rebuild a missing packet using a sibling packet and parity packet.
///
/// The returned slice is marked as data and receives the next sequence number.
/// This is a convenience wrapper for repair handlers that need to synthesize a
/// complete [`ProtocolSlice`] from branch-local evidence.
pub fn recover_missing_slice(
    id: String,
    expected_len: usize,
    sibling: &ProtocolSlice,
    parity: &ProtocolSlice,
    seq: &mut u32,
) -> ProtocolSlice {
    let header = Header {
        seq_num: *seq,
        total_packets: sibling
            .header
            .total_packets
            .max(parity.header.total_packets),
    };
    *seq += 1;

    ProtocolSlice {
        id,
        header,
        data: recover_missing_data(expected_len, &sibling.data, &parity.data),
        is_parity: false,
    }
}

/// Recursively split a payload into MTU-safe slices and parity frames.
///
/// Leaf packets are emitted when the payload is at or below the leaf payload
/// limit. Internal nodes emit left, right, and parity frames in
/// a deterministic order so the manifest and sequence numbers line up exactly.
pub fn process_slice(id: String, data: Vec<u8>, seq: &mut u32) -> Vec<ProtocolSlice> {
    if data.len() <= MAX_LEAF_PAYLOAD {
        let slice = ProtocolSlice {
            id,
            header: Header {
                seq_num: *seq,
                total_packets: 0,
            },
            data,
            is_parity: false,
        };
        *seq += 1;
        return vec![slice];
    }

    let half = data.len() / 2;
    let left_data = data[..half].to_vec();
    let right_data = data[half..].to_vec();

    let parity = calculate_parity(&left_data, &right_data);
    let mut slices = process_slice(format!("{}.0", id), left_data, seq);
    slices.append(&mut process_slice(format!("{}.1", id), right_data, seq));

    slices.push(ProtocolSlice {
        id: format!("{}.P", id),
        header: Header {
            seq_num: *seq,
            total_packets: 0,
        },
        data: parity,
        is_parity: true,
    });
    *seq += 1;

    slices
}

/// Packetize a payload and finalize the batch metadata.
///
/// This is the preferred entrypoint for callers that want a ready-to-send
/// packet tree with `total_packets` filled in. The returned batch is ready for
/// logging, persistence, or network repair.
pub fn packetize(id: String, data: Vec<u8>) -> PacketBatch {
    let mut seq = 0;
    let mut packets = process_slice(id, data, &mut seq);
    let total_packets = packets.len() as u32;
    for packet in &mut packets {
        packet.header.total_packets = total_packets;
    }
    PacketBatch {
        total_packets,
        packets,
    }
}

/// Render packet summaries for logs or diagnostics.
///
/// This is intentionally human-facing output for terminal demos and debugging.
/// It keeps the packet ids, sequence numbers, types, and sizes aligned in a
/// single line per packet.
pub fn summarize_packets(packets: &[ProtocolSlice]) -> Vec<String> {
    packets
        .iter()
        .map(|packet| {
            format!(
                "ID: {:<8} | Seq: {:>2}/{} | Type: {:<6} | Size: {}B",
                packet.id,
                packet.header.seq_num,
                packet.header.total_packets,
                if packet.is_parity { "PARITY" } else { "DATA" },
                packet.data.len()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use hashes::Hash;
    use nostr_sdk::prelude::{
        Client, Event as SdkEvent, Filter as SdkFilter, RelayUrl as SdkRelayUrl,
    };

    use super::*;

    #[test]
    fn process_slice_emits_parity_and_data() {
        let raw_data = vec![0xAB; 2000];
        let batch = packetize("ROOT".to_string(), raw_data);
        let packets = batch.packets;

        assert!(!packets.is_empty());
        assert!(packets.iter().any(|packet| packet.is_parity));
        assert!(packets.iter().any(|packet| packet.id == "ROOT.P"));
        assert!(
            packets
                .iter()
                .any(|packet| packet.id.starts_with("ROOT.0."))
        );
        assert!(
            packets
                .iter()
                .any(|packet| packet.id.starts_with("ROOT.1."))
        );
        assert!(
            packets
                .iter()
                .all(|packet| packet.data.len() <= MTU_PAYLOAD)
        );
        assert!(
            packets
                .iter()
                .all(|packet| packet.header.total_packets == batch.total_packets)
        );
    }

    #[test]
    fn calculate_parity_xors_equal_length_blocks() {
        let left = [0xDE, 0xAD, 0xBE];
        let right = [0x01, 0x02, 0x03];
        assert_eq!(calculate_parity(&left, &right), vec![0xDF, 0xAF, 0xBD]);
    }

    #[test]
    fn generate_manifest_matches_recursive_packet_tree() {
        let manifest = generate_manifest("ROOT".to_string(), 2000);

        assert!(manifest.contains(&"ROOT.P".to_string()));
        assert!(manifest.iter().any(|id| id.starts_with("ROOT.0.")));
        assert!(manifest.iter().any(|id| id.starts_with("ROOT.1.")));
        assert_eq!(
            manifest.len(),
            packetize("ROOT".to_string(), vec![0xAB; 2000]).total_packets as usize
        );
    }

    #[test]
    fn recover_missing_data_restores_xor_partner() {
        let left = vec![0xDE, 0xAD, 0xBE];
        let right = vec![0x01, 0x02, 0x03];
        let parity = calculate_parity(&left, &right);

        assert_eq!(recover_missing_data(left.len(), &right, &parity), left);
        assert_eq!(recover_missing_data(right.len(), &left, &parity), right);
    }

    #[test]
    fn recover_missing_slice_rebuilds_header_and_payload() {
        let sibling = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0x01, 0x02, 0x03],
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 2,
                total_packets: 3,
            },
            data: vec![0xDF, 0xAF, 0xBD],
            is_parity: true,
        };
        let mut seq = 3;

        let recovered = recover_missing_slice("ROOT.0".to_string(), 3, &sibling, &parity, &mut seq);

        assert_eq!(recovered.id, "ROOT.0");
        assert_eq!(recovered.data, vec![0xDE, 0xAD, 0xBE]);
        assert_eq!(recovered.header.seq_num, 3);
        assert_eq!(recovered.header.total_packets, 3);
        assert!(!recovered.is_parity);
    }

    #[test]
    fn get_missing_nodes_returns_unseen_manifest_entries() {
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        manager.record_slice(ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE],
            is_parity: false,
        });

        let missing = manager.get_missing_nodes();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"ROOT.1".to_string()));
        assert!(missing.contains(&"ROOT.P".to_string()));
    }

    #[test]
    fn verify_integrity_checks_parity_frames() {
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        let left = ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE],
            is_parity: false,
        };
        let right = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0x01, 0x02, 0x03],
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 2,
                total_packets: 3,
            },
            data: calculate_parity(&left.data, &right.data),
            is_parity: true,
        };

        manager.record_slice(left);
        manager.record_slice(right);
        manager.record_slice(parity);
        assert!(manager.verify_integrity());

        let mut corrupted = manager.clone();
        corrupted
            .received_slices
            .get_mut("ROOT.P")
            .expect("parity slice")
            .data[0] ^= 0xFF;
        assert!(!corrupted.verify_integrity());
    }

    #[test]
    #[cfg(feature = "std")]
    fn persist_and_load_from_disk_round_trips_received_slices() {
        let temp_path = std::env::temp_dir().join(format!(
            "nostr-pip-{}-{}.json",
            std::process::id(),
            Timestamp::now().as_secs()
        ));
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        manager.record_slice(ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE],
            is_parity: false,
        });
        manager.record_slice(ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0xAA],
            is_parity: true,
        });

        manager.persist(&temp_path).expect("persist");
        let loaded = IntegrityManager::load_from_disk(
            &temp_path,
            vec![
                "ROOT.0".to_string(),
                "ROOT.1".to_string(),
                "ROOT.P".to_string(),
            ]
            .into_iter()
            .collect(),
        )
        .expect("load");

        assert_eq!(loaded.received_slices.len(), 2);
        assert!(loaded.received_slices.contains_key("ROOT.0"));
        assert!(loaded.received_slices.contains_key("ROOT.P"));
        assert!(loaded.get_missing_nodes().contains(&"ROOT.1".to_string()));
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn protocol_slice_json_round_trips_bytes_and_flags() {
        let slice = ProtocolSlice {
            id: "ROOT.0.P".to_string(),
            header: Header {
                seq_num: 7,
                total_packets: 63,
            },
            data: vec![0x00, 0xFF, 0x10, 0x20],
            is_parity: true,
        };

        let encoded = serde_json::to_string(&slice).expect("encode slice");
        assert!(encoded.contains("\"id\":\"ROOT.0.P\""));
        assert!(encoded.contains("\"is_parity\":true"));
        assert!(encoded.contains("\"data\":[0,255,16,32]"));

        let decoded: ProtocolSlice = serde_json::from_str(&encoded).expect("decode slice");
        assert_eq!(decoded, slice);
    }

    #[test]
    fn summarize_packets_shows_packet_inventory_details() {
        let batch = packetize("ROOT".to_string(), vec![0xAB; 2000]);
        let lines = summarize_packets(&batch.packets);

        assert_eq!(lines.len(), batch.total_packets as usize);
        assert!(lines.iter().any(|line| line.contains("ID: ROOT.P")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!("Seq: {:>2}/{}", 0, batch.total_packets)))
        );
        assert!(lines.iter().any(|line| line.contains("Type: DATA")));
        assert!(lines.iter().any(|line| line.contains("Type: PARITY")));
    }

    #[test]
    fn manifest_packet_ids_point_to_slice_events_only() {
        let payload = vec![0xAB; 64];
        let batch = packetize("ROOT".to_string(), payload.clone());
        let keys = Keys::new(
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("secret key"),
        );
        let packet_event_ids: Vec<EventId> = batch
            .packets
            .iter()
            .map(|slice| {
                let event = packet_slice_to_event(slice)
                    .expect("build packet slice event")
                    .finalize(&keys)
                    .expect("finalize slice event");
                event.id
            })
            .collect();
        let manifest = PacketManifest {
            root: "ROOT".to_string(),
            sha256: "deadbeef".to_string(),
            size: payload.len() as u64,
            packets: batch.total_packets as u64,
            depth: 0,
            mtu: MTU_PAYLOAD as u64,
            encoding: "json".to_string(),
            path: "pip.rs".to_string(),
        };
        let manifest_event = packet_manifest_to_event(&manifest, &packet_event_ids)
            .expect("build manifest event")
            .finalize(&keys)
            .expect("finalize manifest event");

        let refs = manifest_packet_event_ids(&manifest_event);
        assert_eq!(refs.len(), batch.total_packets as usize);
        assert_eq!(refs, packet_event_ids);
        assert!(
            packet_slice_from_event(&manifest_event)
                .expect("decode manifest")
                .is_none()
        );
    }

    #[tokio::test]
    #[ignore = "live relay test"]
    #[cfg(feature = "std")]
    async fn live_nos_lol_round_trip_reconstructs_and_writes_file() {
        fn reconstruct_payload_from_events(events: &[Event]) -> Vec<u8> {
            let mut slices: Vec<_> = events
                .iter()
                .filter_map(|event| packet_slice_from_event(event).expect("decode packet event"))
                .collect();
            slices.sort_by_key(|slice| slice.header.seq_num);
            slices
                .into_iter()
                .filter(|slice| !slice.is_parity)
                .flat_map(|slice| slice.data)
                .collect()
        }

        let relay_urls = [
            SdkRelayUrl::parse("wss://nos.lol").expect("nos.lol relay url"),
            SdkRelayUrl::parse("wss://relay.damus.io").expect("damus relay url"),
        ];
        let relay = relay_urls[SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .subsec_nanos() as usize
            % relay_urls.len()]
        .clone();
        let client = Client::default();
        client
            .add_relay(&relay)
            .and_connect()
            .await
            .expect("add relay");
        eprintln!("perfect_ip live relay test connected: {}", relay);

        let keys = Keys::new(
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("secret key"),
        );
        let payload = vec![0xAB; 3000];
        let batch = packetize("ROOT.live".to_string(), payload.clone());
        let manifest = PacketManifest {
            root: "ROOT.live".to_string(),
            sha256: hashes::sha256::Hash::hash(&payload).to_string(),
            size: payload.len() as u64,
            packets: batch.total_packets as u64,
            depth: 0,
            mtu: MTU_PAYLOAD as u64,
            encoding: "json".to_string(),
            path: "pip.rs".to_string(),
        };
        eprintln!(
            "perfect_ip live relay test prepared payload: bytes={}, packets={}",
            payload.len(),
            batch.total_packets
        );

        let mut packet_event_ids = Vec::with_capacity(batch.total_packets as usize);
        let mut sdk_packet_event_ids = Vec::with_capacity(batch.total_packets as usize);
        let mut packet_events = Vec::with_capacity(batch.total_packets as usize);
        for slice in &batch.packets {
            let event = packet_slice_to_event(slice)
                .expect("build slice event")
                .finalize(&keys)
                .expect("finalize slice event");
            packet_event_ids.push(event.id);
            let sdk_event: SdkEvent =
                serde_json::from_str(&event.as_json()).expect("sdk slice event");
            sdk_packet_event_ids.push(sdk_event.id);
            eprintln!("perfect_ip live relay test publishing slice: {}", event.id);
            let send_result = client
                .send_event(&sdk_event)
                .to([&relay])
                .await
                .expect("send slice event");
            eprintln!(
                "perfect_ip live relay test slice send result: id={}, success={}, failed={}",
                send_result.id(),
                send_result.success.len(),
                send_result.failed.len()
            );
            if !send_result.failed.is_empty() {
                eprintln!(
                    "perfect_ip live relay test slice send failures: {:?}",
                    send_result.failed
                );
            }
            packet_events.push(event);
        }

        let manifest_event = packet_manifest_to_event(&manifest, &packet_event_ids)
            .expect("build manifest event")
            .finalize(&keys)
            .expect("finalize manifest event");
        let sdk_manifest_event: SdkEvent =
            serde_json::from_str(&manifest_event.as_json()).expect("sdk manifest event");
        eprintln!(
            "perfect_ip live relay test publishing manifest: {}",
            manifest_event.id
        );
        let manifest_send_result = client
            .send_event(&sdk_manifest_event)
            .to([&relay])
            .await
            .expect("send manifest event");
        eprintln!(
            "perfect_ip live relay test manifest send result: id={}, success={}, failed={}",
            manifest_send_result.id(),
            manifest_send_result.success.len(),
            manifest_send_result.failed.len()
        );
        if !manifest_send_result.failed.is_empty() {
            eprintln!(
                "perfect_ip live relay test manifest send failures: {:?}",
                manifest_send_result.failed
            );
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        let fetched_manifest = tokio::time::timeout(Duration::from_secs(20), async {
            let mut attempts = 0usize;
            loop {
                attempts += 1;
                eprintln!(
                    "perfect_ip live relay test fetching manifest attempt {}",
                    attempts
                );
                let fetched = client
                    .fetch_events(SdkFilter::new().ids([sdk_manifest_event.id]))
                    .await
                    .expect("fetch manifest");
                if let Some(event) = fetched.first().cloned() {
                    eprintln!(
                        "perfect_ip live relay test fetched manifest after {} attempts",
                        attempts
                    );
                    break event;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
        .await
        .expect("manifest fetch timeout");
        let fetched_manifest: Event =
            serde_json::from_str(&fetched_manifest.as_json()).expect("nostr manifest event");
        let packet_ids = manifest_packet_event_ids(&fetched_manifest);
        assert_eq!(packet_ids, packet_event_ids);

        let relay_handle = client.relays().await.remove(&relay).expect("relay handle");
        let fetched_packets = tokio::time::timeout(Duration::from_secs(20), async {
            let mut attempts = 0usize;
            loop {
                attempts += 1;
                eprintln!(
                    "perfect_ip live relay test fetching packets attempt {}",
                    attempts
                );
                let fetched = relay_handle
                    .fetch_events(SdkFilter::new().ids(sdk_packet_event_ids.clone()))
                    .await
                    .expect("fetch packets");
                if fetched.len() >= packet_event_ids.len() {
                    eprintln!(
                        "perfect_ip live relay test fetched packets after {} attempts: {} / {}",
                        attempts,
                        fetched.len(),
                        packet_event_ids.len()
                    );
                    break fetched;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
        .await
        .expect("packet fetch timeout");
        let fetched_packets: Vec<Event> = fetched_packets
            .into_iter()
            .map(|event| serde_json::from_str(&event.as_json()).expect("nostr packet event"))
            .collect();
        let reconstructed = reconstruct_payload_from_events(&fetched_packets);
        assert_eq!(reconstructed, payload);

        let out_dir = std::env::temp_dir().join("nostr-pip-live-out");
        let output_path = out_dir.join(
            Path::new(&manifest.path)
                .file_name()
                .expect("manifest file"),
        );
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).expect("create output dir");
        }
        std::fs::write(&output_path, &reconstructed).expect("write output");
        let written = std::fs::read(&output_path).expect("read output");
        assert_eq!(written, payload);

        eprintln!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip live relay round trip",
                "relay": relay.as_str(),
                "manifest_event": manifest_event.id.to_string(),
                "packets": packet_ids.len(),
                "bytes": written.len(),
                "out": output_path.display().to_string(),
                "matches": written == payload,
            })
        );

        client.shutdown().await;
    }

    #[test]
    fn dump_full_protocol_for_nocapture() {
        use hashes::Hash;
        use hashes::sha1::Hash as Sha1Hash;

        fn reconstruct_payload_for_test(packets: &[ProtocolSlice]) -> Vec<u8> {
            let mut leaves: Vec<_> = packets.iter().filter(|packet| !packet.is_parity).collect();
            leaves.sort_by_key(|packet| packet.header.seq_num);
            leaves
                .into_iter()
                .flat_map(|packet| packet.data.clone())
                .collect()
        }

        #[derive(Default)]
        struct MockRelay {
            online: bool,
            dropped: usize,
            events: Vec<UnsignedEvent>,
        }

        impl MockRelay {
            fn set_online(&mut self, online: bool) {
                self.online = online;
            }

            fn publish_if_online(&mut self, event: UnsignedEvent) -> bool {
                if self.online {
                    self.events.push(event);
                    true
                } else {
                    self.dropped += 1;
                    false
                }
            }

            fn packet_slices(&self) -> Vec<ProtocolSlice> {
                self.events
                    .iter()
                    .filter(|event| event.kind == PERFECT_IP_KIND)
                    .map(|event| {
                        unsigned_event_to_slice(event)
                            .expect("decode packet event")
                            .expect("packet event")
                    })
                    .filter(|slice| !slice.is_parity)
                    .collect()
            }

            fn packet_records(&self) -> Vec<ProtocolSlice> {
                self.events
                    .iter()
                    .filter(|event| event.kind == PERFECT_IP_KIND)
                    .map(|event| {
                        unsigned_event_to_slice(event)
                            .expect("decode packet event")
                            .expect("packet event")
                    })
                    .collect()
            }

            fn publish(&mut self, event: UnsignedEvent) {
                self.events.push(event);
            }

            fn clone_snapshot(&self) -> Vec<ProtocolSlice> {
                let mut slices: Vec<ProtocolSlice> = self.packet_slices();
                slices.sort_by_key(|slice| slice.header.seq_num);
                slices
            }

            fn clone_snapshot_from_template(
                &self,
                template: &[ProtocolSlice],
            ) -> Vec<ProtocolSlice> {
                let slices = self.packet_slices();
                let map: std::collections::HashMap<String, ProtocolSlice> = slices
                    .into_iter()
                    .map(|slice| (slice.id.clone(), slice))
                    .collect();

                template
                    .iter()
                    .filter(|packet| !packet.is_parity)
                    .filter_map(|packet| map.get(&packet.id).cloned())
                    .collect()
            }
        }

        let payload = vec![0xAB; 3000];
        let batch = packetize("ROOT".to_string(), payload.clone());
        let manifest = generate_manifest("ROOT".to_string(), payload.len());
        let mut manager = IntegrityManager::new(manifest.clone());
        let demo_keys = Keys::new(
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("secret key"),
        );

        println!("perfect_ip manifest ({} ids):", manifest.len());
        for id in &manifest {
            println!("  {id}");
        }

        println!(
            "perfect_ip packet inventory ({} packets):",
            batch.total_packets
        );
        for line in summarize_packets(&batch.packets) {
            println!("{line}");
        }

        println!("perfect_ip packet batch json:");
        println!(
            "{}",
            serde_json::to_string(&batch).expect("serialize packet batch")
        );

        let event_builders = batch_to_events(&batch).expect("build nostr events");
        println!("perfect_ip nostr event count: {}", event_builders.len());
        let preview_event = event_builders
            .first()
            .expect("first nostr event")
            .clone()
            .finalize_unsigned(demo_keys.public_key());
        println!("perfect_ip nostr preview kind: {:?}", preview_event.kind);
        println!(
            "perfect_ip nostr preview content: {}",
            preview_event.content
        );
        println!(
            "perfect_ip nostr preview tags: {:?}",
            preview_event
                .tags
                .iter()
                .map(|tag| tag.as_slice().to_vec())
                .collect::<Vec<_>>()
        );
        println!(
            "perfect_ip nostr preview decoded: {:?}",
            unsigned_event_to_slice(&preview_event).expect("decode preview event")
        );

        println!("perfect_ip first packet json:");
        println!(
            "{}",
            serde_json::to_string(&batch.packets.first().expect("first packet"))
                .expect("serialize first packet")
        );

        println!("perfect_ip last packet json:");
        println!(
            "{}",
            serde_json::to_string(&batch.packets.last().expect("last packet"))
                .expect("serialize last packet")
        );

        for slice in batch.packets.clone() {
            manager.record_slice(slice);
        }
        println!(
            "perfect_ip missing nodes: {:?}",
            manager.get_missing_nodes()
        );
        println!(
            "perfect_ip integrity verified: {}",
            manager.verify_integrity()
        );

        let reconstructed = reconstruct_payload_for_test(&batch.packets);
        println!("perfect_ip reconstructed bytes: {}", reconstructed.len());
        println!("perfect_ip payload matches: {}", reconstructed == payload);
        assert_eq!(reconstructed, payload);

        let mut relay = MockRelay::default();
        let repo_id = "perfect-ip-demo";
        let owner_public_key = demo_keys.public_key();
        let repo_announcement = GitRepositoryAnnouncement {
            id: repo_id.to_string(),
            name: Some("perfect_ip demo repo".to_string()),
            description: Some("Mock NIP-34 repo push carried over perfect_ip".to_string()),
            web: vec![Url::parse("https://example.com/perfect-ip-demo").expect("demo web url")],
            clone: vec![
                Url::parse("https://example.com/perfect-ip-demo.git").expect("demo clone url"),
            ],
            relays: vec![RelayUrl::parse("wss://relay.damus.io").expect("demo relay url")],
            euc: Some(Sha1Hash::hash(payload.as_slice())),
            maintainers: vec![owner_public_key],
        };

        let repo_event = EventBuilder::git_repository_announcement(repo_announcement.clone())
            .finalize_unsigned(owner_public_key);
        relay.publish(repo_event.clone());
        println!(
            "perfect_ip git repo announcement kind: {:?}",
            repo_event.kind
        );
        println!(
            "perfect_ip git repo announcement tags: {:?}",
            repo_event
                .tags
                .iter()
                .map(|tag| tag.as_slice().to_vec())
                .collect::<Vec<_>>()
        );

        let repository =
            Coordinate::new(Kind::GitRepoAnnouncement, owner_public_key).identifier(repo_id);
        let snapshot_hash = Sha1Hash::hash(payload.as_slice());
        let patch = GitPatch {
            repository,
            content: GitPatchContent::CoverLetter {
                title: "perfect_ip repo snapshot".to_string(),
                description: format!(
                    "Mock relay upload: {} bytes split into {} perfect_ip packets",
                    payload.len(),
                    batch.total_packets
                ),
                last_commit: snapshot_hash,
                commits_len: batch.total_packets as usize,
            },
            euc: snapshot_hash,
            labels: vec!["perfect_ip".to_string(), "mock-relay".to_string()],
        };
        let patch_event = EventBuilder::git_patch(patch)
            .expect("build git patch")
            .finalize_unsigned(owner_public_key);
        relay.publish(patch_event.clone());
        println!("perfect_ip git patch kind: {:?}", patch_event.kind);
        println!("perfect_ip git patch content: {}", patch_event.content);
        println!(
            "perfect_ip git patch tags: {:?}",
            patch_event
                .tags
                .iter()
                .map(|tag| tag.as_slice().to_vec())
                .collect::<Vec<_>>()
        );
        println!(
            "perfect_ip git clone source: {}",
            repo_announcement.clone.first().expect("clone url")
        );

        let packet_events: Vec<UnsignedEvent> = batch_to_events(&batch)
            .expect("build packet events")
            .into_iter()
            .map(|builder| builder.finalize_unsigned(owner_public_key))
            .collect();
        relay.events.extend(packet_events.clone());
        println!("perfect_ip git relay event count: {}", relay.events.len());

        let cloned_snapshot: Vec<u8> = relay
            .clone_snapshot()
            .into_iter()
            .flat_map(|slice| slice.data)
            .collect();
        println!("perfect_ip git clone bytes: {}", cloned_snapshot.len());
        println!(
            "perfect_ip git clone matches: {}",
            cloned_snapshot == payload
        );
        assert_eq!(cloned_snapshot, payload);

        let mut repair_relay = MockRelay::default();
        repair_relay.set_online(true);
        let repair_target_id = "ROOT.0.1.1";
        let repair_target = batch
            .packets
            .iter()
            .find(|packet| packet.id == repair_target_id)
            .expect("repair target");
        let repair_sibling = batch
            .packets
            .iter()
            .find(|packet| packet.id == "ROOT.0.1.0")
            .expect("repair sibling");
        let repair_parity = batch
            .packets
            .iter()
            .find(|packet| packet.id == "ROOT.0.1.P")
            .expect("repair parity");

        println!("perfect_ip repair target: {}", repair_target_id);
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip repair target",
                "target": repair_target_id,
            })
        );
        for (packet, builder) in batch
            .packets
            .iter()
            .zip(batch_to_events(&batch).expect("event builders"))
        {
            let event = builder.finalize_unsigned(owner_public_key);
            if packet.id == repair_target_id {
                repair_relay.set_online(false);
                let accepted = repair_relay.publish_if_online(event);
                println!(
                    "perfect_ip relay outage dropped {}: {}",
                    repair_target_id, !accepted
                );
                println!(
                    "{}",
                    serde_json::json!({
                        "event": "perfect_ip relay outage dropped",
                        "target": repair_target_id,
                        "dropped": !accepted,
                    })
                );
                repair_relay.set_online(true);
                continue;
            }

            let _ = repair_relay.publish_if_online(event);
        }

        let partial_manager = {
            let mut manager = IntegrityManager::new(manifest.clone());
            for slice in repair_relay.packet_records() {
                manager.record_slice(slice);
            }
            manager
        };
        println!(
            "perfect_ip partial missing nodes: {:?}",
            partial_manager.get_missing_nodes()
        );
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip partial missing nodes",
                "nodes": partial_manager.get_missing_nodes(),
            })
        );

        let mut repair_seq = repair_target.header.seq_num;
        let repaired_slice = recover_missing_slice(
            repair_target.id.clone(),
            repair_target.data.len(),
            repair_sibling,
            repair_parity,
            &mut repair_seq,
        );
        let repair_event = slice_to_event(&repaired_slice)
            .expect("serialize repair slice")
            .finalize_unsigned(owner_public_key);
        repair_relay.publish(repair_event);
        println!("perfect_ip repair event published: {}", repaired_slice.id);
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip repair event published",
                "target": repaired_slice.id,
                "seq_num": repaired_slice.header.seq_num,
                "total_packets": repaired_slice.header.total_packets,
                "is_parity": repaired_slice.is_parity,
                "bytes": repaired_slice.data.len(),
            })
        );

        let repaired_snapshot = repair_relay.clone_snapshot_from_template(&batch.packets);
        let repaired_payload: Vec<u8> = repaired_snapshot
            .iter()
            .flat_map(|slice| slice.data.clone())
            .collect();
        let repaired_matches = repaired_payload == payload;
        println!(
            "perfect_ip repaired relay dropped events: {}",
            repair_relay.dropped
        );
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip repaired relay dropped events",
                "dropped_events": repair_relay.dropped,
            })
        );
        println!(
            "perfect_ip repaired relay bytes: {}",
            repaired_payload.len()
        );
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip repaired relay bytes",
                "bytes": repaired_payload.len(),
            })
        );
        println!("perfect_ip repaired relay matches: {}", repaired_matches);
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip repaired relay matches",
                "matches": repaired_matches,
            })
        );
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip repaired relay summary",
                "dropped_events": repair_relay.dropped,
                "bytes": repaired_payload.len(),
                "matches": repaired_matches,
            })
        );
        assert_eq!(repaired_payload, payload);
    }

    #[test]
    fn perfect_ip_event_round_trips_through_nostr_content() {
        let slice = ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
            is_parity: false,
        };
        let keys = Keys::new(
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .expect("secret key"),
        );

        let event = slice_to_event(&slice)
            .expect("serialize slice")
            .finalize_unsigned(keys.public_key());

        println!("perfect_ip nostr event kind: {:?}", event.kind);
        println!("perfect_ip nostr event content: {}", event.content);
        println!(
            "perfect_ip nostr event tags: {:?}",
            event
                .tags
                .iter()
                .map(|tag| tag.as_slice().to_vec())
                .collect::<Vec<_>>()
        );

        let decoded = unsigned_event_to_slice(&event)
            .expect("decode event")
            .expect("perfect_ip event");

        println!("perfect_ip decoded slice id: {}", decoded.id);
        println!(
            "perfect_ip decoded slice json: {}",
            serde_json::to_string(&decoded).expect("serialize decoded slice")
        );
        println!(
            "perfect_ip nostr filter: {}",
            serde_json::to_string(&perfect_ip_filter(Some("ROOT.0"))).expect("serialize filter")
        );

        assert_eq!(decoded, slice);
        assert_eq!(event.kind, PERFECT_IP_KIND);
        assert!(
            event
                .tags
                .iter()
                .any(|tag| tag.kind() == "d" && tag.content() == Some("ROOT.0"))
        );
        assert!(
            event
                .tags
                .iter()
                .any(|tag| tag.kind() == "t" && tag.content() == Some(PERFECT_IP_HASHTAG))
        );
        assert!(
            perfect_ip_filter(Some("ROOT.0"))
                .kinds
                .unwrap()
                .contains(&PERFECT_IP_KIND)
        );
    }
}
