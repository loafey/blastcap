use smol::{future::FutureExt, stream::Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

#[test]
fn stream() {
    use select::repeat;
    use smol::{Timer, stream::StreamExt as _};
    use std::time::{Duration, Instant};
    smol::block_on(async {
        let mut streams = repeat!(
            async {
                Timer::after(Duration::from_secs(1)).await;
                1
            },
            async {
                Timer::after(Duration::from_secs(2)).await;
                2
            }
        );

        let time = Instant::now();
        let mut sum = 0;
        while let Some(p) = streams.next().await {
            println!("{p}, {:0.001}s", time.elapsed().as_secs_f32());
            sum += p;
            if time.elapsed().as_secs() >= 5 {
                println!("{sum}");
                break;
            }
        }
    });
}

struct UnorderedInner<'l, T> {
    inner: Option<Pin<Box<dyn Future<Output = T> + 'l>>>,
}
impl<'l, T> Stream for UnorderedInner<'l, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.inner {
            Some(i) => match i.poll(cx) {
                Poll::Ready(f) => {
                    self.inner = None;
                    Poll::Ready(Some(f))
                }
                Poll::Pending => Poll::Pending,
            },
            None => Poll::Ready(None),
        }
    }
}
pub struct Unordered<'l, T> {
    _inner: futures_concurrency::vec::Merge<UnorderedInner<'l, T>>,
}
fn main() {}
