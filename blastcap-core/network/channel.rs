use smol::channel;

pub struct Channel<T> {
    send: channel::Sender<T>,
    recv: channel::Receiver<T>,
}
impl<T> Channel<T> {
    pub fn new() -> Self {
        let (send, recv) = channel::unbounded();
        Self { send, recv }
    }
    pub async fn recv(&mut self) -> Option<T> {
        self.recv.recv().await.ok()
    }
    pub fn try_recv(&mut self) -> Result<T, channel::TryRecvError> {
        self.recv.try_recv()
    }
    pub fn sender(&self) -> channel::Sender<T> {
        self.send.clone()
    }
    pub async fn send(&self, t: T) -> Result<(), channel::SendError<T>> {
        self.send.send(t).await
    }
}

pub struct DisjointChannel<S, R> {
    send: channel::Sender<S>,
    recv: channel::Receiver<R>,
}
impl<S, R> DisjointChannel<S, R> {
    pub async fn recv(&mut self) -> Option<R> {
        self.recv.recv().await.ok()
    }
    #[allow(unused)]
    pub fn try_recv(&mut self) -> Result<R, channel::TryRecvError> {
        self.recv.try_recv()
    }
    pub async fn send(&self, t: S) -> Result<(), channel::SendError<S>> {
        self.send.send(t).await
    }
}
pub fn disjoint<A, B>() -> (DisjointChannel<A, B>, DisjointChannel<B, A>) {
    let (a_send, a_recv) = channel::unbounded();
    let (b_send, b_recv) = channel::unbounded();
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
