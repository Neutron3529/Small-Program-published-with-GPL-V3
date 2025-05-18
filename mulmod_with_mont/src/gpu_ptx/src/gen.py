#!/bin/python
def extend(a,b): return ([i for a0 in a[0] for i in range(a0, b*a[1], a[1]) if i % b != 0], a[1]*b)
def extends(a,b): return (sorted(a[0]),len(a[0]),a[1]) if len(b) == 0 else extends(extend(a,b[0]),b[1:])
x = extends(([1],6),[5,7,11,13,17,19,23])
with open("consts.rs",'w') as f:
    f.write(f"pub const RANGE: u32 = {x[2]};\npub const VALS: [u32; {x[1]}] = {x[0]};")
import os
os.system("rustfmt consts.rs")
