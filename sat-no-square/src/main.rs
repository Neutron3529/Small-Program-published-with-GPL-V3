use std::env;
#[derive(Clone, Copy)]
struct Slot {
    slot: u32,
    bits: u32,
}
fn decode_number(i: u32, idx: u32, slot: Slot) {
    for bits in (0..slot.bits).rev() {
        if idx & 1 << bits != 0 {
            print!("-{} ", i * slot.bits - bits)
        } else {
            print!("{} ", i * slot.bits - bits)
        }
    }
}
fn decode_node(i: u32, slot: Slot) {
    println!("c calculating {i}");
    for idx in slot.slot..slot.slot.next_power_of_two() {
        decode_number(i, idx, slot);
        println!("0")
    }
}
fn decode(i: u32, j: u32, slot: Slot) {
    println!("c calculating {i} {j}");
    for idx in 0..slot.slot {
        decode_number(i, idx, slot);
        decode_number(j, idx, slot);
        println!("0")
    }
}
fn main() {
    let nodes = env::args()
        .nth(1)
        .map(|x| x.parse().ok())
        .flatten()
        .unwrap_or(475u32);
    let slots = env::args()
        .nth(2)
        .map(|x| x.parse().ok())
        .flatten()
        .unwrap_or(4u32);
    let slot = Slot {
        slot: slots,
        bits: slots.next_power_of_two().ilog2() as _,
    };
    println!("c calculating with node {nodes} and slot {slots}");
    let mut rule = Vec::new();
    for i in 1..=(nodes * 2u32).isqrt() {
        let i = i * i;
        for j in 1.max(i as i32 - nodes as i32) as u32..(i + 1) / 2 {
            if j <= nodes && i - j <= nodes {
                rule.push((j, i - j));
            } else {
                panic!("got {i} {j}")
            }
        }
    }
    println!(
        "p cnf {} {}",
        nodes * slot.bits,
        nodes * (slot.slot.next_power_of_two() - slot.slot) + slot.slot * rule.len() as u32
    );
    for i in 1..=nodes {
        decode_node(i, slot);
    }
    for (i, j) in rule {
        decode(i, j, slot)
    }
}
