use std::io::Write;

use rand::Rng;
use random::{POOL_SIZE, Random};

/// this generation is not cryptographically secure, and should not be treated as such :)
fn main() {
    // let mut a = Random::new(rand::random());
    // for _ in 0..20 {
    //     println!("{:?}", a.get_range_float::<f32>(0.0..65565.0));
    // }

    let path = "blastcap-crates/random/rand.bin";
    if std::fs::exists(path).unwrap() {
        eprintln!("rand file already exists, please remove to rerun it");
        std::process::exit(1);
    }
    let mut attempts = 0;
    let pool = loop {
        println!("Attempt {attempts}...");
        let pool = gen_pool();
        let (ptr, len, cap) = pool.into_raw_parts();
        let source = unsafe { std::slice::from_raw_parts(ptr, len) };
        let result = tests(source);
        let pool = unsafe { Vec::from_raw_parts(ptr, len, cap) };
        if let Err(e) = result {
            eprintln!(" - failed on: {e}")
        } else {
            break pool;
        }
        attempts += 1;
    };

    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&pool).unwrap();
}

type Test = Result<(), &'static str>;
fn tests(source: &'static [u8]) -> Test {
    fn repeats(seed: u64, source: &'static [u8]) -> Test {
        let mut random = Random::with_pool(seed, source);
        let base = random.get_u8();
        for _ in 0..POOL_SIZE - 1 {
            _ = random.get_u8();
        }
        let end = random.get_u8();
        if base == end { Err("repeat") } else { Ok(()) }
    }

    fn distr(seed: u64, source: &'static [u8]) -> Test {
        let mut random = Random::with_pool(seed, source);
        let mut count = [0; 255];
        let sample = 10000;
        for _ in 0..sample {
            let u = random.get_u8() as usize;
            count[u] += 1;
        }
        let mean = count.iter().sum::<usize>() / count.len();
        let mean_mid = sample / count.len();
        let mean_off = mean_mid / 4;
        if !(mean >= mean_mid - mean_off && mean <= mean_mid + mean_off) {
            return Err("u8: bad mean");
        }

        count.sort();
        let median = count[count.len() / 2];
        if !(median >= mean_mid - mean_off && median <= mean_mid + mean_off) {
            return Err("u8: bad median");
        }

        Ok(())
    }

    let mut random = rand::rng();

    for _ in 0..1024 {
        let seed = random.random::<u64>();
        repeats(seed, source)?;
        distr(seed, source)?;
    }
    Ok(())
}

pub fn gen_pool() -> Vec<u8> {
    let mut rng = rand::rng();
    let mut pool = Vec::with_capacity(POOL_SIZE);
    for _ in 0..POOL_SIZE {
        pool.push(rng.random::<u8>());
    }
    pool
}
