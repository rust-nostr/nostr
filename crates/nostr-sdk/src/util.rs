// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Util

use nostr::url::{ParseError, Url};

/// Try into [`Url`]
pub trait TryIntoUrl {
    /// Error
    type Err;
    /// Try into [`Url`]
    fn try_into_url(&self) -> Result<Url, Self::Err>;
}

impl TryIntoUrl for Url {
    type Err = ParseError;
    fn try_into_url(&self) -> Result<Url, Self::Err> {
        Ok(self.clone())
    }
}

impl TryIntoUrl for &Url {
    type Err = ParseError;
    fn try_into_url(&self) -> Result<Url, Self::Err> {
        Ok(<&Url>::clone(self).clone())
    }
}

impl TryIntoUrl for String {
    type Err = ParseError;
    fn try_into_url(&self) -> Result<Url, Self::Err> {
        Url::parse(self)
    }
}

impl TryIntoUrl for &str {
    type Err = ParseError;
    fn try_into_url(&self) -> Result<Url, Self::Err> {
        Url::parse(self)
    }
}
