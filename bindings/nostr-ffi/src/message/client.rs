// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub enum ClientMessage {
    Ev {
        event: String,
    },
    Req {
        subscription_id: String,
        filters: Vec<String>,
    },
    Count {
        subscription_id: String,
        filters: Vec<String>,
    },
    Close {
        subscription_id: String,
    },
    Auth {
        event: String,
    },
}
