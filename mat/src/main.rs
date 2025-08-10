#![feature(generic_const_exprs)]
use core::{
    fmt,
    ops::{Add, AddAssign, Index, IndexMut, Mul},
};
#[derive(Clone)]
pub struct Matrix<T, const M: usize, const N: usize>([T; M * N])
where
    [(); M * N]:;
impl<T: Default + Copy, const M: usize, const N: usize> Matrix<T, M, N>
where
    [(); M * N]:,
{
    fn new() -> Self {
        Self([Default::default(); M * N])
    }
}
impl<T: fmt::Display, const M: usize, const N: usize> fmt::Display for Matrix<T, M, N>
where
    [(); M * N]:,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if N > 0 {
            for m in 0..M {
                for n in 0..N - 1 {
                    write!(f, "{} ", self[(m, n)])?
                }
                writeln!(f, "{}", self[(m, N - 1)])?
            }
        }
        Ok(())
    }
}
impl<T, const M: usize, const N: usize> Index<(usize, usize)> for Matrix<T, M, N>
where
    [(); M * N]:,
{
    type Output = T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0 * M + index.1]
    }
}
impl<T, const M: usize, const N: usize> IndexMut<(usize, usize)> for Matrix<T, M, N>
where
    [(); M * N]:,
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0 * M + index.1]
    }
}
impl<T: AddAssign<T>, const M: usize, const N: usize> Add for Matrix<T, M, N>
where
    [(); M * N]:,
{
    type Output = Matrix<T, M, N>;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.0
            .iter_mut()
            .zip(rhs.0.into_iter())
            .for_each(|(l, r)| *l += r);
        self
    }
}

impl<T: AddAssign<T> + Copy, const M: usize, const N: usize> Add<&Matrix<T, M, N>>
    for Matrix<T, M, N>
where
    [(); M * N]:,
{
    type Output = Matrix<T, M, N>;
    fn add(mut self, rhs: &Self) -> Self::Output {
        self.0
            .iter_mut()
            .zip(rhs.0.iter().copied())
            .for_each(|(l, r)| *l += r);
        self
    }
}
impl<
    T: AddAssign + Mul<Output = T> + Copy + Default,
    const I: usize,
    const J: usize,
    const K: usize,
> Mul<&Matrix<T, J, K>> for &Matrix<T, I, J>
where
    [(); I * K]:,
    [(); I * J]:,
    [(); J * K]:,
{
    type Output = Matrix<T, I, K>;
    fn mul(self, rhs: &Matrix<T, J, K>) -> Self::Output {
        let mut res = [Default::default(); I * K];
        for i in 0..I {
            for j in 0..J {
                for k in 0..K {
                    res[i * K + k] += self.0[i * J + j] * rhs.0[j * K + k]
                }
            }
        }
        Matrix(res)
    }
}
const MOD: u16 = 10;
fn main() {
    let mut mat = Matrix::<u16, 20, 20>::new();
    for i in 0..19 {
        mat[(i, i + 1)] = 1;
        mat[(19, i)] = 1;
    }
    mat[(19, 19)] = 1;
    // println!("{mat}");
    let mut total = std::env::args()
        .nth(1)
        .map(|x| x.parse::<u128>().ok())
        .flatten()
        .unwrap_or(153547986096937705890316909406143682736)
        / std::env::args()
            .nth(2)
            .map(|x| x.parse::<u128>().ok())
            .flatten()
            .unwrap_or(1);
    let test_idx = total;

    let mut curr = 1;
    let mut res = Matrix::<u16, 20, 20>::new();
    for i in 0..20 {
        res[(i, i)] = 1
    }
    while total != 0 {
        if total & curr != 0 {
            res = &res * &mat;
            total -= curr;
            res.0.iter_mut().for_each(|x| *x %= MOD)
        }
        mat = &mat * &mat;
        curr <<= 1;
        mat.0.iter_mut().for_each(|x| *x %= MOD)
    }
    println!("{res}");
    let target = [1, 8, 4, 4, 6, 7, 4, 4, 0, 7, 3, 7, 0, 9, 5, 5, 1, 6, 1, 5].map(|x| x % MOD);
    let mut res = &res * &Matrix::<u16, 20, 1>(target);
    res.0 = res.0.map(|x| x % MOD);
    println!("{test_idx} is {}: {:?}", res.0 == target, res.0);
}

// fn main() {
//     let mut mat = Matrix::<u16, 20, 20>::new();
//     for i in 0..19 {
//         mat[(i, i + 1)] = 1;
//         mat[(19, i)] = 1;
//     }
//     mat[(19, 19)] = 1;
//     println!("{mat}");
//     let correct_res = [1, 8, 4, 4, 6, 7, 4, 4, 0, 7, 3, 7, 0, 9, 5, 5, 1, 6, 1, 5].map(|x|x%MOD);
//
//     for mut total in (1..20).map(|x|(MOD as usize).pow(x) - 1).chain((0..20).map(|x|(MOD as usize).pow(x) + 1)).chain(1..usize::MAX) {
//         let mut res = Matrix::<u16, 20, 20>::new();
//         for i in 0..20 {
//             res[(i, i)] = 1
//         }
//
//         let idx = total;
//         let mut curr = 1;
//         while total != 0 {
//             if total & curr != 0 {
//                 res = &res * &mat;
//                 total -= curr;
//                 res.0.iter_mut().for_each(|x| *x %= MOD)
//             }
//             mat = &mat * &mat;
//             curr <<= 1;
//             mat.0.iter_mut().for_each(|x| *x %= MOD)
//         }
//         mat = res.clone();
//
//         // println!("{res}");
//         let mut res_vec = &res * &Matrix::<u16, 20, 1>([1, 8, 4, 4, 6, 7, 4, 4, 0, 7, 3, 7, 0, 9, 5, 5, 1, 6, 1, 5]);
//         res_vec.0.iter_mut().for_each(|x|*x%=MOD);
//         if idx & 262143 == 0 {
//             println!("{idx:10}: {:?}\n{res}", res_vec.0);
//         }
//         if res_vec.0 == correct_res {
//             println!("Found: {idx:10}: {:?}\n{res}", res_vec.0);
//             break
//         }
//     }
// }
