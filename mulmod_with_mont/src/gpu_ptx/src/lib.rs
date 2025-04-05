#![no_std]
#![no_main]
#![feature(abi_ptx, stdarch_nvptx, asm_experimental_arch)]

mod consts;
use core::arch::asm;
use core::arch::nvptx::*;
use cuda_min::repeat;

const REM: u64 = 2;

#[unsafe(no_mangle)]
pub unsafe extern "ptx-kernel" fn mont_pows_batch(out: *mut u64) {
    use consts::{RANGE, VALS};
    let mut cur = VALS.len();
    let mut cntr = 0;

    let start =
        unsafe { ((_block_idx_x() * _block_dim_x() + _thread_idx_x()) as u64) * RANGE as u64 };
    let aux = u64::MAX / (start + (RANGE - 1) as u64); // aux * start - MAX
    while cur > 0 {
        cur -= 1;
        cntr += mont_pows(start + VALS[cur] as u64, aux);
    }
    unsafe { *out.add((_block_idx_x() * _block_dim_x() + _thread_idx_x()) as usize) = cntr as u64 };
}

#[inline(always)]
fn mont_pows(base: u64, aux: u64) -> u32 {
    let r = (aux * base).wrapping_neg();
    // let r = (u64::MAX % base) + 1;
    let rsq = r * 3; // currently 3r, will be rsq in loop
    let ibase = neg_mod_2pow64_inv(base);
    let [mut r0, mut r1] = unpack(r);
    let [mut rsq0, mut rsq1] = unpack(rsq);
    let [base0, base1] = unpack(base);
    let [ibase0, ibase1] = unpack(ibase);
    [r0, r1] = mont_mul(r0, r1, rsq0, rsq1, base0, base1, ibase0, ibase1);
    repeat! {
        { @ @ @  @ @ @  @ @ @   @ @ @  @ @ @  @ @ @   @ @ @  @ @ @  @ @ @ }
        {
            [rsq0, rsq1] = mont_mul(r0, r1, r0, r1, base0, base1, ibase0, ibase1);
            [r0, r1] = mont_mul(r0, r1, rsq0, rsq1, base0, base1, ibase0, ibase1);
        }
    }
    [r0, r1] = mont_mul(r0, r1, 1, 0, base0, base1, ibase0, ibase1);
    (pack([r0, r1]) + REM == base) as u32
}

#[inline(always)]
fn unpack(from: u64) -> [u32; 2] {
    // core::mem::transmute(from)
    let r0: u32;
    let r1: u32;
    unsafe {
        asm!("mov.b64 {{{r0},{r1}}},{from};", r0 = out(reg32) r0, r1 = out(reg32) r1, from = in(reg64) from)
    }
    [r0, r1]
}
#[inline(always)]
fn pack([small, large]: [u32; 2]) -> u64 {
    // core::mem::transmute(from)
    let reg;
    unsafe {
        asm!("mov.b64 {reg},{{{small},{large}}};", reg = out (reg64) reg, small=in(reg32) small, large = in(reg32) large)
    }
    reg
}
fn neg_mod_2pow64_inv(p: u64) -> u64 {
    let mut neg_inv = 1u64;
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 2bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 4bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 8bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 16bit
    neg_inv = neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))); // 32bit
    neg_inv.wrapping_mul(2u64.wrapping_add(p.wrapping_mul(neg_inv))) // fine.
}

#[inline(always)]
fn mont_mul(a0: u32, a1: u32, b0: u32, b1: u32, m0: u32, m1: u32, n0: u32, n1: u32) -> [u32; 2] {
    // let t = a as u128 * b as u128;
    let [t0, t1, t2, t3] = mul64_wide(a0, a1, b0, b1);
    // let k = (t as u64).wrapping_mul(n);
    let [k0, k1] = mul64_lo(t0, t1, n0, n1);
    // let res = ((t + k as u128 * m as u128) >> 64) as u64;
    let [r0, r1] = mul64_hi(k0, k1, m0, m1); // k * m >> 64
    // let [r0, r1] = unpack(mul64_hi(pack([k0, k1]), pack([m0, m1])));
    // if res >= m { res - m } else { res }
    addsub(r0, r1, t2, t3, m0, m1) // (km>>64 + t>>64 + 1) - m if possible
}

