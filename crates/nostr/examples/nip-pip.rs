// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs};

use nostr::hashes::{Hash, sha256};
use nostr::nips::pip::{
    MTU_PAYLOAD, PIP_MANIFEST_KIND, PIP_SLICE_KIND, PacketManifest, ProtocolSlice,
    manifest_packet_event_ids, packet_manifest_to_event, packet_slice_from_event,
    packet_slice_to_event, packetize,
};
use nostr::prelude::*;
use nostr_sdk::prelude::Client;

const SECRET_KEY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";

type CliResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
struct Options {
    relay: RelayUrl,
    mode: Mode,
}

#[derive(Debug)]
enum Input {
    File(PathBuf),
    Directory(PathBuf),
}

#[derive(Debug)]
enum Mode {
    Publish(Input),
    Reconstruct { manifest: EventId, out: PathBuf },
}

#[tokio::main]
async fn main() -> CliResult<()> {
    let options = parse_args()?;
    let keys = Keys::new(SecretKey::from_hex(SECRET_KEY_HEX)?);

    let client = Client::default();
    client.add_relay(&options.relay).and_connect().await?;

    match options.mode {
        Mode::Publish(Input::File(path)) => {
            let base = path.parent().unwrap_or_else(|| Path::new("."));
            push_file(&client, &options.relay, &keys, base, &path).await?
        }
        Mode::Publish(Input::Directory(path)) => {
            push_directory(&client, &options.relay, &keys, &path).await?
        }
        Mode::Reconstruct { manifest, out } => {
            reconstruct_from_manifest(&client, &options.relay, manifest, &out).await?
        }
    }

    client.shutdown().await;
    Ok(())
}

fn parse_args() -> CliResult<Options> {
    let mut relay: Option<RelayUrl> = None;
    let mut file: Option<PathBuf> = None;
    let mut directory: Option<PathBuf> = None;
    let mut manifest: Option<EventId> = None;
    let mut out: Option<PathBuf> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--relay" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage_error("--relay requires a value"))?;
                relay = Some(RelayUrl::parse(&value)?);
            }
            "--file" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage_error("--file requires a value"))?;
                file = Some(PathBuf::from(value));
            }
            "--directory" | "--folder" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage_error("--directory requires a value"))?;
                directory = Some(PathBuf::from(value));
            }
            "--manifest" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage_error("--manifest requires a value"))?;
                manifest = Some(EventId::parse(&value)?);
            }
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| usage_error("--out requires a value"))?;
                out = Some(PathBuf::from(value));
            }
            other => {
                return Err(usage_error(format!("unknown argument: {other}")));
            }
        }
    }

    let relay = relay.ok_or_else(|| usage_error("--relay is required"))?;

    let mode = match (file, directory, manifest, out) {
        (Some(file), None, None, None) => Mode::Publish(Input::File(file)),
        (None, Some(directory), None, None) => Mode::Publish(Input::Directory(directory)),
        (None, None, Some(manifest), Some(out)) => Mode::Reconstruct { manifest, out },
        (Some(_), Some(_), _, _) => {
            return Err(usage_error("use either --file or --directory/--folder"));
        }
        (_, _, Some(_), None) => return Err(usage_error("--out is required with --manifest")),
        (_, _, None, Some(_)) => return Err(usage_error("--manifest is required with --out")),
        _ => {
            return Err(usage_error(
                "provide either --file/--directory/--folder or --manifest/--out",
            ));
        }
    };

    Ok(Options { relay, mode })
}

fn print_usage() {
    eprintln!(
        "Usage:\n  cargo run -p nostr --example nip-pip -- --file <path> --relay <ws(s)://relay>\n  cargo run -p nostr --example nip-pip -- --directory <path> --relay <ws(s)://relay>\n  cargo run -p nostr --example nip-pip -- --folder <path> --relay <ws(s)://relay>\n  cargo run -p nostr --example nip-pip -- --manifest <event-id> --out <path> --relay <ws(s)://relay>"
    );
}

fn usage_error(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message.into(),
    ))
}

async fn push_directory(
    client: &Client,
    relay: &RelayUrl,
    keys: &Keys,
    directory: &Path,
) -> CliResult<()> {
    let mut files = Vec::new();
    collect_files(directory, &mut files)?;
    files.sort();

    for file in files {
        push_file(client, relay, keys, directory, &file).await?;
    }

    Ok(())
}

