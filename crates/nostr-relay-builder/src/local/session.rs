// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use negentropy::{Negentropy, NegentropyStorageVector};
use nostr::{Event, PublicKey, Result, SubscriptionId, Timestamp};

pub(super) enum RateLimiterResponse {
    Allowed,
    Limited,
}

#[derive(Default)]
pub(super) struct Nip42Session {
    /// Is authenticated
    pub public_key: Option<PublicKey>,
    /// Challenges
    pub challenges: HashSet<String>,
}

impl Nip42Session {
    /// Get or generate challenge
    pub fn generate_challenge(&mut self) -> String {
        // TODO: alternatives?

        // Too many challenges without reply
        if self.challenges.len() > 20 {
            // Clean to avoid possible attack where client never complete auth
            self.challenges.clear();
        }

        let challenge: String = SubscriptionId::generate().to_string();
        self.challenges.insert(challenge.clone());
        challenge
    }

    #[inline]
    pub fn is_authenticated(&self) -> bool {
        self.public_key.is_some()
    }

    pub fn check_challenge(&mut self, event: &Event) -> Result<(), String> {
        match event.tags.challenge() {
            Some(challenge) => {
                // Tried to remove challenge but wasn't in the set: return false.
                if !self.challenges.remove(challenge) {
                    return Err(String::from("received invalid challenge"));
                }

                // Check created_at
                let now = Timestamp::now();
                let diff: u64 = now.as_u64().abs_diff(event.created_at.as_u64());
                if diff > 120 {
                    return Err(String::from("challenge is too old (max allowed 2 min)"));
                }

                // Verify event
                event.verify().map_err(|e| e.to_string())?;

                // TODO: check `relay` tag

                // Mark as authenticated
                self.public_key = Some(event.pubkey);

                Ok(())
            }
            None => Err(String::from("challenge not found")),
        }
    }
}

pub(super) struct Session {
    pub negentropy_subscription: HashMap<SubscriptionId, Negentropy<NegentropyStorageVector>>,
    pub nip42: Nip42Session,
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
