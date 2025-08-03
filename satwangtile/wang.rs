// 生产cnf格式送cadical（做这个问题cadical比kissat快）
type Color = u8;
fn r#gen(mut v: i32, mut h: i32, blocks: [&[Color]; 4]) {
    let cyc_h = v < 0; // horizontal
    let cyc_v = h < 0; // vertical
    h = h.abs();
    v = v.abs();
    let mut nodes = vec![vec![0; v as usize]; h as usize];
    // 0|1|2|3 v_edges=vec[h][v-1]
    // 4 5 6 7
    // - - - -
    // 8 9 A B h_edges=vec[h-1][v]
    // println!("{}",nodes[1][1])
    let h_rules = (h - if cyc_h { 0 } else { 1 }) as usize;
    let v_rules = (v - if cyc_v { 0 } else { 1 }) as usize;
    let mut v_edges = vec![vec![0; v_rules]; h as usize];
    let mut h_edges = vec![vec![0; v as usize]; h_rules];

    if blocks.len() != 4 {
        panic!("需要提供Wang block的格式：[[上面边的颜色]，[下面边的颜色]，[左边边的颜色]，[右边边的颜色]]）")
    } else {
        let color = blocks
            .iter()
            .flat_map(|x| x.iter().copied())
            .max()
            .unwrap_or(0);
        if color == 0 {
            panic!("需要至少两种颜色（否则自然是可以密铺的）")
        }
        let clens = ((color - 1).max(1).ilog(2) + 1) as i32;
        let blens = ((blocks[0].len() - 1).max(1).ilog(2) + 1) as i32;
        let mut t = 1;
        nodes.iter_mut().flat_map(|x| x.iter_mut()).for_each(|x| {
            *x = t;
            t += blens
        });
        h_edges.iter_mut().flat_map(|x| x.iter_mut()).for_each(|x| {
            *x = t;
            t += clens
        });
        v_edges.iter_mut().flat_map(|x| x.iter_mut()).for_each(|x| {
            *x = t;
            t += clens
        });
        let mut restrictions =
            // edges:
            2 // 2 prints
            *(h_rules as i32*v+h*v_rules as i32) // prints calls
            *blocks[0].len() as i32 // rules[bidx].iter().copied().enumerate() loop
            *clens // color_bits loop
            + // nodes
            h*v
            *((1<<blens)-blocks[0].len() as i32) // node restrictions, 1 2 3 4 means col = 1111 is not acceptable, needs (1<<blens)-blocks[0].len() as i32 restrictions.
        ;
        let long = vec![];
        println! {"c {h} x {v} tiling with {:?}", if t>1000 {&long} else {&nodes}}
        println! {"c h[{}][{}] = {:?}",h_edges.len(),h_edges[0].len(), if t>1000 {&long} else {&h_edges}}
        println! {"c v[{}][{}] = {:?}",v_edges.len(),v_edges[0].len(), if t>1000 {&long} else {&v_edges}}
        println! {"p cnf {t} {}", restrictions}
        for i in 0..h as usize {
            for j in 0..v as usize {
                let mut edges = |rel, cur, edge, bidx: usize| {
                    // block idx=0 or 2
                    restrictions -= prints(rel, edge, &blocks, bidx + 0, blens, clens);
                    restrictions -= prints(cur, edge, &blocks, bidx + 1, blens, clens)
                };
                let cur_node = nodes[i][j];
                println! {"c curr idx {i} {j} starts with {cur_node}"}
                if i < h as usize - 1 {
                    println! {"c up --- down"}
                    edges(nodes[i + 1][j], cur_node, h_edges[i][j], 0)
                } else if cyc_h {
                    println! {"c cyc_h {cur_node} at {i},{j}"}
                    edges(nodes[0 + 0][j], cur_node, h_edges[i][j], 0)
                }
                if j < v as usize - 1 {
                    println! {"c left | right"}
                    edges(nodes[i][j + 1], cur_node, v_edges[i][j], 2)
                } else if cyc_v {
                    println! {"c cyc_v {cur_node} at {i},{j}"}
                    edges(nodes[i][0 + 0], cur_node, v_edges[i][j], 2)
                }
                println! {"c restrict"}
                for t in blocks[0].len()..(1 << blens) {
                    for i in 0..blens {
                        print!(
                            "{} ",
                            (cur_node + i) * if (1 << i) & t != 0 { -1 } else { 1 }
                        )
                    }
                    restrictions -= 1;
                    println!("0")
                }
            }
        }
        assert_eq!(restrictions, 0);
        // [&mut nodes,&mut h_edges,&mut v_edges].iter_mut().flat_map(|x|x.iter_mut().flat_map(|x|x.iter_mut())).for_each(|x|{*x=t;t+=1});
    }
}
fn prints(
    node: i32,
    edge: i32,
    rules: &[&[u8]],
    bidx: usize,
    block_bits: i32,
    color_bits: i32,
) -> i32 {
    let mut zeros = 0;
    for (n, r) in rules[bidx].iter().copied().enumerate() {
        println! {"c rules {n}, color={r}, direction=`{}`",['W','S','A','D'][bidx]}
        for c in 0..color_bits {
            for i in 0..block_bits {
                print!("{} ", (node + i) * if (1 << i) & n != 0 { -1 } else { 1 })
            }
            println! {"{} 0", (edge + c) * if (1 << c) & r != 0 { 1 } else { -1 }}
            zeros += 1
        }
    }
    zeros
}
fn main() {
    // 红黄蓝绿
    let mut w = std::env::args()
        .nth(3)
        .map_or(vec![1 as Color, 2, 0, 0, 1, 2, 2, 2, 3, 2], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });
    let mut s = std::env::args()
        .nth(4)
        .map_or(vec![2 as Color, 3, 2, 2, 0, 2, 2, 1, 0, 1], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });
    let mut a = std::env::args()
        .nth(5)
        .map_or(vec![2 as Color, 2, 2, 3, 3, 3, 0, 0, 3, 3], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });
    let mut d = std::env::args()
        .nth(6)
        .map_or(vec![3 as Color, 3, 2, 3, 2, 2, 3, 0, 0, 3], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });

    let mut hs = std::collections::BTreeSet::new();
    [&w, &s, &a, &d]
        .iter()
        .flat_map(|x| x.iter().copied())
        .for_each(|it| {
            hs.insert(it);
        });
    [&mut w, &mut s, &mut a, &mut d]
        .iter_mut()
        .flat_map(|x| x.iter_mut())
        .for_each(|x| *x = hs.range(..*x).count() as Color);
    if w.len() == s.len() && s.len() == a.len() && a.len() == d.len() {
        r#gen(
            std::env::args()
                .nth(1)
                .map(|x| x.parse::<i32>().unwrap_or(10))
                .unwrap_or(10),
            std::env::args()
                .nth(2)
                .map(|x| x.parse::<i32>().unwrap_or(10))
                .unwrap_or(10),
            [&w, &s, &a, &d],
        )
    }
}