#[inline(always)]
fn addsub(a0: u32, a1: u32, b0: u32, b1: u32, m0: u32, m1: u32) -> [u32; 2] {
    unsafe {
        let r0: u32;
        let r1: u32;
        asm!(r#"add.cc.s32 {garbage},-1,-1; // `add.cc.s32` has the same meaning as `add.cc.u32`, thus -1 + -1 = -2 will set carry bit.
	addc.cc.u32 {0},{0},{2};
	addc.u32 {1},{1},{3};
	sub.cc.u32 {2},{0},{4};
	subc.u32 {3},{1},{5};
	{{
		.reg .pred p;
		setp.le.u32 p,{1},2147483648;
		selp.u32 {0},{0},{2},p;
		selp.u32 {1},{1},{3},p;
    }}"#, inout(reg32) a0 => r0, inout(reg32) a1 => r1, inout(reg32) b0 => _, inout(reg32) b1 => _, in(reg32) m0, in(reg32) m1, garbage = lateout(reg32) _);
        [r0, r1]
    }
}

#[inline(always)]
fn mul64_lo(x0: u32, x1: u32, y0: u32, y1: u32) -> [u32; 2] {
    // unpack(pack([x0, x1]) * pack([y0, y1]))
    unsafe {
        let r0: u32;
        let r1: u32;
        asm!(r#"mul.lo.u32     {r0},{x0},{y0};      // r0=(r4*r6).[31:0], no carry-out
	mul.hi.u32     {r1},{x0},{y0};      // r1=(r4*r6).[63:32], no carry-out
	mad.lo.u32     {r1},{x0},{y1},{r1}; // r1=(r4*r6).[63:32], no carry-out
	mad.lo.u32     {r1},{x1},{y0},{r1}; // r1+=(r5*r6).[31:0], may carry-out"#,
            x0 = in(reg32) x0, x1 = in(reg32) x1, y0 = in(reg32) y0, y1 = in(reg32) y1,
            r0 = out(reg32) r0, r1 = out(reg32) r1
        );
        [r0, r1]
    }
}

#[inline(always)]
fn mul64_wide(x0: u32, x1: u32, y0: u32, y1: u32) -> [u32; 4] {
    unsafe {
        let r0: u32;
        let r1: u32;
        let r2: u32;
        let r3: u32;
        asm!(r#"mul.lo.u32     {0},{4},{6};      // r0=(r4*r6).[31:0], no carry-out
	mul.hi.u32     {1},{4},{6};      // r1=(r4*r6).[63:32], no carry-out
	mad.lo.cc.u32  {1},{5},{6},{1};   // r1+=(r5*r6).[31:0], may carry-out
	madc.hi.u32    {2},{5},{6},0;    // r2 =(r5*r6).[63:32]+carry-in,
	// no carry-out
	mad.lo.cc.u32   {1},{4},{7},{1};  // r1+=(r4*r7).[31:0], may carry-out
	madc.hi.cc.u32  {2},{4},{7},{2};  // r2+=(r4*r7).[63:32]+carry-in,
	// may carry-out
	addc.u32        {3},0,0;       // r3 = carry-in, no carry-out
	mad.lo.cc.u32   {2},{5},{7},{2};  // r2+=(r5*r7).[31:0], may carry-out
	madc.hi.u32     {3},{5},{7},{3};  // r3+=(r5*r7).[63:32]+carry-in"#,
             out(reg32) r0, out(reg32) r1, out(reg32) r2, out(reg32) r3,
             in(reg32) x0, in(reg32) x1, in(reg32) y0, in(reg32) y1,
        );
        [r0, r1, r2, r3]
    }
}

#[inline(always)]
fn mul64_hi(x0: u32, x1: u32, y0: u32, y1: u32) -> [u32; 2] {
    unsafe {
        let r2: u32;
        let r3: u32;
        asm!(r#"mul.hi.u32     {r1},{x0},{y0};      // r1=(r4*r6).[63:32], no carry-out
	mad.lo.cc.u32  {r1},{x1},{y0},{r1};   // r1+=(r5*r6).[31:0], may carry-out
	madc.hi.u32    {r2},{x1},{y0},0;    // r2 =(r5*r6).[63:32]+carry-in,
	// no carry-out
	mad.lo.cc.u32   {r1},{x0},{y1},{r1};  // r1+=(r4*r7).[31:0], may carry-out
	madc.hi.cc.u32  {r2},{x0},{y1},{r2};  // r2+=(r4*r7).[63:32]+carry-in,
	// may carry-out
	addc.u32        {r3},0,0;       // r3 = carry-in, no carry-out
	mad.lo.cc.u32   {r2},{x1},{y1},{r2};  // r2+=(r5*r7).[31:0], may carry-out
	madc.hi.u32     {r3},{x1},{y1},{r3};  // r3+=(r5*r7).[63:32]+carry-in"#,
             r1 = out(reg32) _, r2 = out(reg32) r2, r3 = out(reg32) r3,
             x0 = in(reg32) x0, x1 = in(reg32) x1, y0 = in(reg32) y0, y1 = in(reg32) y1,
        );
        [r2, r3]
    }
}
