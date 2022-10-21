// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::KindBase;

pub enum Kind {
    Base { kind: KindBase },
    Custom { kind: u16 },
}
