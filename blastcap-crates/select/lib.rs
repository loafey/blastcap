use smol::{future::FutureExt, stream::Stream};
use std::{pin::Pin, task::Poll};

pub struct Repeat<'l, T: 'l> {
    func: Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T> + 'l>>>,
    inner: RepeatInner<'l, T>,
}
impl<'l, T: 'l> Repeat<'l, T> {
    pub fn new<P: FnMut() -> R + 'l + 'static, R: Future<Output = T> + 'l>(mut func: P) -> Self {
        Self {
            func: Box::new(move || Box::pin(func())),
            inner: RepeatInner::NotSpawned,
        }
    }
}

impl<'l, T: 'l> Future for Repeat<'l, T> {
    type Output = T;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match &mut self.inner {
            RepeatInner::NotSpawned => {
                self.inner = RepeatInner::Spawned(Box::pin((self.func)()));
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            RepeatInner::Spawned(future) => match future.poll(cx) {
                Poll::Ready(out) => {
                    self.inner = RepeatInner::Spawned(Box::pin((self.func)()));
                    Poll::Ready(out)
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
impl<'l, T: 'l> Stream for Repeat<'l, T> {
    type Item = T;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.poll(cx).map(|a| Some(a))
    }
}

enum RepeatInner<'l, T> {
    NotSpawned,
    Spawned(Pin<Box<dyn Future<Output = T> + 'l>>),
}
