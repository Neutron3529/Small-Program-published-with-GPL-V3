pub trait MulMod: Copy {
    type Item: Copy;
    fn mul(self, a: Self::Item, b: Self::Item) -> Self::Item;
}

impl MulMod for u64 {
    type Item = u64;
    fn mul(self, a: Self::Item, b: Self::Item) -> Self::Item {
        ((a as u128 * b as u128) % self as u128) as u64
    }
}
impl MulMod for () {
    // Debug
    type Item = u64;
    fn mul(self, a: Self::Item, b: Self::Item) -> Self::Item {
        a + b
    }
}

pub fn pow<T: Copy>(a0: T, m: impl MulMod<Item = T>) -> T {
    pow13(pow14(a0, m), m)
}
#[inline(always)]
pub(crate) fn pow13<T: Copy>(a0: T, m: impl MulMod<Item = T>) -> T {
    // 0                 1 = a
    // 1( 0)             2
    // 2( 1)             4
    // 3( 2)             8
    // 4( 3, 2)         12
    // 5( 4)            24
    // 6( 5)            48
    // 7( 6)            96
    // 8( 7)           192
    // 9( 8, 0)        193
    // 10( 9, 8)       385
    // 11(10, 3)       393
    // 12(11,10)       778
    // 13(12)         1556
    // 14(13)         3112
    // 15(14)         6224
    // 16(15)        12448
    // 17(16)        24896
    // 18(17)        49792
    // 19(18)        99584
    // 20(19)       199168
    // 21(20)       398336
    // 22(21)       796672
    // 23(22,11)    797065
    // 24(23)      1594130
    // 25(24, 9)   1594323 // 3^13
    // python
    // for cntr, i in enumerate([[j.strip() for j in i] for i in [i.split(')')[0].split(',') for i in a.split('(')][1:]]):print(f'a{cntr+1} = m.mul(a{i[0]}, a{i[-1]});')
    let a1 = m.mul(a0, a0);
    let a2 = m.mul(a1, a1);
    let a3 = m.mul(a2, a2);
    let a4 = m.mul(a3, a2);
    let a5 = m.mul(a4, a4);
    let a6 = m.mul(a5, a5);
    let a7 = m.mul(a6, a6);
    let a8 = m.mul(a7, a7);
    let a9 = m.mul(a8, a0);
    let a10 = m.mul(a9, a8);
    let a11 = m.mul(a10, a3);
    let a12 = m.mul(a11, a10);
    let a13 = m.mul(a12, a12);
    let a14 = m.mul(a13, a13);
    let a15 = m.mul(a14, a14);
    let a16 = m.mul(a15, a15);
    let a17 = m.mul(a16, a16);
    let a18 = m.mul(a17, a17);
    let a19 = m.mul(a18, a18);
    let a20 = m.mul(a19, a19);
    let a21 = m.mul(a20, a20);
    let a22 = m.mul(a21, a21);
    let a23 = m.mul(a22, a11);
    let a24 = m.mul(a23, a23);
    let a25 = m.mul(a24, a9);
    a25
}
#[inline(always)]
pub(crate) fn pow14<T: Copy>(a0: T, m: impl MulMod<Item = T>) -> T {
    // 0                 1 = 1594323
    // 1( 0)             2
    // 2( 1, 0)          3
    // 3( 2)             6
    // 4( 3)            12
    // 5( 4)            24
    // 6( 5, 4)         36
    // 7( 6)            72
    // 8( 7)           144
    // 9( 8)           288
    // 10( 9)          576
    // 11(10, 2)       579
    // 12(11,10)      1155
    // 13(12, 5)      1179
    // 14(13,12)      2334
    // 15(14)         4668
    // 16(15)         9336
    // 17(16)        18672
    // 18(17)        37344
    // 19(18)        74688
    // 20(19)       149376
    // 21(20)       298752
    // 22(21)       597504
    // 23(22)      1195008
    // 24(23)      2390016
    // 25(24,13)   2391195
    // 26(25)      4782390
    // 27(26,11)   4782969 // 3^14
    let a1 = m.mul(a0, a0);
    let a2 = m.mul(a1, a0);
    let a3 = m.mul(a2, a2);
    let a4 = m.mul(a3, a3);
    let a5 = m.mul(a4, a4);
    let a6 = m.mul(a5, a4);
    let a7 = m.mul(a6, a6);
    let a8 = m.mul(a7, a7);
    let a9 = m.mul(a8, a8);
    let a10 = m.mul(a9, a9);
    let a11 = m.mul(a10, a2);
    let a12 = m.mul(a11, a10);
    let a13 = m.mul(a12, a5);
    let a14 = m.mul(a13, a12);
    let a15 = m.mul(a14, a14);
    let a16 = m.mul(a15, a15);
    let a17 = m.mul(a16, a16);
    let a18 = m.mul(a17, a17);
    let a19 = m.mul(a18, a18);
    let a20 = m.mul(a19, a19);
    let a21 = m.mul(a20, a20);
    let a22 = m.mul(a21, a21);
    let a23 = m.mul(a22, a22);
    let a24 = m.mul(a23, a23);
    let a25 = m.mul(a24, a13);
    let a26 = m.mul(a25, a25);
    let a27 = m.mul(a26, a11);
    a27
}
