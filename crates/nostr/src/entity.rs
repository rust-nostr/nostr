// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Entity {
    Account,
    Channel,
    Unknown,
}
