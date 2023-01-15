// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Entity {
    Account,
    Channel,
    Unknown,
}
