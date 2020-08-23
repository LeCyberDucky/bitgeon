use crossbeam_channel::{self, TrySendError};
use std::thread;
use std::time;

#[derive(Clone)]
pub struct Channel<T> {
    pub sender: crossbeam_channel::Sender<T>,
    pub receiver: crossbeam_channel::Receiver<T>,
}

impl<T> Channel<T> {
    pub fn send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(message)
    }

    pub fn receive(&self) -> Vec<T> {
        self.receiver.try_iter().collect()
    }
}

// TODO: I really don't like all the type casting going on below. Look into that.

pub fn period_elapsed(clock: &time::Instant, count: &u64, rate: &u16) -> bool {
    clock.elapsed().as_micros() >= *count as u128 * 1000 / *rate as u128 // Should we be using floating point values here?
}

pub fn sleep_remaining_frame(clock: &time::Instant, count: &mut u64, rate: &u16) {
    // Note: This allows catching up via a high instantaneous frame rate after a lag. Maybe lost frames should just be skipped?
    *count += 1;

    let delta_t =
        (*count as f64 / *rate as f64 * 1000f64) as i128 - clock.elapsed().as_micros() as i128;

    if delta_t > 0 {
        let sleep_time = time::Duration::from_micros(delta_t as u64);
        thread::sleep(sleep_time);
    }
}
