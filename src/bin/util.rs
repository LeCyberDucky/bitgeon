use crossbeam_channel::{self, TrySendError};

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

