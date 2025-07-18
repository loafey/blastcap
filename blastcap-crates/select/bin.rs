#![feature(macro_metavar_expr)]

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

macro_rules! select {
    ($(($input:expr, |$pat:pat_param| $func:expr)),*) => {{
        paste::paste! {
            use futures_concurrency::future::Race as _;
            use results::*;
            mac
            let futures = ($(async { [<SelectionResult ${count($input)}>]::[<Res ${index()}>] ($input.await)}),*).race();
            match futures.await {
                $([<SelectionResult ${len()}>]::[<Res ${index()}>]($pat) => $func),*
            }
        }
    }};
}

pub mod results {
    pub enum SelectionResult1<A> {
        Res1(A),
    }
    pub enum SelectionResult2<A, B> {
        Res1(A),
        Res2(B),
    }
    pub enum SelectionResult3<A, B, C> {
        Res1(A),
        Res2(B),
        Res3(C),
    }
    pub enum SelectionResult4<A, B, C, D> {
        Res1(A),
        Res2(B),
        Res3(C),
        Res4(D),
    }
    pub enum SelectionResult5<A, B, C, D, E> {
        Res1(A),
        Res2(B),
        Res3(C),
        Res4(D),
        Res5(E),
    }
}

fn main() {
    smol::block_on(async {
        select!(
            (async { 1u32 }, |out| {}),
            (async { 2u32 }, |out| {
                println!("{out}");
            }),
            (async { 3u32 }, |out| {
                println!("{out}");
            })
        )
    });
}