async fn push_file(
    client: &Client,
    relay: &RelayUrl,
    keys: &Keys,
    base: &Path,
    file: &Path,
) -> CliResult<()> {
    let bytes = fs::read(file)?;
    let relative = file
        .strip_prefix(base)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    let root_id = packet_root_id(&relative);
    let batch = packetize(root_id.clone(), bytes.clone());
    let mut packet_events = Vec::with_capacity(batch.packets.len());
    let mut packet_event_ids = Vec::with_capacity(batch.total_packets as usize);
    let manifest = PacketManifest {
        root: root_id,
        sha256: sha256::Hash::hash(&bytes).to_string(),
        size: bytes.len() as u64,
        packets: batch.total_packets as u64,
        depth: packet_depth(&batch.packets),
        mtu: MTU_PAYLOAD as u64,
        encoding: String::from("json"),
        path: relative.clone(),
    };
    println!(
        "{}",
        serde_json::json!({
            "event": "perfect_ip file push start",
            "file": relative,
            "relay": relay.as_str(),
            "bytes": bytes.len(),
            "packets": batch.total_packets,
        })
    );

    for slice in batch.packets {
        let event = packet_slice_to_event(&slice)?.finalize(keys)?;
        let event_id = event.id.to_string();

        let output = client.send_event(&event).to([relay]).await?;
        packet_event_ids.push(event.id);
        packet_events.push(event);
        println!(
            "{}",
            serde_json::json!({
                "event": "perfect_ip packet pushed",
                "file": relative,
                "packet": slice.id,
                "event_id": event_id,
                "relay": relay.as_str(),
                "ok_relays": output.success.len(),
                "failed_relays": output.failed.len(),
            })
        );
    }

    let manifest_event = packet_manifest_to_event(&manifest, &packet_event_ids)?.finalize(keys)?;
    println!("{}", manifest_event.as_json());
    let manifest_output = client.send_event(&manifest_event).to([relay]).await?;
    println!(
        "{}",
        serde_json::json!({
            "event": "perfect_ip manifest pushed",
            "file": relative,
            "event_id": manifest_event.id.to_string(),
            "relay": relay.as_str(),
            "ok_relays": manifest_output.success.len(),
            "failed_relays": manifest_output.failed.len(),
        })
    );
    let reconstructed = reconstruct_payload_from_fetched_events(&packet_events)?;
    let reconstructed_hash = sha256::Hash::hash(&reconstructed).to_string();
    println!(
        "{}",
        serde_json::json!({
            "event": "perfect_ip file reconstructed",
            "file": relative,
            "bytes": reconstructed.len(),
            "matches": reconstructed == bytes,
            "sha256_matches": reconstructed_hash == manifest.sha256,
        })
    );

    println!(
        "{}",
        serde_json::json!({
            "event": "perfect_ip file push complete",
            "file": relative,
            "relay": relay.as_str(),
        })
    );

    Ok(())
}

async fn reconstruct_from_manifest(
    client: &Client,
    relay: &RelayUrl,
    manifest_id: EventId,
    out: &Path,
) -> CliResult<()> {
    let manifest_filter = Filter::new().kind(PIP_MANIFEST_KIND).ids([manifest_id]);
    let manifest_events = client.fetch_events(manifest_filter).await?;
    let manifest_event = manifest_events
        .first()
        .cloned()
        .ok_or_else(|| usage_error("manifest event not found"))?;
    let manifest: PacketManifest = serde_json::from_str(&manifest_event.content)?;
    let packet_event_ids = manifest_packet_event_ids(&manifest_event);
    let packet_event_count = packet_event_ids.len();
    if packet_event_ids.is_empty() {
        return Err(usage_error(
            "manifest event does not reference any packet events",
        ));
    }

    let packet_filter = Filter::new()
        .kind(PIP_SLICE_KIND)
        .ids(packet_event_ids.clone());
    let packet_events = client.fetch_events(packet_filter).await?;
    let packet_events = packet_events.into_iter().collect::<Vec<_>>();
    let reconstructed = reconstruct_payload_from_fetched_events(&packet_events)?;
    let reconstructed_hash = sha256::Hash::hash(&reconstructed).to_string();
    let output_path = resolve_output_path(out, &manifest);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, &reconstructed)?;

    println!(
        "{}",
        serde_json::json!({
            "event": "perfect_ip manifest fetched",
            "relay": relay.as_str(),
            "manifest": manifest_id.to_string(),
            "manifest_event": manifest_event.id.to_string(),
            "packet_events": packet_event_count,
            "out": output_path.display().to_string(),
            "bytes": reconstructed.len(),
            "matches": reconstructed_hash == manifest.sha256,
        })
    );

    Ok(())
}

fn reconstruct_payload_from_fetched_events(events: &[Event]) -> CliResult<Vec<u8>> {
    let mut slices: Vec<ProtocolSlice> = Vec::new();

    for event in events {
        if let Some(slice) = packet_slice_from_event(event)? {
            if !slice.is_parity {
                slices.push(slice);
            }
        }
    }

    slices.sort_by_key(|slice| slice.header.seq_num);

    Ok(slices.into_iter().flat_map(|slice| slice.data).collect())
}

fn resolve_output_path(out: &Path, manifest: &PacketManifest) -> PathBuf {
    let manifest_name = Path::new(&manifest.path)
        .file_name()
        .unwrap_or_else(|| OsStr::new(&manifest.path));

    if out.is_dir() || out.extension().is_none() {
        out.join(manifest_name)
    } else {
        out.to_path_buf()
    }
}

fn packet_depth(packets: &[ProtocolSlice]) -> u32 {
    packets
        .iter()
        .map(|slice| slice.id.matches('.').count() as u32)
        .max()
        .unwrap_or(0)
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> CliResult<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }

    Ok(())
}

fn packet_root_id(relative: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;

    for byte in relative.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("ROOT.{hash:016x}")
}
