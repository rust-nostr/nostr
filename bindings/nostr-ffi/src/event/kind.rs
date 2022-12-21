// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use nostr::event::kind::{Kind as KindSdk, KindBase};

pub enum Kind {
    Base { kind: KindBase },
    Custom { kind: u64 },
}

impl From<KindSdk> for Kind {
    fn from(kind: KindSdk) -> Self {
        match kind {
            KindSdk::Base(kind) => Self::Base { kind },
            KindSdk::Custom(kind) => Self::Custom { kind },
        }
    }
}

impl From<Kind> for KindSdk {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Base { kind } => Self::Base(kind),
            Kind::Custom { kind } => Self::Custom(kind),
        }
    }
}
