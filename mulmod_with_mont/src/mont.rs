use crate::MulMod;
#[derive(Copy, Clone)]
pub struct Mul64 {
    pub base: u64,
    pub(crate) baseinv: u64,
}
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Mont64(u64);
impl Mul64 {
    pub fn new(base: u64) -> Self {
        Self {
            base,
            baseinv: neg_mod_2pow64_inv(base),
        }
    }
    pub fn redc(self, a: Mont64, b: Mont64) -> Mont64 {
        Mont64(redc_64(a.0, b.0, self.base, self.baseinv))
    }
    pub fn num_to_mont(self, a: u64) -> Mont64 {
        Mont64(num_to_mont(a, self.base, self.baseinv))
    }
    pub fn mont_to_num(self, a: Mont64) -> u64 {
        mont_to_num(a.0, self.base, self.baseinv)
    }
}
impl MulMod for Mul64 {
    type Item = Mont64;
    fn mul(self, a: Self::Item, b: Self::Item) -> Self::Item {
        self.redc(a, b)
    }
}
/// m: mod, n: mn=-1 mod R, R:2^64
pub(crate) fn redc_64(a: u64, b: u64, m: u64, n: u64) -> u64 {
    let t = a as u128 * b as u128;
    let k = (t as u64).wrapping_mul(n);
    let res = ((t + k as u128 * m as u128) >> 64) as u64;
    if res >= m { res - m } else { res }
}
pub(crate) fn num_to_mont(a: u64, m: u64, n: u64) -> u64 {
    redc_64(a, (u128::MAX % m as u128) as u64 + 1, m, n)
}
pub(crate) fn mont_to_num(a: u64, m: u64, n: u64) -> u64 {
    redc_64(a, 1, m, n)
}

// p must be odd.
pub(crate) fn neg_mod_2pow64_inv(p: u64) -> u64 {
    // 牛顿迭代法
    // 原理:
    // neg_inv *= 1 + epsilon, epsilon = 1 + p*neg_inv
    // 如果我有m bit精度，或者说neg_inv * p = c * 2^m - 1，那么下次迭代时
    // new_new_inv = neg_inv + (1 + p*neg_inv) * neg_inv
    // 由此 new_neg_inv * p = p*neg_inv + (1 + p*neg_inv) * neg_inv * p = c * 2^m - 1 + (c * 2^m) (c * 2^m - 1) = -1 + c*2^m
    // 从而，每次迭代精度翻倍
    let mut neg_inv = 1u64;
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 2bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 4bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 8bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 16bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 32bit
    neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))) // fine.
}

// pub (crate) fn neg_mod_2pow64_inv(a: u64) -> u64 {
//     // b mod a
//     fn extended_euclidean_algorithm(b: u64) -> (u64, u64, u64) {
//         #[inline(always)]
//         fn update_step(a: &mut u64, old_a: &mut u64, quotient: u64) {
//             (*a, *old_a) = ((*old_a).wrapping_sub(quotient.wrapping_mul(*a)), *a);
//         }
//         let (mut old_r, mut rem) = (u64::MAX, b);
//         let (mut old_s, mut coeff_s) = (1, 0);
//         let (mut old_t, mut coeff_t) = (0, 1);
//
//         let quotient = old_r / rem;
//         update_step(&mut rem, &mut old_r, quotient);
//         update_step(&mut coeff_s, &mut old_s, quotient);
//         update_step(&mut coeff_t, &mut old_t, quotient);
//         rem += 1; // 补上u64::MAX中省略的1
//
//         while rem != 0 {
//             let quotient = old_r / rem;
//             update_step(&mut rem, &mut old_r, quotient);
//             update_step(&mut coeff_s, &mut old_s, quotient);
//             update_step(&mut coeff_t, &mut old_t, quotient);
//         }
//         (old_r, old_s, old_t)
//     }
//     (-(extended_euclidean_algorithm(a).2 as i64)) as u64
// }
