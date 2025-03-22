#![feature(iter_array_chunks)]
use mulmod_with_mont::*;

use rayon::prelude::*;

#[cfg(any())]
mod old {
    fn euler_sieve(n: u64) -> Vec<u64> {
        let mut is_prime = vec![true; n as usize + 1];
        let mut primes = Vec::new();
        for i in 2..=n {
            if is_prime[i as usize] {
                primes.push(i);
            }
            for &p in &primes {
                let m = i * p;
                if m > n {
                    break;
                }
                is_prime[m as usize] = false;
                if i % p == 0 {
                    break;
                }
            }
        }
        primes
    }

    fn check_with_parallel_sieve(limit: u64) {
        use rayon::iter::ParallelBridge;
        if limit < 100_0000 {
            return;
        }
        let sqrt_limit = limit.isqrt() + 1;
        // let small_primes = euler_sieve(sqrt_limit);
        let small_primes = euler_sieve(20);

        let block_size = sqrt_limit.max(100_0000);
        let start = sqrt_limit + 1;
        let end = limit;

        let blocks: Vec<(u64, u64)> = (start..=end)
            .step_by(block_size as usize)
            .map(|low| {
                let high = std::cmp::min(low + block_size - 1, end);
                (low, high)
            })
            .collect();

        blocks.into_iter().par_bridge().for_each(|(low, high)| {
            let r = low % 6;
            let x0 = match r {
                0 => low + 1,
                1 => low,
                2 => low + 3,
                3 => low + 2,
                4 => low + 1,
                5 => low,
                _ => unreachable!(),
            };

            let mut positions = Vec::new();
            let mut x = x0;
            let mut add = if x0 % 6 == 1 { 4 } else { 2 };
            while x <= high {
                positions.push(x);
                x += add;
                add = 6 - add;
            }

            let mut sieve = vec![true; positions.len()];

            for &p in small_primes.iter().skip(2) {
                let mut multiple = p * ((x0 + p - 1) / p);
                while multiple <= high {
                    if let Ok(index) = positions.binary_search(&multiple) {
                        sieve[index as usize] = false;
                    }
                    multiple += p;
                }
            }
            const REM:u64 =2;
            let mut iter = sieve
                .into_iter()
                .enumerate()
                .filter(|&(_, is_prime)| is_prime).map(|(i,_)|positions[i])
                .array_chunks();
                while let Some(i) = iter.next() {
                    let mont = s52::Mul52::new(s52::Base::new(i));
                    mont.mont_to_num(pow(mont.num_to_mont(s52::Base::THREE.0), mont)).iter().zip(mont.base.0.iter()).for_each(|(&x,&y)|{
                        if x == y - REM {
                            println!("Found {} (by s52)", y)
                        }
                    });

                }
                let _ = iter.into_remainder().map(|x|x.for_each(|p| {
                    let mont = mont::Mul64::new(p);
                    if mont.mont_to_num(pow(mont.num_to_mont(3), mont)) == mont.base - REM {
                        println!("Found {}", mont.base)
                    }
                }));
        })
    }
}


fn check_table_with_parallel(mut start: u64, mut end: u64) {
    const PRIMES:&[u64] = &[2,3,5,7,11,13,17,19,23];

    end = end.div_ceil(BLOCK_SIZE);
    start = (start / BLOCK_SIZE).max(1);
    let start_idx = start * BLOCK_SIZE;

    fn block_remain() -> (u64, u64) {
        (PRIMES.iter().product(), PRIMES.iter().map(|x|x-1).product())
    }
    let (block_size, remain_size) = block_remain();
    const BLOCK_SIZE:u64 = 223092870;
    const REMAIN_SIZE:u64 = 36495360;
    assert_eq!( block_size,  BLOCK_SIZE);
    assert_eq!(remain_size, REMAIN_SIZE);
    fn calc_idx(plus:u64) -> [u64;REMAIN_SIZE as usize] {
        let mut prime = [true;BLOCK_SIZE as usize];
        let mut res = [0u64;REMAIN_SIZE as usize];
        PRIMES.iter().for_each(|x|{
            for i in 0..BLOCK_SIZE/x {
                prime[(i * x) as usize] = false;
            }
        });
        assert_eq!(prime.iter().enumerate().fold(0, |s,(x,&b)|if b {res[s as usize] = plus + x as u64;s+1} else {s}), REMAIN_SIZE);
        res
    }
    let idx = calc_idx(start * BLOCK_SIZE);

    assert!(REMAIN_SIZE % 8 == 0,"REMAIN_SIZE (={REMAIN_SIZE}) 应该是8的倍数");
    (0..(end - start)).par_bridge().for_each(|blk_idx| {
        let offset = blk_idx * BLOCK_SIZE;
        let mut coef = Vec::new();
        const REM:u64 =4;
        let mut iter = idx.iter().map(|x|x + offset).array_chunks::<8>();
        while let Some(i) = iter.next() {
            let mont = s52::Mul52::new(s52::Base::new(i));
            mont.mont_to_num(pow(mont.num_to_mont(s52::Base::THREE.0), mont)).iter().zip(mont.base.0.iter()).for_each(|(&x,&y)|{
                if x == y - REM {
                    // println!("Found {} (by s52)", y)
                    coef.push(y)
                }
            });
        }
        println!("{}",format!("  Done block {blk_idx} {:5.2}% (from {} to {}), found {coef:?}", blk_idx as f64 / (end - start) as f64 * 100., start_idx + blk_idx * BLOCK_SIZE, start_idx + (blk_idx +1)* BLOCK_SIZE))
        // assert!(REMAIN_SIZE % 8 == 0,"由于REMAIN_SIZE (={REMAIN_SIZE}) 是8的倍数，没必要执行后续计算");
        // let _ = iter.into_remainder().map(|x|x.for_each(|p| {
        //     let mont = mont::Mul64::new(p);
        //     if mont.mont_to_num(pow(mont.num_to_mont(3), mont)) == mont.base - REM {
        //         println!("Found {}", mont.base)
        //     }
        // }));
    })
}

fn main() {
    std::thread::Builder::new().stack_size(1_000_000_000).spawn(||{
        let now = std::time::Instant::now();
        check_table_with_parallel(std::env::args().nth(2).map_or(None,|x|x.replace("_","").parse().ok()).unwrap_or(3),std::env::args().nth(1).map_or(None,|x|x.replace("_","").parse().ok()).unwrap_or(10_000_000_000));
        println!("cost {:?}", now.elapsed())
    }).expect("spawn failed").join().unwrap();
    // check_with_parallel_sieve(std::env::args().nth(1).as_deref().unwrap_or("").parse().unwrap_or(10_000_000_000));
    // check_with_parallel_sieve(std::env::args().nth(1).map_or(None,|x|x.parse().ok()).unwrap_or(10_000_000_000));
}
