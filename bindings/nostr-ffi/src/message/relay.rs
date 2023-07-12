// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub enum RelayMessage {
    Ev {
        subscription_id: String,
        event: String,
    },
    Notice {
        message: String,
    },
    EndOfStoredEvents {
        subscription_id: String,
    },
    Ok {
        event_id: String,
        status: bool,
        message: String,
    },
    Auth {
        challenge: String,
    },
    Count {
        subscription_id: String,
        count: u64,
    },
}
