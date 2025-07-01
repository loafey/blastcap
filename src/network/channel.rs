use tokio::sync::mpsc::{Receiver, Sender, channel};

pub struct Channel<T> {
    pub send: Sender<T>,
    pub recv: Receiver<T>,
}
impl<T> Channel<T> {
    pub fn new(buffer: usize) -> Self {
        let (send, recv) = channel(buffer);
        Self { send, recv }
    }
}
