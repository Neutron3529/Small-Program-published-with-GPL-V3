#![no_std]
#![no_main]
#![feature(abi_ptx, stdarch_nvptx, asm_experimental_arch)]

mod consts;
use core::arch::asm;
use core::arch::nvptx::*;

#[panic_handler]
fn ph(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
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

#[unsafe(no_mangle)]
pub unsafe extern "ptx-kernel" fn mont_pows_batch4(out: *mut u64) {
    use consts::{RANGE, VALS};
    let mut cur = VALS.len();
    let mut cntr = 0;

    let start =
    unsafe { ((_block_idx_x() * _block_dim_x() + _thread_idx_x()) as u64) * RANGE as u64 };
    let aux = u64::MAX / (start + (RANGE - 1) as u64); // aux * start - MAX
    while cur > 0 {
        cur -= 1;
        cntr += mont_pows4([start + VALS[cur] as u64, start + VALS[cur+1] as u64, start + VALS[cur+2] as u64, start + VALS[cur+3] as u64], aux);
    }
    unsafe { *out.add((_block_idx_x() * _block_dim_x() + _thread_idx_x()) as usize) = cntr as u64 };
}
#[inline(always)]
fn mont_pows(base: u64, aux: u64) -> u32 {
    let r = (aux * base).wrapping_neg();
    let rsq = r * 3; // currently 3r, will be rsq in loop
    let ibase = neg_mod_2pow64_inv(base);
    let [mut r0, mut r1] = unpack(r);
    let [mut rsq0, mut rsq1] = unpack(rsq);
    let [base0, base1] = unpack(base);
    let [ibase0, ibase1] = unpack(ibase);
    [r0, r1] = mont_mul(r0, r1, rsq0, rsq1, base0, base1, ibase0, ibase1);
    let mut cntr = 0;
    while cntr < 27 {
        cntr += 1;
        [rsq0, rsq1] = mont_mul(r0, r1, r0, r1, base0, base1, ibase0, ibase1);
        [r0, r1] = mont_mul(r0, r1, rsq0, rsq1, base0, base1, ibase0, ibase1);
    }
    [r0, r1] = mont_mul(r0, r1, 1, 0, base0, base1, ibase0, ibase1);
    (pack([r0, r1]) + REM == base) as u32
}

#[inline(always)]
fn mont_pows4(base: [u64; 4], aux: u64) -> u32 {
    let r = [(aux * base[0]).wrapping_neg(), (aux * base[1]).wrapping_neg(), (aux * base[2]).wrapping_neg(), (aux * base[3]).wrapping_neg()];
    let rsq = [3*r[0],3*r[1],3*r[2],3*r[3]]; // currently 3r, will be rsq in loop
    let ibase = neg_mod_2pow64_inv4(base);
    let [mut r0, mut r1] = unpack4(r);
    let [mut rsq0, mut rsq1] = unpack4(rsq);
    let [base0, base1] = unpack4(base);
    let [ibase0, ibase1] = unpack4(ibase);
    [r0, r1] = mont_mul4(r0, r1, rsq0, rsq1, base0, base1, ibase0, ibase1);
    let mut cntr = 0;
    while cntr < 27 {
        cntr += 1;
        [rsq0, rsq1] = mont_mul4(r0, r1, r0, r1, base0, base1, ibase0, ibase1);
        [r0, r1] = mont_mul4(r0, r1, rsq0, rsq1, base0, base1, ibase0, ibase1);
    }
    [r0, r1] = mont_mul4(r0, r1, 1, 0, base0, base1, ibase0, ibase1);
    let res = pack4([r0, r1]);
    (res[0] + REM == base[0]) as u32 + (res[1] + REM == base[1]) as u32 + (res[2] + REM == base[2]) as u32 + (res[3] + REM == base[3]) as u32
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
fn unpack4(from: [u64;4]) -> [[u32;4]; 2] {
    // core::mem::transmute(from)
    let r00: u32;
    let r01: u32;
    let r02: u32;
    let r03: u32;
    let r10: u32;
    let r11: u32;
    let r12: u32;
    let r13: u32;
    unsafe {
        asm!("mov.b64 {{{r0},{r1}}},{from};", r0 = out(reg32) r00, r1 = out(reg32) r10, from = in(reg64) from[0])
        asm!("mov.b64 {{{r0},{r1}}},{from};", r0 = out(reg32) r01, r1 = out(reg32) r11, from = in(reg64) from[1])
        asm!("mov.b64 {{{r0},{r1}}},{from};", r0 = out(reg32) r02, r1 = out(reg32) r12, from = in(reg64) from[2])
        asm!("mov.b64 {{{r0},{r1}}},{from};", r0 = out(reg32) r03, r1 = out(reg32) r13, from = in(reg64) from[3])
    }
    [[r00, r01, r02, r03], [r10, r11, r12, r13]]
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
#[inline(always)]
fn pack4([small, large]: [[u32 ;4]; 2]) -> u64 {
    // core::mem::transmute(from)
    let reg0;
    let reg1;
    let reg2;
    let reg3;
    unsafe {
        asm!("mov.b64 {reg},{{{small},{large}}};", reg = out (reg64) reg0, small=in(reg32) small[0], large = in(reg32) large[0])
        asm!("mov.b64 {reg},{{{small},{large}}};", reg = out (reg64) reg1, small=in(reg32) small[1], large = in(reg32) large[1])
        asm!("mov.b64 {reg},{{{small},{large}}};", reg = out (reg64) reg2, small=in(reg32) small[2], large = in(reg32) large[2])
        asm!("mov.b64 {reg},{{{small},{large}}};", reg = out (reg64) reg3, small=in(reg32) small[3], large = in(reg32) large[3])
    }
    [reg0, reg1, reg2, reg3]
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

fn neg_mod_2pow64_inv4(p: [u64; 4]) -> [u64; 4] {
    let mut neg_inv = [1u64; 4];
    neg_inv[0] = neg_inv.wrapping_mul(2u64.wrapping_add(p[0].wrapping_mul(neg_inv[0]))); // 2bit
    neg_inv[1] = neg_inv.wrapping_mul(2u64.wrapping_add(p[1].wrapping_mul(neg_inv[1])));
    neg_inv[2] = neg_inv.wrapping_mul(2u64.wrapping_add(p[2].wrapping_mul(neg_inv[2])));
    neg_inv[3] = neg_inv.wrapping_mul(2u64.wrapping_add(p[3].wrapping_mul(neg_inv[3])));
    neg_inv[0] = neg_inv.wrapping_mul(2u64.wrapping_add(p[0].wrapping_mul(neg_inv[0]))); // 4bit
    neg_inv[1] = neg_inv.wrapping_mul(2u64.wrapping_add(p[1].wrapping_mul(neg_inv[1])));
    neg_inv[2] = neg_inv.wrapping_mul(2u64.wrapping_add(p[2].wrapping_mul(neg_inv[2])));
    neg_inv[3] = neg_inv.wrapping_mul(2u64.wrapping_add(p[3].wrapping_mul(neg_inv[3])));
    neg_inv[0] = neg_inv.wrapping_mul(2u64.wrapping_add(p[0].wrapping_mul(neg_inv[0]))); // 8bit
    neg_inv[1] = neg_inv.wrapping_mul(2u64.wrapping_add(p[1].wrapping_mul(neg_inv[1])));
    neg_inv[2] = neg_inv.wrapping_mul(2u64.wrapping_add(p[2].wrapping_mul(neg_inv[2])));
    neg_inv[3] = neg_inv.wrapping_mul(2u64.wrapping_add(p[3].wrapping_mul(neg_inv[3])));
    neg_inv[0] = neg_inv.wrapping_mul(2u64.wrapping_add(p[0].wrapping_mul(neg_inv[0]))); // 16bit
    neg_inv[1] = neg_inv.wrapping_mul(2u64.wrapping_add(p[1].wrapping_mul(neg_inv[1])));
    neg_inv[2] = neg_inv.wrapping_mul(2u64.wrapping_add(p[2].wrapping_mul(neg_inv[2])));
    neg_inv[3] = neg_inv.wrapping_mul(2u64.wrapping_add(p[3].wrapping_mul(neg_inv[3])));
    neg_inv[0] = neg_inv.wrapping_mul(2u64.wrapping_add(p[0].wrapping_mul(neg_inv[0]))); // 32bit
    neg_inv[1] = neg_inv.wrapping_mul(2u64.wrapping_add(p[1].wrapping_mul(neg_inv[1])));
    neg_inv[2] = neg_inv.wrapping_mul(2u64.wrapping_add(p[2].wrapping_mul(neg_inv[2])));
    neg_inv[3] = neg_inv.wrapping_mul(2u64.wrapping_add(p[3].wrapping_mul(neg_inv[3])));
    neg_inv[0] = neg_inv.wrapping_mul(2u64.wrapping_add(p[0].wrapping_mul(neg_inv[0]))); // 64bit
    neg_inv[1] = neg_inv.wrapping_mul(2u64.wrapping_add(p[1].wrapping_mul(neg_inv[1])));
    neg_inv[2] = neg_inv.wrapping_mul(2u64.wrapping_add(p[2].wrapping_mul(neg_inv[2])));
    neg_inv[3] = neg_inv.wrapping_mul(2u64.wrapping_add(p[3].wrapping_mul(neg_inv[3])));
    neg_inv // fine.
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
fn mont_mul4(a0: [u32; 4], a1: [u32; 4], b0: [u32; 4], b1: [u32; 4], m0: [u32; 4], m1: [u32; 4], n0: [u32; 4], n1: [u32; 4]) -> [[u32; 4]; 2] {
    // let t = a as u128 * b as u128;
    let [t0, t1, t2, t3] = mul64_wide4(a0, a1, b0, b1);
    // let k = (t as u64).wrapping_mul(n);
    let [k0, k1] = mul64_lo4(t0, t1, n0, n1);
    // let res = ((t + k as u128 * m as u128) >> 64) as u64;
    let [r0, r1] = mul64_hi4(k0, k1, m0, m1); // k * m >> 64
    // let [r0, r1] = unpack(mul64_hi(pack([k0, k1]), pack([m0, m1])));
    // if res >= m { res - m } else { res }
    addsub4(r0, r1, t2, t3, m0, m1) // (km>>64 + t>>64 + 1) - m if possible
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
fn addsub4(a0: [u32; 4], a1: [u32; 4], b0: [u32; 4], b1: [u32; 4], m0: [u32; 4], m1: [u32; 4]) -> [[u32; 4]; 2] {
    let r0 = addsub(a0[0], a1[0], b0[0], b1[0], m0[0], m1[0]);
    let r1 = addsub(a1[1], a1[1], b1[1], b1[1], m1[1], m1[1]);
    let r2 = addsub(a2[2], a1[2], b2[2], b1[2], m2[2], m1[2]);
    let r3 = addsub(a3[3], a1[3], b3[3], b1[3], m3[3], m1[3]);
    [[r0[0], r1[0], r2[0],r3[0]],[r0[1], r1[1], r2[1],r3[1]]]
}

#[inline(always)]
fn mul64_lo(x0: u32, x1: u32, y0: u32, y1: u32) -> [u32; 2] {
    unpack(pack([x0, x1]) * pack([y0, y1]))
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
