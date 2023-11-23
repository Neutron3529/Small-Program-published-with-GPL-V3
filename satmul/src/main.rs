#![feature(iter_intersperse)]
use core::ops::Neg;
use std::collections::{HashSet,HashMap};
#[derive(Clone,Copy,Debug,PartialEq,PartialOrd,Eq,Ord)]
pub enum Bit{
    Constant(bool),
    Node(i32), // Node(-1)代表1不成立
}
impl Neg for Bit {
    type Output=Self;
    fn neg(self)->Self{
        match self {
            Self::Constant(x)=>Self::Constant(!x),
            Self::Node(x)=>Self::Node(-x)
        }
    }
}
impl Bit {
    pub fn format(and:&HashMap<(i32,i32),Self>,xor:&HashMap<(i32,i32),Self>,ceq:&HashSet<i32>,neq:&HashSet<(i32,i32)>,node:i32)->String{
        let mut total=ceq.len()+neq.len();
        let mut and=and.iter().map(|(k,v)|(k.0,k.1,v)).collect::<Vec<_>>();
        and.sort_unstable();
        let and=and.iter().map(|(x,y,r)|{
            match r {
                Self::Constant(false)=>{total+=2;format!("{x} 0\n{y} 0")},
                Self::Constant(true)=>String::new(),
                Self::Node(z)=>{total+=4;format!("{} {} {} 0\n{} {} {} 0\n{} {} {} 0\n{} {} {} 0",x,y,-z,x,-y,-z,-x,y,-z,-x,-y,z)}
            }
        }).filter(|x|x.len()>0).intersperse(String::from("\n")).collect::<String>();
        let mut xor=xor.iter().map(|(k,v)|(k.0,k.1,v)).collect::<Vec<_>>();
        xor.sort_unstable();
        let xor=xor.iter().map(|(x,y,r)|{
            match r {
                Self::Constant(false)=>{total+=2;format!("{} {} 0\n{} {} 0",x,-y,-x,y)},
                Self::Constant(true)=>{total+=2;format!("{} {} 0\n{} {} 0",x,y,-x,-y)},
                Self::Node(z)=>{total+=4;format!("{} {} {} 0\n{} {} {} 0\n{} {} {} 0\n{} {} {} 0",x,y,-z,x,-y,z,-x,y,z,-x,-y,-z)}
            }
        }).filter(|x|x.len()>0).intersperse(String::from("\n")).collect::<String>();
        let mut ceq=ceq.into_iter().collect::<Vec<_>>();
        ceq.sort_unstable();
        let ceq=ceq.iter().map(|x|format!("{x} 0")).intersperse(String::from("\n")).collect::<String>();
        let mut neq=neq.into_iter().collect::<Vec<_>>();
        neq.sort_unstable();
        let neq=neq.iter().map(|(x,y)|format!("{x} {y} 0")).intersperse(String::from("\n")).collect::<String>();
        let all=[and,xor,ceq,neq].into_iter().filter(|x|x.len()>0).intersperse(String::from("\n")).collect::<String>();
        format!(r##"c cnf format
p cnf {} {}
{}"##,node,total,all)
    }
    pub fn calc(x:i32,y:i32,node:&mut i32,hm:&mut HashMap<(i32,i32),Self>)->Self{
        *hm.entry(Self::sort(x,y)).or_insert_with(||{*node+=1;Bit::Node(*node)})
    }
    pub fn sort(x:i32,y:i32)->(i32,i32){
        if x<y {(x,y)} else {(y,x)}
    }
    pub fn str2bin(input: &str, cntr: &mut i32) -> Vec<Self> {
        input
        .bytes()
        .map(|x| match x {
            b'1' => Self::Constant(true),
             b'0' => Self::Constant(false),
             b'x' | b'X' => {
                 *cntr += 1;
                 Self::Node(*cntr)
             }
             _ => panic!("数字字符串{}应为全部由0,1或者x组成的字符串", input),
        })
        .collect()
    }
    pub fn eq(self,rhs:Self,ceq:&mut HashSet<i32>,neq:&mut HashSet<(i32,i32)>){
        match (self,rhs) {
            (Self::Constant(x),Self::Constant(y)) => {
                if x!=y {panic!("导出矛盾，节点不兼容")}
            }
            (Self::Constant(true),Self::Node(res)) | (Self::Node(res),Self::Constant(true)) => {ceq.insert(res);},
            (Self::Constant(false),Self::Node(res)) | (Self::Node(res),Self::Constant(false)) => {ceq.insert(-res);},
            (Self::Node(x),Self::Node(y)) => {neq.insert(Self::sort(x,-y));neq.insert(Self::sort(-x,y));}
        }
    }

    pub fn add(self, rhs:Self, node:&mut i32, and:&mut HashMap<(i32,i32),Self>, xor:&mut HashMap<(i32,i32),Self>)->(Self,Self){
        match (self,rhs) {
            (Self::Constant(false),res)|(res,Self::Constant(false))=>(res,Self::Constant(false)),
            (Self::Constant(true),res)|(res,Self::Constant(true))=>(-res,res),
            (Self::Node(x),Self::Node(y))=>{
                let xor=Self::calc(x,y,node,xor);
                let and=Self::calc(x,y,node,and);
                // v.push((x,y,-curr));  // x=0,y=0 => curr=0
                // v.push((-x,y,curr));  // x=1,y=0 => curr=1
                // v.push((x,-y,curr));  // x=0,y=1 => curr=1
                // v.push((-x,-y,-curr));// x=1,y=1 => curr=0
                // v.push((x,y,-add));   // x=0,y=0 => add=0
                // v.push((-x,y,-add));  // x=1,y=0 => add=0
                // v.push((x,-y,-add));  // x=0,y=1 => add=0
                // v.push((-x,-y,add));  // x=1,y=1 => add=1
                // (Self::Node(curr),Self::Node(add))
                (xor,and)
            }
        }
    }
    pub fn mul(self, rhs:Self, node:&mut i32, and:&mut HashMap<(i32,i32),Self>)->Self{
        match (self,rhs) {
            (Self::Constant(true),res)|(res,Self::Constant(true))=>res,
            (Self::Constant(false),_)|(_,Self::Constant(false))=>Self::Constant(false),
            (Self::Node(x),Self::Node(y))=>Self::calc(x,y,node,and)
        }
    }
    pub fn cvrt<'a>(mut l:&'a str,mut r:&'a str,res:&'a str)->String{
        if l.len()<r.len(){
            std::mem::swap(&mut l, &mut r);
        }
        let nodes=&mut 0;
        let left=Self::str2bin(l,nodes);
        let right=Self::str2bin(r,nodes);
        println!("c {left:?}");
        println!("c {right:?}");
        let res=Self::str2bin(res,nodes);
        let mut mul_res=vec![vec![Self::Constant(false);right.len()];left.len()];
        let and=&mut HashMap::new();
        let xor=&mut HashMap::new();
        let ceq=&mut HashSet::new();
        let neq=&mut HashSet::new();
        for (i,l) in left.iter().copied().enumerate() {
            for (j,r) in right.iter().copied().enumerate() {
                mul_res[i][j]=l.mul(r,nodes,and);
            }
        }
        let mut adds=vec![Self::Constant(false);left.len()+right.len()-1];
        let mut adds_cntr=vec![0;left.len()+right.len()-1]; // 记录进位的最大数值，默认每一个mul_res都是1
        for k in 0..(left.len()+right.len()-1) {
            let mut result=adds.get(k-1).copied().unwrap_or(Self::Constant(false));
            let mut adding;
            let mut new=vec![];
            for (i,j) in (0..k+1).map(|x|(x,k-x)).filter(|&x|x.0<left.len() && x.1<right.len()) {
                (result,adding)=result.add(mul_res[i][j],nodes,and,xor);
                new.push(adding);
            }
            // 开始进位加法，这里记录进位最大可能，不进行超过这一可能的进位
            let mut len=new.len();
            let mut idx=0;
            while len>0 {
                if let (Some(x),Some(cntr))=(adds.get_mut(k+idx),adds_cntr.get_mut(k+idx)) {
                    len+=*cntr; // 于是所有进位的最大值不会超过len(每个mul_res都进位+原来在这里的数字的最大值)
                    *cntr=len&1; // 将剩余进位保存，等待下次继续进位
                    len>>=1; // 进位
                    let mut curr=vec![];
                    *x=new.drain(..).fold(*x,|s,x|{
                        let (result,adding)=s.add(x,nodes,and,xor);
                        curr.push(adding);
                        result
                    });
                    new=curr;
                    idx+=1;
                } else {break}
            }
            new.into_iter().for_each(|x|x.eq(Self::Constant(false),ceq,neq));
            let target=res.get(k).copied().unwrap_or(Self::Constant(false));
            result.eq(target,ceq,neq);
        }
        Self::format(and,xor,ceq,neq,*nodes)
    }
}
fn main(){
    use std::str::FromStr;
    let mut args=std::env::args().skip(1);
    let m1=args.next().as_ref().map(|x|rug::Integer::from_str(x)).unwrap_or_else(||rug::Integer::from_str("2614647029")).unwrap();
    let m2=args.next().as_ref().map(|x|rug::Integer::from_str(x)).unwrap_or_else(||rug::Integer::from_str("3004108631")).unwrap();
    let res=args.next().as_ref().map(|x|rug::Integer::from_str(x).unwrap_or_else(|_|{println!("得到res，以此覆盖m1*m2的值");m1.clone()*&m2})).unwrap_or_else(||m1.clone()*&m2);
    let mut m1=m1.to_string_radix(2);
    let mut m2=m2.to_string_radix(2);

    let mut string=res.to_string_radix(2);
    unsafe {
        m1.as_mut_vec().reverse();
        m2.as_mut_vec().reverse();
        string.as_mut_vec().reverse();
        println!("c {m1}\nc {m2}");
        m1.as_mut_vec().iter_mut().for_each(|x|*x=b'x');
        m2.as_mut_vec().iter_mut().for_each(|x|*x=b'x');
    }
    println!("{}",Bit::cvrt(&m1,&m2,&string));
    // println!("{}",Bit::cvrt("1xxxxx1","1xxxxx1","1100111011111")); // 83*97
    // println!("{}",Bit::cvrt("1xxxxxxxxxxxxxx1","1xxxxxxxxxxxxxx1","1011100011110000001000010000101")); // 35267*38303
    // println!("{}",Bit::cvrt("1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1","1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1","110000100110100111101000100101001110110010001110100000001011011")); // 2614647029*3004108631
    // println!("{}",Bit::cvrt("10001001001000111101011111001101","x0111001001001110101101010101111","0xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"))
    // println!("{}",cvrt("1xxxxxxxxxx1","1xxxxxxxxxx1","110010101010101000101001"));
}
