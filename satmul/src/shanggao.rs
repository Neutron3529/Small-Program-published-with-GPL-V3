// 使用3-SAT计算毕达哥拉斯染色问题
type i=i32;
type i2=i64;
fn shanggao(c:i, target:&mut Vec<(i,i,i)>){
    let csq=c as i2*c as i2;
    for a in 1..(((csq/2) as f64).sqrt() as i+1) {
        let bsq=csq-a as i2*a as i2;
        let b=(bsq as f64).sqrt() as i;
        if (b as i2).pow(2) == bsq {
            // println!("{a} {b} {c}");
            target.push((a,b,c))
        }
    }
}
fn full(){
    let to=std::env::args().nth(1).map_or(7865,|x|x.parse::<i>().unwrap_or(7865));
    let target=&mut vec![];
    for c in 5..=to {
        shanggao(c,target)
    }
    println!("c shanggao biparted graph");
    let mut hs=std::collections::HashSet::new();
    target.iter().for_each(|(a,b,c)|{[a,b,c].map(|x|hs.insert(*x));});
    println!("p cnf {} {}",to,target.len()*2+to as usize-hs.len());
    (1..=to).for_each(|x|if !hs.contains(&x) {println!("{x} 0")});
    target.iter().for_each(|(a,b,c)|{
        println!("{} {} {} 0",a,b,c);
        println!("{} {} {} 0",-a,-b,-c);
    })
}


fn shanggao_simplified(c:i, target:&mut Vec<[i;3]>){
    let csq=c as i2*c as i2;
    for a in 1..(((csq/2) as f64).sqrt() as i+1) {
        let bsq=csq-a as i2*a as i2;
        let b=(bsq as f64).sqrt() as i;
        if (b as i2).pow(2) == bsq {
            // println!("{a} {b} {c}");
            target.push([a,b,c])
        }
    }
}

fn simplified(){
    let to=std::env::args().nth(1).map_or(7865,|x|x.parse::<i>().unwrap_or(7865));
    let target=&mut vec![];
    for c in 5..=to {
        shanggao_simplified(c,target)
    }
    println!("c shanggao biparted graph");
    let mut hs=std::collections::BTreeSet::from([0]);
    target.iter().for_each(|it|{it.map(|x|hs.insert(x));});
    println!("p cnf {} {}",to,target.len()*2+to as usize-hs.len());
    (1..=to).for_each(|x|if !hs.contains(&x) {println!("{x} 0")});
    target.iter().map(|x|x.map(|x|hs.range(..x).count() as i)).for_each(|[a,b,c]|{
        println!("{} {} {} 0",a,b,c);
        println!("{} {} {} 0",-a,-b,-c);
    })
}

fn main(){
    simplified()
}
