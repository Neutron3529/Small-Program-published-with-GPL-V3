fn euler_sieve(n: usize) -> Vec<usize> {
    let mut is_prime = vec![true; n + 1]; // 没必要扣这里的字节数，因为这部份理应很快算完
    let mut primes = Vec::new();
    for i in 2..=n {
        if is_prime[i] {
            primes.push(i);
        }
        for &p in &primes {
            let m = i * p;
            if m > n {
                break;
            }
            is_prime[m] = false;
            if i % p == 0 {
                break;
            }
        }
    }
    primes
}

fn parallel_sieve(limit: usize) -> Vec<usize> {
    let sqrt_limit = limit.isqrt() + 1;
    let small_primes = euler_sieve(sqrt_limit);

    let block_size = sqrt_limit;
    let start = sqrt_limit + 1;
    let end = limit;

    if start > end {
        return small_primes;
    }

    let blocks: Vec<(usize, usize)> = (start..=end)
        .step_by(block_size)
        .map(|low| {
            let high = std::cmp::min(low + block_size - 1, end);
            (low, high)
        })
        .collect();

    let large_primes: Vec<usize> = blocks
        .par_iter()
        .flat_map(|&(low, high)| {
            let segment_size = high - low + 1;
            let mut sieve = vec![true; segment_size];
            for &p in &small_primes {
                let rem = low % p;
                let first_multiple = if rem == 0 { low } else { low + (p - rem) };
                let start = std::cmp::max(first_multiple, p * p);
                if start > high {
                    continue;
                }
                for multiple in (start..=high).step_by(p) {
                    sieve[multiple - low] = false;
                }
            }
            sieve
                .iter()
                .enumerate()
                .filter(|&(_, &is_prime)| is_prime)
                .map(|(i, _)| low + i)
                .collect::<Vec<_>>()
        })
        .collect();

    [small_primes, large_primes].concat()
}
