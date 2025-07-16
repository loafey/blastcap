use tokio::sync::mpsc::{Receiver, Sender, channel};

pub struct Channel<T> {
    send: Sender<T>,
    recv: Receiver<T>,
}
impl<T> Channel<T> {
    pub fn new(buffer: usize) -> Self {
        let (send, recv) = channel(buffer);
        Self { send, recv }
    }
    pub async fn recv(&mut self) -> Option<T> {
        self.recv.recv().await
    }
    pub fn try_recv(&mut self) -> Result<T, tokio::sync::mpsc::error::TryRecvError> {
        self.recv.try_recv()
    }
    pub fn sender(&self) -> Sender<T> {
        self.send.clone()
    }
    pub async fn send(&self, t: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        self.send.send(t).await
    }
}
