#![feature(array_windows)]

pub struct PrimeFlag(Vec<u64>);
impl PrimeFlag {
    pub fn new(i: usize) -> Self {
        Self(vec![u64::MAX;((i+2)/3+63)/64])
    }
    pub fn index(&self, i:usize) -> bool {
        if i<4 {return true}
        let j = i / 3;
        let k = i - (j>>1) * 6;
        if k==1 || k==5 {
            self.0[j>>6] & (1u64<< (j & 63)) != 0
        } else {
            false
        }
    }
    pub fn set(&mut self, i:usize) {
        if i<4 {return}
        let j = i / 3;
        let k = i - (j>>1) * 6;
        if k==1 || k==5 {
            self.0[j>>6] |= 1u64<< (j & 63)
        }
    }
    pub fn unset(&mut self, i:usize) {
        if i<4 {return}
        let j = i / 3;
        let k = i - (j>>1) * 6;
        if k==1 || k==5 {
            self.0[j>>6] &= !(1u64<< (j & 63))
        }
    }
}
pub fn euler_sieve(n: usize) -> Vec<usize> {
    let mut is_prime = PrimeFlag::new(n+1);// vec![true; n + 1];
    let mut primes = Vec::new();
    for i in 2..=n {
        if is_prime.index(i) {
            primes.push(i);
        }
        for &p in &primes {
            let m = i * p;
            if m > n {
                break;
            }
            is_prime.unset(m);
            if i % p == 0 {
                break;
            }
        }
    }
    primes
}
fn main() {
    let now = std::time::Instant::now();
    let res = euler_sieve(1000_0000_0000);
    println!("cost {:?}", now.elapsed());
    let now = std::time::Instant::now();
    let res = res
        .array_windows()
        .filter_map(|&[x, y]| if x + 2 == y { Some(x + 1) } else { None })
        .collect::<Vec<_>>();
    println!("cost {:?}", now.elapsed());
    println!("len: {}", res.len());
    let now = std::time::Instant::now();
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
    println!("cost {:?}", now.elapsed());
}
