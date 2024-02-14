// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use nostr_sdk::Event;
use prettytable::{row, Table};

pub fn print_events<I>(events: I)
where
    I: IntoIterator<Item = Event>,
{
    let mut table = Table::new();

    table.set_titles(row!["#", "ID", "Author", "Kind", "Created At",]);

    for (index, event) in events.into_iter().enumerate() {
        table.add_row(row![
            index + 1,
            event.id,
            event.author(),
            event.kind(),
            event.created_at.to_human_datetime()
        ]);
    }

    table.printstd();
}
