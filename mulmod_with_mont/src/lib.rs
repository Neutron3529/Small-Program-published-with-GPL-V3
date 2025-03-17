#![cfg_attr(test, feature(test))]

pub mod mont;
pub mod pow;
pub use pow::{MulMod, pow};

#[cfg(test)]
mod tests {
    extern crate test;
    use crate::*;
    use std::hint::black_box;
    use test::bench::Bencher;
    #[test]
    fn test() {
        use crate::mont::{self, *};

        for (&m, &n) in PRIMES.iter().zip(PINV.iter()) {
            let mut base = 3u64;
            assert_eq!(neg_mod_2pow64_inv(m), n);
            for _ in 0..50 {
                let mont = num_to_mont(base, m, n);
                base = ((base as u128) * (base as u128) % (m as u128)) as u64;
                assert_eq!(
                    base,
                    mont_to_num(redc_64(mont, mont, m, n), m, n),
                    "m={m},n={n}, mont={mont}"
                )
            }
            let mont = mont::Mul64::new(m);
            assert_eq!(pow(3, m), mont.mont_to_num(pow(mont.num_to_mont(3), mont)));
        }
        use crate::pow::{pow13, pow14};
        assert_eq!(pow13(1, ()), 1594323);
        assert_eq!(pow14(1, ()), 4782969);
        assert_eq!(pow(1, ()), 7625597484987);
    }

    const PRIMES: [u64; 32] = [
        559813183481,
        889050711719,
        982099809157,
        1336624675327,
        1503476849129,
        2865622031843,
        3431666782757,
        3682908178973,
        4306587719177,
        4403762012807,
        4613278001903,
        4711831880999,
        4839503249101,
        5072674337501,
        5175788899357,
        5577301470023,
        5688864717469,
        5934678050123,
        6044385997129,
        6074864632379,
        6088352567161,
        6144840173293,
        6302828258263,
        6764357782153,
        7040056674533,
        7074288468901,
        7331483134741,
        7431893650219,
        7447866118663,
        9181794149243,
        9413985262271,
        9863051989649,
    ]; // a=vecsort(vector(32,X,randomprime(10^13)))
    const PINV: [u64; 32] = [
        11616029185893283255,
        7463282029657325289,
        5699051013811655347,
        10123472275407501825,
        15882628372672655783,
        3175594818807656501,
        14636777370997474899,
        4187160157964597195,
        7107061883965876167,
        10068775883287329993,
        14273319300571429873,
        2443640735777138025,
        12938174169985131003,
        1692054029670173323,
        9607101577957992907,
        5686380102331953545,
        8018468081018233931,
        10691515829396120477,
        1144889158232125191,
        3800434216232843533,
        338187613405271351,
        6358631403941968155,
        17219778285490290713,
        13855687695977411655,
        13481527796604422931,
        5534768641287672787,
        800666586215658435,
        10040804654035969149,
        3285297405705143369,
        5445878735749889101,
        4196856182749981377,
        14874019168110760335,
    ]; // apply(x->lift(Mod(-x,2^64)^-1),a)

    #[bench]
    fn bench_div(b: &mut Bencher) {
        b.iter(|| {
            let mut sum = 0;
            for i in PRIMES {
                sum += pow(3, i)
            }
            sum
        });
    }

    #[bench]
    fn bench_div_mont(b: &mut Bencher) {
        b.iter(|| {
            let mut sum = 0;
            for i in PRIMES {
                let mont = mont::Mul64::new(i);
                sum += mont.mont_to_num(pow(mont.num_to_mont(3), black_box(mont)))
            }
            sum
        });
    }

    // #[bench]
    // fn bench_mul(b: &mut Bencher) {
    //     use std::simd::Simd;
    //     use core::arch::x86_64::__m512i;
    //     b.iter(|| {
    //         let mut sum = 0;
    //         let base = __m512i::from(Simd::from_array([0u64;8]));
    //         let basep1 = __m512i::from(Simd::from_array([1u64;8]));
    //         let a = __m512i::from(Simd::from_array([123451234512345u128, 543215432112345u128, 674383451938563u128, 873648726941659u128, 658493756381723u128, 859372947382912u128, 857493827164836u128, 815481651439659u128]));
    //         let b = __m512i::from(Simd::from_array([658493756381723u128, 859372947382912u128, 857493827164836u128, 815481651439659u128, 123451234512345u128, 543215432112345u128, 674383451938563u128, 873648726941659u128]));
    //         for _ in 0..50 {
    //             unsafe {
    //                 let z = black_box(_mm512_madd52hi_epu64(base, black_box(a), black_box(b)));
    //                 let x = black_box(_mm512_madd52hi_epu64(basep1, black_box(a), black_box(b)));
    //                 black_box(_mm512_cmpge_epu64_mask(black_box(a), black_box(b)))
    //             }
    //         }
    //         sum
    //     });
    // }
}
