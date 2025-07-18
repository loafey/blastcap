use smol::{future::FutureExt, stream::Stream};
use std::{
    cell::RefCell,
    marker::PhantomData,
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

#[allow(clippy::type_complexity)]
struct Scope<'env, 'scope, T> {
    inner: RefCell<Vec<Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T> + 'env>> + 'env>>>,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

impl<'env, 'scope, T> Default for Scope<'env, 'scope, T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            scope: Default::default(),
            env: Default::default(),
        }
    }
}
impl<'scope, 'env, T> Scope<'scope, 'env, T> {
    pub fn add<F, R>(&'env self, mut func: F)
    where
        F: FnMut() -> R + 'scope,
        R: Future<Output = T> + 'scope,
    {
        self.inner
            .borrow_mut()
            .push(Box::new(move || Box::pin(func())));
    }
}
fn scope<'env, T, F: for<'scope> FnOnce(&'scope Scope<'env, 'scope, T>)>(func: F) {
    let mut i = Scope::default();
    func(&mut i);
}

fn main() {
    smol::block_on(async {
        let inc = 0;
        scope(|s| {
            s.add(|| async { inc + 1 });
            s.add(|| async { inc + 2 });
        });
        // repeat!(async { inc + 1 }, async { inc + 2 });
        // let mut stream = {
        //     let mut a = select::select();
        //     let a_fun = || {
        //         inc += 1;
        //         async {}
        //     };
        //     a = a.add(a_fun);
        //     // a = a.add(async || inc += 2);
        //     a.finish()
        // };
        // while let Some(_) = stream.next().await {
        //     println!("{inc}")
        // }
    })
}
