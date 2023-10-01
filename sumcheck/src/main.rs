use ark_ff::fields::{Field, Fp64, MontBackend, MontConfig};
use ark_poly::{
    polynomial::multivariate::{SparsePolynomial, SparseTerm, Term},
    DenseMVPolynomial, Polynomial,
};
use ark_std::{UniformRand, test_rng};
use std::convert::AsRef;

#[derive(MontConfig)]
#[modulus = "17"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

const n:usize = 4;

fn print_poly(p: &SparsePolynomial<Fq,SparseTerm>){
  println!("{}",p.num_vars);
  for (coeff,terms) in p.terms.iter() {
    println!(" {coeff}");
    for (var,power) in terms.vars().iter().zip(terms.powers().iter()) {
      println!("  {var}");
      println!("  ^{power}");
    }
  }
}
fn sum_poly(p: &SparsePolynomial<Fq,SparseTerm>) -> Fq{
  let mut s=Fq::from(0);
  let x=&mut [Fq::from(-1); n];

  let max=2_i32.pow((p.num_vars) as u32);
  for idx in 0 .. max{
    s += p.evaluate(&x.to_vec());
    if idx==max-1{
     return s;
    }
    let mut j=x.len()-1;
    while x[j]==Fq::from(1){
      x[j]=Fq::from(-1);
      j-=1;
    }
    x[j]=Fq::from(1);
  }
  return s;
}

fn set_values(p: &SparsePolynomial<Fq,SparseTerm>, x: usize, values: &[Fq]) -> SparsePolynomial<Fq,SparseTerm>{
  let mut r = Vec::new();
  for (coeff,terms) in p.terms.iter() {
    let mut c=*coeff;
    let mut t=SparseTerm::new(vec![]);
    for (var,power) in terms.vars().iter().zip(terms.powers().iter()) {
      if *var == x {
        t=SparseTerm::new(vec![(0,*power)]);
      }else{
        c*=values[*var].pow(AsRef::as_ref(&[*power as u64]));
      }
    }
    r.push((Fq::from(c),t));
  }
  return SparsePolynomial::from_coefficients_vec(1, r);
}

fn prover_gen_s(p: &SparsePolynomial<Fq,SparseTerm>, i: usize, x: &mut [Fq]) -> SparsePolynomial<Fq,SparseTerm>{
  let mut res=SparsePolynomial::from_coefficients_vec(1,vec![]);
  let max=2_i32.pow((p.num_vars - i) as u32);
  for _idx in 0 .. max{
    res = res.clone() + set_values(p,i-1,x);

    let mut j=x.len()-1;
    while x[j]==Fq::from(1){
      x[j]=Fq::from(-1);
      j-=1;
    }
    x[j]=Fq::from(1);
  }
  println!("generated:");
  print_poly(&res);
  return res;
}

fn verify_si(ss: &[SparsePolynomial<Fq,SparseTerm>], rs:&[Fq], i: usize, s:Fq){
  let one = ss[i-1].evaluate(&vec![Fq::from(1)]);
  let neg = ss[i-1].evaluate(&vec![Fq::from(-1)]);
  println!("one {one}");
  println!("neg {neg}");
  println!("which is  {}",one+neg);
  if i==1{
    println!("should be {s}");
    return;
  }
  println!("should be {}",ss[i-2].evaluate(&vec![rs[i-2]]));
}

fn main() {
  let mut rng = test_rng();
  let p = SparsePolynomial::from_coefficients_vec(
    n,
    vec![
        (Fq::from(1), SparseTerm::new(vec![(0, 1)])),
        (Fq::from(1), SparseTerm::new(vec![(1, 1)])),
        (Fq::from(1), SparseTerm::new(vec![(2, 1),(3,5)])),
        (Fq::from(1), SparseTerm::new(vec![(3, 1)])),
    ],
  );
  let s = sum_poly(&p);
  println!("sum {s}");

  let rs = &mut[Fq::from(0); n];
  let ss:&mut[SparsePolynomial<Fq,SparseTerm>; n] = &mut Default::default();

  let x=&mut [Fq::from(-1); n];
  ss[0]=prover_gen_s(&p,1,x);
  verify_si(ss,rs,1,s);

  for i in 0..n-1{
    rs[i]=Fq::rand(&mut rng);
    println!("generated random {}",rs[i]);
    x[i]=rs[i];
    for j in i+1..n-1{
      x[j]=Fq::from(-1);
    }
    ss[i+1]=prover_gen_s(&p,i+2,x);
    verify_si(ss,rs,i+2,s);
  }

  rs[n-1]=Fq::rand(&mut rng);
  println!("generated random {}",rs[n-1]);
  let last = ss[n-1].evaluate(&vec![rs[n-1]]);
  println!("last is   {}",last);
  println!("should be {}", p.evaluate(&rs.to_vec()));
}




