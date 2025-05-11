use crate::MulMod;
use core::arch::x86_64::{
    __m512i, __mmask8, _mm512_cmpge_epu64_mask, _mm512_madd52hi_epu64, _mm512_madd52lo_epu64,
    _mm512_mask_sub_epi64,
};

#[repr(simd, align(64))]
#[derive(Copy, Clone)]
pub struct Base(pub [u64; 8]);
impl Base {
    const ZERO: Base = Base::new_broadcast(0);
    const ONE: Base = Base::new_broadcast(1);
    const TWO: Base = Base::new_broadcast(2);
    pub const THREE: Base = Base::new_broadcast(3);
    pub const fn simd(self) -> __m512i {
        unsafe { core::mem::transmute(self) }
    }
    pub const fn new_broadcast(a: u64) -> Self {
        Self::new([a; 8])
    }
    pub const fn new(arr: [u64; 8]) -> Self {
        Self(arr)
    }
    fn m52_neg_inv(self) -> Self {
        let mut neg_inv = Self::ONE;
        neg_inv = neg_inv.mullo(self.mullop(neg_inv, Self::TWO));
        neg_inv = neg_inv.mullo(self.mullop(neg_inv, Self::TWO));
        neg_inv = neg_inv.mullo(self.mullop(neg_inv, Self::TWO));
        neg_inv = neg_inv.mullo(self.mullop(neg_inv, Self::TWO));
        neg_inv = neg_inv.mullo(self.mullop(neg_inv, Self::TWO));
        neg_inv.mullo(self.mullop(neg_inv, Self::TWO))
    }
    pub(crate) fn num_to_mont(a: [u64; 8], m: Self, n: Self) -> Self {
        let b = Self(core::array::from_fn(|x| {
            ((1u128 << 104) % m.0[x] as u128) as u64
        }));
        Self::redc(Self(a), b, m, n)
    }
    pub(crate) fn mont_to_num(a: Self, m: Self, n: Self) -> [u64; 8] {
        Self::redc(a, Self::ONE, m, n).0
    }
    fn redc(a: Self, b: Self, m: Self, n: Self) -> Self {
        // let t = a as u128 * b as u128;
        let t_52lo = a.mullo(b);
        let t_52hi_p1 = a.mulhip(b, Self::ONE);
        // let k = (t as u64).wrapping_mul(n);
        let k = t_52lo.mullo(n);
        // let res = ((t + k as u128 * m as u128) >> 64) as u64;
        // // equivlent to let res = (t>>64 +1) + (k*m)>>64
        let res = k.mulhip(m, t_52hi_p1);
        // if res >= m { res - m } else { res }
        res.sub_gt(m)
    }

    //    fn mulhi(self, rhs: Self) -> Self {
    //        self.mulhip(rhs, Self::ZERO)
    //    }
    fn mullo(self, rhs: Self) -> Self {
        self.mullop(rhs, Self::ZERO)
    }
    fn mulhip(self, rhs: Self, plus: Self) -> Self {
        unsafe { core::mem::transmute(_mm512_madd52hi_epu64(plus.simd(), self.simd(), rhs.simd())) }
    }
    fn mullop(self, rhs: Self, plus: Self) -> Self {
        unsafe { core::mem::transmute(_mm512_madd52lo_epu64(plus.simd(), self.simd(), rhs.simd())) }
    }
    fn sub_gt(self, rhs: Self) -> Self {
        self.mask_sub(rhs, self.ge(rhs))
    }
    #[inline(always)]
    fn ge(self, rhs: Self) -> __mmask8 {
        unsafe { _mm512_cmpge_epu64_mask(self.simd(), rhs.simd()) }
    }
    #[inline(always)]
    fn mask_sub(self, rhs: Self, mask: __mmask8) -> Self {
        unsafe {
            core::mem::transmute(_mm512_mask_sub_epi64(
                self.simd(),
                mask,
                self.simd(),
                rhs.simd(),
            ))
        }
    }
}
#[derive(Copy, Clone)]
pub struct Mul52 {
    pub base: Base,
    pub(crate) baseinv: Base,
}
impl Mul52 {
    pub fn new(base: <Self as MulMod>::Item) -> Self {
        Self {
            base,
            baseinv: <Self as MulMod>::Item::m52_neg_inv(base),
        }
    }
    pub fn mont_to_num(self, a: <Self as MulMod>::Item) -> [u64; 8] {
        <Self as MulMod>::Item::mont_to_num(a, self.base, self.baseinv)
    }
    pub fn num_to_mont(self, a: [u64; 8]) -> <Self as MulMod>::Item {
        <Self as MulMod>::Item::num_to_mont(a, self.base, self.baseinv)
    }
}
impl MulMod for Mul52 {
    type Item = Base;
    fn mul(self, a: Self::Item, b: Self::Item) -> Self::Item {
        Self::Item::redc(a, b, self.base, self.baseinv)
    }
}
