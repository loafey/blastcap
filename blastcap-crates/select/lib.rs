#![feature(macro_metavar_expr)]
use futures_concurrency::{stream::Merge, vec};
use smol::{
    future::FutureExt,
    stream::{Stream, StreamExt},
};
use std::{pin::Pin, task::Poll};

#[macro_export]
macro_rules! repeat {
    ($(async $y:expr),*,) => {
        repeat!($($y),*)
    };
    ($(async $y:expr),*) => {
        {
            let mut a = select::select();
            $(a = a.add(async || $y));*;
            a.finish()
        }
    };
}

pub struct SelectBuilder<'l, T> {
    inner: Vec<Repeat<'l, T>>,
}
impl<'l, T> SelectBuilder<'l, T> {
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add<P: FnMut() -> R + 'l + 'static + Send, R: Future<Output = T> + 'l + Send>(
        mut self,
        func: P,
    ) -> Self {
        self.inner.push(Repeat::new(func));
        self
    }
    #[must_use]
    pub fn finish(self) -> Select<'l, T> {
        Select {
            inner: self.inner.merge(),
        }
    }
}
pub struct Select<'l, T> {
    inner: vec::Merge<Repeat<'l, T>>,
}
impl<'l, T> Stream for Select<'l, T> {
    type Item = T;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next(cx)
    }
}

pub fn select<'l, T>() -> SelectBuilder<'l, T> {
    SelectBuilder { inner: Vec::new() }
}

pub struct Repeat<'l, T: 'l> {
    func: Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T> + 'l + Send>> + Send>,
    inner: RepeatInner<'l, T>,
}
impl<'l, T: 'l> Repeat<'l, T> {
    pub fn new<P: FnMut() -> R + 'l + 'static + Send, R: Future<Output = T> + 'l + Send>(
        mut func: P,
    ) -> Self {
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
    Spawned(Pin<Box<dyn Future<Output = T> + 'l + Send>>),
}

pub mod results;

#[macro_export]
macro_rules! select {
    ($(($input:expr, |$pat:pat_param| $func:expr)),*) => {{
        paste::paste! {
            use futures_concurrency::future::Race as _;
            use select::results::[<SelectionResult ${count($input)}>] as __ReturnType;
            let futures = ($(async { __ReturnType::[<Res ${index()}>] ($input.await)}),*).race();
            match futures.await {
                $(__ReturnType::[<Res ${index()}>]($pat) => $func),*
            }
        }
    }};
}
