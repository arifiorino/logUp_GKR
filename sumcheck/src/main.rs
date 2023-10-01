use ark_ff::fields::Field;
use ark_test_curves::bls12_381::Fq;
use ark_poly::{
    polynomial::multivariate::{SparsePolynomial, SparseTerm, Term},
    DenseMVPolynomial, Polynomial,
};
use ark_std::{One, UniformRand, test_rng};
use std::convert::AsRef;

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
  println!("should be {}",ss[i-2].evaluate(&vec![rs[i-1]]));
}

fn main() {
  let mut rng = test_rng();
  let p = SparsePolynomial::from_coefficients_vec(
    3,
    vec![
        (Fq::from(1), SparseTerm::new(vec![(0, 1),(1, 1),(2, 1)])),
    ],
  );
  let s=Fq::from(0);
  let mut r;

  let rs=&mut[Fq::from(0),Fq::from(0),Fq::from(0)];
  let ss=&mut[SparsePolynomial::from_coefficients_vec(1,vec![]),
              SparsePolynomial::from_coefficients_vec(1,vec![]),
              SparsePolynomial::from_coefficients_vec(1,vec![]),];

  let mut si = prover_gen_s(&p,1,&mut [Fq::from(0),Fq::from(-1),Fq::from(-1)]);
  ss[0]=si;
  verify_si(ss,rs,1,s);

  rs[0]=Fq::rand(&mut rng);
  si = prover_gen_s(&p,2,&mut [rs[0],Fq::from(0),Fq::from(-1)]);
  ss[1]=si;
  verify_si(ss,rs,2,s);
  r = Fq::rand(&mut rng);

  rs[1]=Fq::rand(&mut rng);
  si = prover_gen_s(&p,3,&mut [rs[0],rs[1],Fq::from(0)]);
  ss[2]=si;
  verify_si(ss,rs,3,s);

  rs[2]=Fq::rand(&mut rng);
  let last = ss[2].evaluate(&vec![rs[2]]);
  println!("last is  {}",last);
  println!("should be {}", p.evaluate(&rs.to_vec()));
}




