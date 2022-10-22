// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::sync::Arc;

pub(crate) fn unwrap_or_clone_arc<T: Clone>(arc: Arc<T>) -> T {
    Arc::try_unwrap(arc).unwrap_or_else(|x| (*x).clone())
}
