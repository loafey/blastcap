use futures_concurrency::stream::Merge;
use select::Repeat;
use smol::{Timer, stream::StreamExt as _};
use std::time::{Duration, Instant};

fn main() {
    smol::block_on(async {
        let rep1 = Repeat::new(async || {
            Timer::after(Duration::from_secs(1)).await;
            1
        });
        let rep2 = Repeat::new(async || {
            Timer::after(Duration::from_secs(2)).await;
            2
        });
        let mut streams = (rep1, rep2).merge();
        let time = Instant::now();
        let mut sum = 0;
        while let Some(p) = streams.next().await {
            println!("{p}, {:0.001}s", time.elapsed().as_secs_f32());
            sum += p;
            if time.elapsed().as_secs() >= 3 {
                println!("{sum}");
                break;
            }
        }
    });
}
