use ark_ff::fields::{Field, Fp64, MontBackend, MontConfig};
use ark_poly::evaluations::multivariate::multilinear::{MultilinearExtension,DenseMultilinearExtension};
use ark_std::{rand::RngCore, UniformRand, test_rng};

#[derive(MontConfig)]
#[modulus = "17"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

const n:usize = 3;

fn print_poly(p: &DenseMultilinearExtension<Fq>){
  for x in p.to_evaluations().iter() {
    print!("{x}, ");
  }
  println!("");
}

fn sum_frac(p: &DenseMultilinearExtension<Fq>, q: &DenseMultilinearExtension<Fq>) -> Fq{
  let mut s = Fq::from(0);
  for (a, b) in p.evaluations.iter().zip(q.evaluations.iter()) {
    s+=a/b;
  }
  return s
}

fn get_next_pq(p: &DenseMultilinearExtension<Fq>, q: &DenseMultilinearExtension<Fq>) -> (DenseMultilinearExtension<Fq>, DenseMultilinearExtension<Fq>){
  let mut pv: Vec<Fq> = vec![];
  let mut qv: Vec<Fq> = vec![];
  let mut prev=None;
  for (a, b) in p.evaluations.iter().zip(q.evaluations.iter()) {
    if prev==None{
      prev=Some((a,b))
    }else{
      let (c,d)=prev.expect("err");
      pv.push(a*d+c*b);
      qv.push(d*b);
      prev=None;
    }
  }
  return (DenseMultilinearExtension::from_evaluations_vec(p.num_vars-1,pv),
          DenseMultilinearExtension::from_evaluations_vec(q.num_vars-1,qv))
}

fn verify_i<R: RngCore>(rng:&mut R, ps: &mut [DenseMultilinearExtension<Fq>], qs: &mut [DenseMultilinearExtension<Fq>], i:usize){
  let r = Fq::rand(rng);
  let lambda = Fq::rand(rng);

  let left = ps[i].evaluate(&[r]).expect("err") + lambda * qs[i].evaluate(&[r]).expect("err");
  println!("a{}",qs[i].evaluate(&[r]).expect("err"));

  for j in i+1 .. n{
    ps[j]=ps[j].fix_variables(&[r]);
    qs[j]=qs[j].fix_variables(&[r]);
  }

  let right = ps[i+1].evaluate(&[Fq::from(1)]).expect("err")*qs[i+1].evaluate(&[Fq::from(0)]).expect("err") +
              ps[i+1].evaluate(&[Fq::from(0)]).expect("err")*qs[i+1].evaluate(&[Fq::from(1)]).expect("err") +
              lambda * qs[i+1].evaluate(&[Fq::from(1)]).expect("err") * qs[i+1].evaluate(&[Fq::from(0)]).expect("err");
  println!("b{}",qs[i+1].evaluate(&[Fq::from(1)]).expect("err") * qs[i+1].evaluate(&[Fq::from(0)]).expect("err"));
  print!("{left} ==? ");
  println!("{right}");
}

fn main() {
  let mut rng = test_rng();
  let p = DenseMultilinearExtension::from_evaluations_vec(3,vec![Fq::from(1),
                                                                 Fq::from(2),
                                                                 Fq::from(3),
                                                                 Fq::from(4),
                                                                 Fq::from(5),
                                                                 Fq::from(6),
                                                                 Fq::from(7),
                                                                 Fq::from(10)]);
  let q = DenseMultilinearExtension::from_evaluations_vec(3,vec![Fq::from(1),
                                                                 Fq::from(2),
                                                                 Fq::from(3),
                                                                 Fq::from(4),
                                                                 Fq::from(5),
                                                                 Fq::from(6),
                                                                 Fq::from(7),
                                                                 Fq::from(1)]);
  let s = sum_frac(&p,&q);
  let ps:&mut[DenseMultilinearExtension<Fq>; n] = &mut Default::default();
  let qs:&mut[DenseMultilinearExtension<Fq>; n] = &mut Default::default();
  ps[n-1]=p;
  qs[n-1]=q;
  for i in 0..n-1{
    let (p2,q2)=get_next_pq(&ps[n-1-i],&qs[n-1-i]);
    ps[n-2-i]=p2;
    qs[n-2-i]=q2;
    print_poly(&ps[n-2-i]);
    print_poly(&qs[n-2-i]);
  }

  for i in 0..n-1{
    verify_i(&mut rng, ps,qs,i);
  }

  let mut left = ps[0].evaluate(&[Fq::from(1)]).expect("err")*qs[0].evaluate(&[Fq::from(0)]).expect("err") + ps[0].evaluate(&[Fq::from(0)]).expect("err")*qs[0].evaluate(&[Fq::from(1)]).expect("err");
  println!("{left} ==? 0");
  left = qs[0].evaluate(&[Fq::from(0)]).expect("err") * qs[0].evaluate(&[Fq::from(1)]).expect("err");
  println!("{left} !=? 0");


}




