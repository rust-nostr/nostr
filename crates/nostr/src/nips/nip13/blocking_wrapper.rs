use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;

use crate::UnsignedEvent;

type Work = dyn FnOnce(Arc<AtomicBool>) -> Option<UnsignedEvent> + Send + 'static;

#[derive(Default)]
struct PowState {
    // - Some(v) = thread done, v is the result
    // - None = thread not done yet
    result: Mutex<Option<Option<UnsignedEvent>>>,
    waker: Mutex<Option<Waker>>,
}

pub(super) struct BlockingPowFuture {
    #[allow(clippy::type_complexity)]
    work: Option<Box<Work>>,
    state: Arc<PowState>,
    cancel: Arc<AtomicBool>,
    spawned: bool,
}

impl BlockingPowFuture {
    pub(super) fn new(work: Box<Work>) -> Self {
        Self {
            work: Some(work),
            state: Arc::new(PowState::default()),
            cancel: Arc::new(AtomicBool::new(false)),
            spawned: false,
        }
    }
}

impl Drop for BlockingPowFuture {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

impl Future for BlockingPowFuture {
    type Output = Option<UnsignedEvent>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::get_mut(self);

        // Always update the waker. The executor may provide a new one on each poll.
        // Drop the guard before acquiring the result lock to avoid deadlock.
        {
            let mut waker = this.state.waker.lock().unwrap();
            *waker = Some(cx.waker().clone());
        }

        if !this.spawned {
            this.spawned = true;

            let work: Box<Work> = this.work.take().unwrap();
            let state: Arc<PowState> = this.state.clone();
            let cancel: Arc<AtomicBool> = this.cancel.clone();

            thread::spawn(move || {
                let event: Option<UnsignedEvent> = work(cancel);

                // Release result lock before acquiring waker lock.
                {
                    let mut result = state.result.lock().unwrap();
                    *result = Some(event);
                }

                let mut waker = state.waker.lock().unwrap();
                if let Some(waker) = waker.take() {
                    waker.wake();
                }
            });
        }

        let mut result = this.state.result.lock().unwrap();

        match result.take() {
            Some(result) => Poll::Ready(result),
            None => Poll::Pending,
        }
    }
}
