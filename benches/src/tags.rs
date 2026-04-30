// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::hint::black_box;

use criterion::{BatchSize, Criterion};
use nostr::prelude::*;

fn generate_tags(n: usize) -> Tags {
    let half = n / 2;

    let mut pubkeys = Vec::with_capacity(half);

    let mut tags = Vec::with_capacity(n);

    for _ in 0..half {
        let keys = Keys::generate();

        // Save pubkey
        pubkeys.push(keys.public_key());

        // Push simple p tag
        tags.push(Tag::public_key(keys.public_key()));
    }

    for pk in pubkeys.into_iter() {
        // Push long p tag
        let long_p_tag = Tag::from_standardized_without_cell(TagStandard::PublicKey {
            public_key: pk,
            relay_url: Some(RelayUrl::parse("wss://relay.damus.io").unwrap()),
            uppercase: false,
            alias: None,
        });
        tags.push(long_p_tag)
    }

    Tags::from_list(tags)
}

fn generate_keys(c: &mut Criterion) {
    c.bench_function("generate_keys", |bh| {
        bh.iter(|| black_box(Keys::generate()))
    });
}

fn tags_dedup_10_tags(c: &mut Criterion) {
    let tags = generate_tags(10);
    c.bench_function("tags_dedup_10_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags)
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_dedup_50_tags(c: &mut Criterion) {
    let tags = generate_tags(50);

    c.bench_function("tags_dedup_50_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags);
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_dedup_100_tags(c: &mut Criterion) {
    let tags = generate_tags(100);

    c.bench_function("tags_dedup_100_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags);
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_dedup_500_tags(c: &mut Criterion) {
    let tags = generate_tags(500);

    c.bench_function("tags_dedup_500_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags);
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_dedup_1000_tags(c: &mut Criterion) {
    let tags = generate_tags(1000);

    c.bench_function("tags_dedup_1000_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags);
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_dedup_2000_tags(c: &mut Criterion) {
    let tags = generate_tags(2000);

    c.bench_function("tags_dedup_2000_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags);
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_dedup_4000_tags(c: &mut Criterion) {
    let tags = generate_tags(4000);

    c.bench_function("tags_dedup_4000_tags", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.dedup();
                black_box(tags);
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_push(c: &mut Criterion) {
    let tags = Tags::with_capacity(10);

    c.bench_function("tags_push", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.push(Tag::protected());
                black_box(tags)
            },
            BatchSize::SmallInput,
        )
    });
}

fn vec_tag_push(c: &mut Criterion) {
    let tags = Vec::with_capacity(10);

    c.bench_function("vec_tag_push", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                tags.push(Tag::protected());
                black_box(tags)
            },
            BatchSize::SmallInput,
        )
    });
}

fn tags_pop(c: &mut Criterion) {
    let tags = generate_tags(4000);

    c.bench_function("tags_pop", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                black_box(tags.pop());
            },
            BatchSize::SmallInput,
        )
    });
}

fn vec_tag_pop(c: &mut Criterion) {
    let tags = generate_tags(4000);
    let tags = tags.to_vec();

    c.bench_function("vec_tag_pop", |bh| {
        bh.iter_batched(
            || tags.clone(),
            |mut tags| {
                black_box(tags.pop());
            },
            BatchSize::SmallInput,
        )
    });
}

fn get_tag_kind(c: &mut Criterion) {
    let tag = Tag::identifier("id");
    c.bench_function("get_tag_kind", |bh| bh.iter(|| black_box(tag.kind())));
}

fn parse_p_tag(c: &mut Criterion) {
    let tag = [
        "p",
        "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
    ];
    c.bench_function("parse_p_tag", |bh| {
        bh.iter(|| black_box(Tag::parse(tag)).unwrap())
    });
}

fn parse_p_standardized_tag(c: &mut Criterion) {
    let tag = &[
        "p",
        "13adc511de7e1cfcf1c6b7f6365fb5a03442d7bcacf565ea57fa7770912c023d",
    ];
    c.bench_function("parse_p_standardized_tag", |bh| {
        bh.iter(|| black_box(TagStandard::parse(tag)).unwrap())
    });
}

fn parse_e_tag(c: &mut Criterion) {
    let tag = [
        "e",
        "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
        "wss://relay.damus.io",
    ];
    c.bench_function("parse_e_tag", |bh| {
        bh.iter(|| black_box(Tag::parse(tag)).unwrap())
    });
}

fn parse_e_standardized_tag(c: &mut Criterion) {
    let tag = &[
        "e",
        "378f145897eea948952674269945e88612420db35791784abf0616b4fed56ef7",
        "wss://relay.damus.io",
    ];
    c.bench_function("parse_e_standardized_tag", |bh| {
        bh.iter(|| black_box(TagStandard::parse(tag)).unwrap())
    });
}

fn parse_a_tag(c: &mut Criterion) {
    let tag = [
        "a",
        "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
        "wss://relay.nostr.org",
    ];
    c.bench_function("parse_a_tag", |bh| {
        bh.iter(|| black_box(Tag::parse(tag)).unwrap())
    });
}

fn parse_t_tag(c: &mut Criterion) {
    let tag = ["t", "test"];
    c.bench_function("parse_t_tag", |bh| {
        bh.iter(|| black_box(Tag::parse(tag)).unwrap())
    });
}

fn parse_a_standardized_tag(c: &mut Criterion) {
    let tag = &[
        "a",
        "30023:a695f6b60119d9521934a691347d9f78e8770b56da16bb255ee286ddf9fda919:ipsum",
        "wss://relay.nostr.org",
    ];
    c.bench_function("parse_a_standardized_tag", |bh| {
        bh.iter(|| black_box(TagStandard::parse(tag)).unwrap())
    });
}

pub fn benches(c: &mut Criterion) {
    generate_keys(c);

    tags_dedup_10_tags(c);
    tags_dedup_50_tags(c);
    tags_dedup_100_tags(c);
    tags_dedup_500_tags(c);
    tags_dedup_1000_tags(c);
    tags_dedup_2000_tags(c);
    tags_dedup_4000_tags(c);
    tags_push(c);
    vec_tag_push(c);
    tags_pop(c);
    vec_tag_pop(c);

    get_tag_kind(c);
    parse_p_tag(c);
    parse_p_standardized_tag(c);
    parse_e_tag(c);
    parse_e_standardized_tag(c);
    parse_a_tag(c);
    parse_t_tag(c);
    parse_a_standardized_tag(c);
}
