#![feature(macro_metavar_expr)]

use smol::{
    Timer,
    future::FutureExt,
    stream::{Stream, StreamExt},
};
use std::time::{Duration, Instant};

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

fn main() {
    smol::block_on(async {
        let mut sec1 = select::Interval::new(Duration::from_secs(1));
        let mut sec2 = select::Interval::new(Duration::from_secs(2));
        let timer = Instant::now();
        loop {
            select::select!(
                (sec1.next(), |d| {
                    let d = d.unwrap();
                    println!(
                        "1 second - since_last: {:0.5}s, total_time: {:0.5}s",
                        d.as_secs_f32(),
                        timer.elapsed().as_secs_f32()
                    );
                }),
                (sec2.next(), |d| {
                    let d = d.unwrap();
                    println!(
                        "2 second - since_last: {:0.5}s, total_time: {:0.5}s",
                        d.as_secs_f32(),
                        timer.elapsed().as_secs_f32()
                    );
                })
            );
        }
    });
}
