use anyhow::Result;
use crossbeam_channel::{self, TrySendError};

use std::thread;
use std::time;

#[derive(Clone)]
pub struct ThreadChannel<TX, RX> {
    pub sender: crossbeam_channel::Sender<TX>,
    pub receiver: crossbeam_channel::Receiver<RX>,
}

impl<TX, RX> ThreadChannel<TX, RX> {
    pub fn new(
        sender: crossbeam_channel::Sender<TX>,
        receiver: crossbeam_channel::Receiver<RX>,
    ) -> Self {
        Self { sender, receiver }
    }

    pub fn new_pair() -> (ThreadChannel<TX, RX>, ThreadChannel<RX, TX>) {
        let (a_tx, b_rx) = crossbeam_channel::unbounded::<TX>();
        let (b_tx, a_rx) = crossbeam_channel::unbounded::<RX>();

        let a = ThreadChannel::new(a_tx, a_rx);
        let b = ThreadChannel::new(b_tx, b_rx);

        (a, b)
    }

    pub fn send(&self, message: TX) -> Result<(), TrySendError<TX>> {
        self.sender.try_send(message)
    }

    pub fn receive(&self) -> Vec<RX> {
        self.receiver.try_iter().collect()
    }
}

// TODO: I really don't like all the type casting going on below. Look into that.

pub fn period_elapsed(clock: &time::Instant, count: &u64, rate: &u16) -> bool {
    clock.elapsed().as_micros() >= *count as u128 * 1000 / *rate as u128 // Should we be using floating point values here?
}

pub fn sleep_remaining_frame(clock: &time::Instant, count: &mut u128, rate: u128) {
    *count += 1;

    let micros_per_second = 1_000_000;
    let expected_frame_count = clock.elapsed().as_micros() * rate;
    let frame_count = *count * micros_per_second;

    // If this is positive, we should sleep the difference away
    let count_delta = (frame_count as i128) - (expected_frame_count as i128);

    if count_delta > 0 {
        let sleep_time = (count_delta as u128) / rate;
        thread::sleep(time::Duration::from_micros(sleep_time as u64));
    }
}
