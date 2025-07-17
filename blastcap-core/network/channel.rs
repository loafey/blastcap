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

pub struct DisjointChannel<A, B> {
    send: Sender<A>,
    recv: Receiver<B>,
}
impl<A, B> DisjointChannel<A, B> {
    pub async fn recv(&mut self) -> Option<B> {
        self.recv.recv().await
    }
    pub fn try_recv(&mut self) -> Result<B, tokio::sync::mpsc::error::TryRecvError> {
        self.recv.try_recv()
    }
    pub async fn send(&self, t: A) -> Result<(), tokio::sync::mpsc::error::SendError<A>> {
        self.send.send(t).await
    }
}
pub fn disjoint<A, B>(buffer: usize) -> (DisjointChannel<A, B>, DisjointChannel<B, A>) {
    let (a_send, a_recv) = channel(buffer);
    let (b_send, b_recv) = channel(buffer);
    (
        DisjointChannel {
            send: a_send,
            recv: b_recv,
        },
        DisjointChannel {
            send: b_send,
            recv: a_recv,
        },
    )
}
