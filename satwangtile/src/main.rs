type Color = u8;
use image::{Rgb, RgbImage};
use std::io::Read;
fn gen(file: &str, w: i32, h: i32, blocks: [&[Color]; 4]) -> (Vec<[Color; 4]>, i32, i32) {
    let mut f = std::fs::File::open(file).expect(&format!("File `{file}` cannot be opened."));
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect(&format!("File `{file}` cannot be read."));
    let bitw = blocks[0].len().ilog2() as usize + 1;
    println!("bitw = {bitw}");
    let mut vec = vec![0; (w * h) as usize];
    contents
        .split('\n')
        .flat_map(|x| x.split(' '))
        .filter_map(|x| x.parse::<i32>().ok())
        .filter(|&x| x <= w * h * 4 && x > 0)
        .for_each(|y| {
            let x = (y - 1) as usize;
            *vec.get_mut(x / bitw).unwrap_or(&mut 0) |= 1 << (x % bitw)
        });
    for j in 0..h as usize {
        for i in 0..w as usize {
            print!("{:X}", vec[j * w as usize + i])
        }
        println!();
    }
    (
        vec.into_iter()
            .map(|x| {
                [
                    blocks[0][x as usize],
                    blocks[1][x as usize],
                    blocks[2][x as usize],
                    blocks[3][x as usize],
                ]
            })
            .collect(),
        w,
        h,
    )
}
fn main() {
    let mut w = std::env::args()
        .nth(4)
        .map_or(vec![1 as Color, 2, 0, 0, 1, 2, 2, 2, 3, 2], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });
    let mut s = std::env::args()
        .nth(5)
        .map_or(vec![2 as Color, 3, 2, 2, 0, 2, 2, 1, 0, 1], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });
    let mut a = std::env::args()
        .nth(6)
        .map_or(vec![2 as Color, 2, 2, 3, 3, 3, 0, 0, 3, 3], |x| {
            x.split(',').map(|x| x.parse().unwrap()).collect()
        });
    let mut d = std::env::args()
        .nth(7)
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
        eprintln!("brick len = {}",w.len());
        let (colmap, width, height) = gen(
            &std::env::args().nth(1).expect("input file needed"),
            std::env::args()
                .nth(2)
                .map(|x| x.parse::<i32>().unwrap_or(10))
                .unwrap_or(10),
            std::env::args()
                .nth(3)
                .map(|x| x.parse::<i32>().unwrap_or(10))
                .unwrap_or(10),
            [&w, &s, &a, &d],
        );
        let mut img = RgbImage::new(width as u32 * 3, height as u32 * 3 + 6);
        let mut iter = colmap.into_iter();
        // 蓝绿红白
        let colors = [
            Rgb([0, 0, 255]),
            Rgb([0, 255, 0]),
            Rgb([255, 0, 0]),
            Rgb([255, 255, 255]),
            Rgb([255, 0, 255]),
        ];
        for sample in 0..(w.len().min((width as usize+1) / 2)) {
            let i = sample as u32;
            img.put_pixel(6 * i + 1, 0, colors[w[sample] as usize]);
            img.put_pixel(6 * i + 1, 2, colors[s[sample] as usize]);
            img.put_pixel(6 * i + 0, 1, colors[a[sample] as usize]);
            img.put_pixel(6 * i + 2, 1, colors[d[sample] as usize])
        }

        for j in 2..2 + height as u32 {
            for i in 0..width as u32 {
                let cols = iter.next().unwrap();
                for (n, [x, y]) in [[1, 0], [1, 2], [0, 1], [2, 1]].into_iter().enumerate() {
                    img.put_pixel(3 * i + x, 3 * j + y, colors[cols[n] as usize]);
                }
            }
        }
        img.save(&format!(
            "{}.png",
            std::env::args().nth(1).expect("input file needed")
        ))
        .expect("error while save png file.");
    }
}
