#![feature(array_windows)]
fn euler_sieve(n: usize) -> Vec<usize> {
    let mut is_prime = vec![true; n + 1];
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
fn main() {
    let now = std::time::Instant::now();
    let res = euler_sieve(100_0000_0000);
    println!("cost {:?}", now.elapsed());
    let now = std::time::Instant::now();
    let res = res
        .array_windows()
        .filter_map(|&[x, y]| if x + 2 == y { Some(x + 1) } else { None })
        .collect::<Vec<_>>();
    println!("cost {:?}", now.elapsed());
    println!("len: {}", res.len());
    let hs = std::collections::HashSet::<usize>::from_iter(res.iter().copied());
    for i in 0..res.len() {
        let mut find = false;
        for j in 0..i {
            if hs.contains(&(res[i] - res[j])) {
                find = true;
                break;
            }
        }
        if !find {
            println!("index = {i}, {} cannot be decomposed", res[i])
        }
    }
}
