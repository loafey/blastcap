#![feature(macro_metavar_expr)]

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
        select::select!(
            (async { 1u32 }, |out| { println!("{out}") }),
            (async { 2u32 }, |out| { println!("{out}") }),
            (async { 3u32 }, |out| { println!("{out}") })
        )
    });
}
