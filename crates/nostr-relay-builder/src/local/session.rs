// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::HashMap;
use std::time::{Duration, Instant};

use nostr::{Filter, SubscriptionId};

pub(super) enum RateLimiterResponse {
    Allowed,
    Limited,
}

pub(super) struct Session {
    pub subscriptions: HashMap<SubscriptionId, Vec<Filter>>,
    pub tokens: Tokens,
}

impl Session {
    const MIN: Duration = Duration::from_secs(60);

    fn calculate_elapsed_time(&self, now: Instant, last: Instant) -> Duration {
        let mut elapsed_time: Duration = now - last;

        if elapsed_time > Self::MIN {
            elapsed_time = Self::MIN;
        }

        elapsed_time
    }

    pub fn check_rate_limit(&mut self, max_per_minute: u32) -> RateLimiterResponse {
        match self.tokens.last {
            Some(last) => {
                let now: Instant = Instant::now();
                let elapsed_time: Duration = self.calculate_elapsed_time(now, last);

                self.tokens
                    .calculate_new_tokens(max_per_minute, elapsed_time);

                if self.tokens.count == 0 {
                    return RateLimiterResponse::Limited;
                }

                self.tokens.last = Some(now);

                RateLimiterResponse::Allowed
            }
            None => {
                self.tokens.last = Some(Instant::now());
                RateLimiterResponse::Allowed
            }
        }
    }
}

/// Tokens to keep track of session limits
pub(super) struct Tokens {
    pub count: u32,
    pub last: Option<Instant>,
}

impl Tokens {
    #[inline]
    pub fn new(tokens: u32) -> Self {
        Self {
            count: tokens,
            last: None,
        }
    }

    fn calculate_new_tokens(&mut self, max_per_minute: u32, elapsed_time: Duration) {
        let percent: f32 = (elapsed_time.as_secs() as f32) / 60.0;
        let new_tokens: u32 = (percent * max_per_minute as f32).floor() as u32;

        self.count = self.count.saturating_add(new_tokens);

        self.count = self.count.saturating_sub(1);

        if self.count >= max_per_minute {
            self.count = max_per_minute.saturating_sub(1);
        }
    }
}
