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

pub fn sleep_remaining_frame(clock: &time::Instant, count: &u64, rate: &u16) {
    let sleep_time: i128 =
        ((*count as i128 + 1) * *rate as i128 * 1000 - clock.elapsed().as_micros() as i128) as i128;
    if sleep_time > 0 {
        let sleep_time = time::Duration::from_micros(sleep_time as u64);
        thread::sleep(sleep_time);
    }
}
