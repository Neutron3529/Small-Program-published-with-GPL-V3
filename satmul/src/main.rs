use core::ops::Neg;
#[derive(Clone,Copy,Debug)]
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
    fn format(graph:Vec<(i32,i32,i32)>,node:i32)->String{
        format!(r##"c cnf format
p cnf {} {}
{}"##,node,graph.len(),graph.iter().map(|x|match x {(0,0,0)=>format!("c unexpected 0 0 0"),(i,0,0)=>format!("{i} 0"),(i,j,0)=>format!("{i} {j} 0"),(i,j,k)=>format!("{i} {j} {k} 0")}).collect::<Vec<String>>().join("\n"))
    }
    fn str2bin(input: &str, cntr: &mut i32) -> Vec<Bit> {
        input
        .bytes()
        .map(|x| match x {
            b'1' => Bit::Constant(true),
             b'0' => Bit::Constant(false),
             b'x' | b'X' => {
                 *cntr += 1;
                 Bit::Node(*cntr)
             }
             _ => panic!("数字字符串{}应为全部由0,1或者x组成的字符串", input),
        })
        .collect()
    }
    fn eq(self,rhs:Self)->Vec<(i32,i32,i32)>{
        match (self,rhs) {
            (Bit::Constant(x),Bit::Constant(y)) => {
                if x==y {vec![]} else {panic!("导出矛盾，节点不兼容")}
            }
            (Bit::Constant(true),Bit::Node(res)) | (Bit::Node(res),Bit::Constant(true)) => vec![(res,0,0)],
            (Bit::Constant(false),Bit::Node(res)) | (Bit::Node(res),Bit::Constant(false)) => vec![(-res,0,0)],
            (Bit::Node(x),Bit::Node(y)) => vec![(x,-y,0),(-x,y,0)]
        }
    }
    fn add(self, rhs:Self, node:&mut i32)->(Self,Self,Vec<(i32,i32,i32)>){
        let mut v=Vec::new();
        let (res,add)=match (self,rhs) {
            (Bit::Constant(false),res)|(res,Bit::Constant(false))=>(res,Bit::Constant(false)),
            (Bit::Constant(true),res)|(res,Bit::Constant(true))=>(-res,res),
            (Bit::Node(x),Bit::Node(y))=>{
                *node+=2;
                let curr=*node-1;
                let add=*node;
                v.push((x,y,-curr));  // x=0,y=0 => curr=0
                v.push((-x,y,curr));  // x=1,y=0 => curr=1
                v.push((x,-y,curr));  // x=0,y=1 => curr=1
                v.push((-x,-y,-curr));// x=1,y=1 => curr=0
                v.push((x,y,-add));   // x=0,y=0 => add=0
                v.push((-x,y,-add));  // x=1,y=0 => add=0
                v.push((x,-y,-add));  // x=0,y=1 => add=0
                v.push((-x,-y,add));  // x=1,y=1 => add=1
                (Bit::Node(curr),Bit::Node(add))
            }
        };
        (res,add,v)
    }
    fn mul(self, rhs:Self, node:&mut i32)->(Self,Vec<(i32,i32,i32)>){
        let mut v=Vec::new();
        let res=match (self,rhs) {
            (Bit::Constant(true),res)|(res,Bit::Constant(true))=>res,
            (Bit::Constant(false),_)|(_,Bit::Constant(false))=>Bit::Constant(false),
            (Bit::Node(x),Bit::Node(y))=>{
                *node+=1;
                v.push((-x,-y,*node)); // x=1,y=1 => node=1
                v.push((x,y,-*node)); // x=0,y=0 => node=0
                v.push((-x,y,-*node)); // x=1,y=0 => node=0
                v.push((x,-y,-*node)); // x=0,y=1 => node=0
                Bit::Node(*node)
            }
        };
        (res,v)
    }
}
fn cvrt<'a>(mut l:&'a str,mut r:&'a str,res:&'a str)->String{
    if l.len()<r.len(){
        std::mem::swap(&mut l, &mut r);
    }
    let nodes=&mut 0;
    let left=Bit::str2bin(l,nodes);
    let right=Bit::str2bin(r,nodes);
    println!("c {left:?}");
    println!("c {right:?}");
    let res=Bit::str2bin(res,nodes);
    let mut graph=vec![];
    let mut mul_res=vec![vec![Bit::Constant(false);right.len()];left.len()];

    for (i,l) in left.iter().copied().enumerate() {
        for (j,r) in right.iter().copied().enumerate() {
            let tmp=l.mul(r,nodes);
            mul_res[i][j]=tmp.0;
            graph.extend(tmp.1)
        }
    }
    let mut adds=vec![];
    for k in 1..(left.len()+right.len()) {
        let old_adds=adds;
        adds=vec![];
        let mut result=Bit::Constant(false);
        let mut adding;
        let mut g;
        // println!("c currently processing {k}");
        for add in old_adds.iter().copied() {
            println!("c adding {add:?} to {result:?}");
            (result,adding,g)=result.add(add,nodes);
            adds.push(adding);
            println!("c pre {g:?}");
            graph.extend(g);
        }
        for (i,j) in (0..k).map(|x|(x,k-x-1)).filter(|&x|x.0<left.len() && x.1<right.len()) {
            // println!("c result is {:?}+{:?}",result,mul_res[i][j]);
            (result,adding,g)=result.add(mul_res[i][j],nodes);
            // println!("c got {:?} + {:?}", result, adding);
            adds.push(adding);
            println!("c add {g:?}");
            graph.extend(g);
        }
        let target=res.get(k-1).copied().unwrap_or(Bit::Constant(false));
        // println!("c currently processing {k} - verify, result is {result:?}, target is {target:?}");
        println!("c eq {:?}",result.eq(target));
        graph.extend(result.eq(target))
    }
    // println!("c {} {} {}",graph.len(),adds.len(), nodes);
    Bit::format(graph,*nodes)
}
fn main(){
    // println!("{}",cvrt("1xxxxx1","1xxxxx1","1100111011111")); // 83*97
    // println!("{}",cvrt("1xxxxxxxxxxxxxx1","1xxxxxxxxxxxxxx1","1011100011110000001000010000101")); // 35267*38303
    println!("{}",cvrt("1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1","1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1","110000100110100111101000100101001110110010001110100000001011011")); // 2614647029*3004108631

//    println!("{}",cvrt("1xxxxxxxxxx1","1xxxxxxxxxx1","110010101010101000101001"));
//    println!("{}",cvrt("1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1","1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx1","1110110010001100110111110010100011101000001100111000010000011110011110000101101101100110100110000000101010110101110101001100110101110110001100110101110000011010010000110111101010000101010011111111110010111001101001100000011011101100010010001101110110001111"));
}
