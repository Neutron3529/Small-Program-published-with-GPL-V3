use mulmod_with_mont::*;

use rayon::prelude::*;

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
    let small_primes = euler_sieve(sqrt_limit);

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

        sieve
            .into_iter()
            .enumerate()
            .filter(|&(_, is_prime)| is_prime)
            .for_each(|(i, _)| {
                let mont = mont::Mul64::new(positions[i]);
                if mont.mont_to_num(pow(mont.num_to_mont(3), mont)) == mont.base - 4 {
                    println!("Found {}", mont.base)
                }
            })
    })
}

fn main() {
    let now = std::time::Instant::now();
    // check_with_parallel_sieve(std::env::args().nth(1).as_deref().unwrap_or("").parse().unwrap_or(10_000_000_000));
    check_with_parallel_sieve(std::env::args().nth(1).map_or(None,|x|x.parse().ok()).unwrap_or(10_000_000_000));
    println!("cost {:?}", now.elapsed())
}
