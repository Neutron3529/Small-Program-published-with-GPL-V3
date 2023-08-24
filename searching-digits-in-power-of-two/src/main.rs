#![feature(
    portable_simd,
    array_chunks,
    const_mut_refs,
    array_windows,
    int_roundings
)]
use core::ops::Shl;
use core::simd::{Simd, SimdPartialEq, SimdPartialOrd};
// use rayon::iter::{ParallelBridge,ParallelIterator};
type Base = u8;
const BASE: Base = 10;
const FLAG: Base = ((BASE).next_power_of_two() + 1).next_power_of_two();
const FLAG_MASK: Base = FLAG - 1;
const FLAG_SHR: u32 = FLAG.ilog2();
const BASE_ADD: Base = FLAG - BASE;
const LANES: usize = 64;

type BaseCalc = u32;
const DIGITS: u32 = 6;
const TARGET: BaseCalc = (BASE as BaseCalc).pow(DIGITS);
const TARGET_MAXOUT: [BaseCalc; BASE as usize] = maxout_gen();

const fn maxout_gen() -> [BaseCalc; BASE as usize] {
    let mut ret = [0; BASE as usize];
    let mut x = 0;
    while x < BASE as usize {
        ret[x] = (x as BaseCalc) * (BASE as BaseCalc).pow(DIGITS - 1);
        x += 1
    }
    ret
}

#[derive(Debug)]
struct MSlice([u8; TARGET as usize]);
impl MSlice {
    const FILLED: Simd<u8, LANES> = Simd::from_array([1; LANES]);
    pub const fn new() -> Self {
        Self([0; TARGET as usize])
    }
    pub fn reset(&mut self) {
        self.0.fill(0)
    }
    pub const fn set(&mut self, idx: BaseCalc) {
        self.0[idx as usize] = 1
    }
    pub fn filled(&self) -> bool {
        let (a, b, c) = self.0.as_simd::<LANES>();
        a.iter().chain(c.iter()).all(|&x| x != 0) && b.iter().all(|x| x.simd_eq(Self::FILLED).all())
    }
    pub fn check_and_reset(&mut self) -> bool {
        let x = self.filled();
        self.reset();
        x
    }
}

fn mul2(mut num: Vec<Base>) -> Vec<Base> {
    {
        let compare = Simd::<Base, LANES>::splat(BASE);
        let add = Simd::<Base, LANES>::splat(BASE_ADD);
        let (first, middle, last) = num.as_simd_mut::<LANES>();
        fn m2set1(x: &mut Base) {
            *x <<= 1;
            if *x >= BASE {
                *x += BASE_ADD
            }
        }
        first.iter_mut().for_each(m2set1);
        last.iter_mut().for_each(m2set1);
        middle.iter_mut().for_each(|x| {
            *x += *x;
            let z = *x + add;
            *x = x.simd_ge(compare).select(z, *x);
        })
    }
    {
        let mut s = 0;
        num.iter_mut()
            .for_each(|x| (*x, s) = ((*x + s) & FLAG_MASK, *x >> FLAG_SHR));
        if s > 0 {
            num.push(1)
        }
    }
    num
}

static STOP: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(usize::MAX);

fn inner_block(block_size: u32, cur_block_idx: u32, mut num: Vec<Base>, mask: &mut MSlice) {
    for curidx in cur_block_idx * block_size + 1..cur_block_idx * block_size + block_size + 1 {
        if STOP.load(std::sync::atomic::Ordering::Relaxed) < curidx as usize {
            println!("{cur_block_idx} triggered early stop after {curidx}");
            break;
        }
        num = mul2(num);
        if num.len() > TARGET as usize {
            num.array_windows::<{ DIGITS as usize }>().fold(
                num.iter()
                    .take(DIGITS as usize - 1)
                    .fold(0, |s, &x| s * (BASE as BaseCalc) + (x as BaseCalc)),
                |mut s, x| {
                    s = s * (BASE as BaseCalc) + (x[(DIGITS - 1) as usize] as BaseCalc);
                    mask.set(s);
                    s - TARGET_MAXOUT[x[0] as usize]
                },
            );
            if mask.check_and_reset() {
                println!("{cur_block_idx} found {curidx}");
                STOP.fetch_min(curidx as usize, std::sync::atomic::Ordering::SeqCst);
                break;
            }
        }
    }
}
#[test]
fn test() {
    let block_size = 100;
    let total = std::time::Instant::now();
    rayon::scope_fifo(|s| {
        for j in 100000..100010 {
            s.spawn_fifo(move |_| {
                let mut mask = MSlice::new();
                let now = std::time::Instant::now();
                let num: Vec<_> = rug::Integer::from(rug::Integer::from(1).shl(j * block_size))
                .to_string_radix(BASE as i32)
                .bytes()
                .rev()
                .map(|x| x - b'0')
                .collect();
                inner_block(block_size, j, num, &mut mask);
                println!("{j:4} {:?}", now.elapsed());
            })
        }
    });
    println!("total: {:?}", total.elapsed())
}
fn main() {
    let block_size = 10000;
    let total = std::time::Instant::now();
    rayon::scope_fifo(|s| {
        for j in 0..4000 {
            s.spawn_fifo(move |_| {
                let mut mask = MSlice::new();
                let now = std::time::Instant::now();
                let num: Vec<_> = rug::Integer::from(rug::Integer::from(1).shl(j * block_size))
                .to_string_radix(BASE as i32)
                .bytes()
                .rev()
                .map(|x| x - b'0')
                .collect();
                inner_block(block_size, j, num, &mut mask);
                println!("{j:4} {:?}", now.elapsed());
            })
        }
    });
    println!("total: {:?}", total.elapsed())
}
