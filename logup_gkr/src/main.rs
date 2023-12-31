use gkr_protocol::circuit::{Circuit, CircuitLayer, Gate, GateType};
use gkr_protocol::{Prover, Verifier, ProverMessage, VerifierMessage};
use ark_std::{UniformRand, test_rng};
use ark_ff::{Fp64, MontBackend, MontConfig};
use std::collections::HashMap;

#[derive(MontConfig)]
#[modulus = "17"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

const LOOKUP_N:usize = 2;
const LOOKUP_K:usize = 3;
const FRAC_SUMCHECK_N:usize = LOOKUP_N+LOOKUP_K;

// generate the fractional sumcheck circuit
fn gen_circuit() -> Circuit{
  let mut v = vec![];
  for i in 0..FRAC_SUMCHECK_N{
    let mut layer1 = vec![];
    let mut layer2 = vec![];
    for j in 0..1<<i{
      layer1.append(&mut vec![Gate::new(GateType::Add,[0+4*j, 1+4*j]),
                              Gate::new(GateType::Add,[2+4*j, 3+4*j])]);
      layer2.append(&mut vec![Gate::new(GateType::Mul,[0+4*j, 3+4*j]),
                              Gate::new(GateType::Mul,[1+4*j, 2+4*j]),
                              Gate::new(GateType::Mul,[1+4*j, 3+4*j]),
                              Gate::new(GateType::Mul,[1+4*j, 3+4*j])]);
    }
    v.push(CircuitLayer::new(layer1));
    v.push(CircuitLayer::new(layer2));
  }
  return Circuit::new(v, 2* 1<<FRAC_SUMCHECK_N);
}

// verify that sum(p_i/q_i) == 0
fn verify_rational_sum(p: Vec<Fq>, q: Vec<Fq>){
  let rng = &mut test_rng();

  let circuit = gen_circuit();

  let mut input = [Fq::from(0); 2*1<<FRAC_SUMCHECK_N];
  for (i, (a, b)) in p.iter().zip(q.iter()).enumerate(){
    input[i*2]=*a;
    input[i*2+1]=*b;
  }

  println!("circuit input: {:?}",input);
  println!("circuit result layers: {:?}",circuit.evaluate(&input).layers);

  let mut prover = Prover::new(circuit.clone(), &input);

  let circuit_outputs_message = prover.start_protocol();

  let output_vec;
  match circuit_outputs_message {
    ProverMessage::Begin {ref circuit_outputs} => output_vec = (*circuit_outputs).clone(),
    _ => panic!("{:?}", circuit_outputs_message)
  }

  println!("fractional sum (should be zero): {:?}",output_vec);
  assert_eq!(output_vec[0], Fq::from(0));
  assert_ne!(output_vec[1], Fq::from(0));

  let mut verifier = Verifier::new(circuit.clone());
  let verifier_message = verifier
      .receive_prover_msg(circuit_outputs_message, rng)
      .unwrap();

  let mut r_i = match verifier_message {
      VerifierMessage::R { r } => r,
      _ => panic!(),
  };

  for i in 0..circuit.layers().len() {
      let msg = prover.start_round(i, &r_i);
      let num_vars = 2 * circuit.num_vars_at(i + 1).unwrap();
      verifier.receive_prover_msg(msg, rng).unwrap();
      for j in 0..(num_vars - 1) {
          let prover_msg = prover.round_msg(j);
          let verifier_msg = verifier.receive_prover_msg(prover_msg, rng).unwrap();
          prover.receive_verifier_msg(verifier_msg);
      }
      let last_rand = verifier.final_random_point(rng).unwrap();
      prover.receive_verifier_msg(last_rand);
      let prover_msg = prover.round_msg(num_vars - 1);
      let verifier_msg = verifier.receive_prover_msg(prover_msg, rng).unwrap();
      match verifier_msg {
          VerifierMessage::R { r } => r_i = r,
          _ => panic!("{:?}", verifier_msg),
      }
  }
  assert!(verifier.check_input(&input));
}

//verify that forall i,j: ws[i][j] in t
fn verify_lookup(ws: Vec<Vec<Fq>>, t: Vec<Fq>){
  let rng = &mut test_rng();
  let mut m_hashmap = HashMap::new();
  for w in &ws{
    for x in w{
      *m_hashmap.entry(x).or_insert(0) += 1;
    }
  }
  let mut m = [Fq::from(0) ; 1<<LOOKUP_N];
  for (i,x) in t.iter().enumerate(){
    match m_hashmap.get(&x) {
      Some(c) => {m[i]=Fq::from(*c);},
      None => {}
    }
  }
  let alpha=Fq::rand(rng);
  let mut p = Vec::new();
  for x in m{
    for _ in 0 .. (1<<LOOKUP_K)-1{
      p.push(Fq::from(-1));
    }
    p.push(x);
  }
  let mut q = Vec::new();
  for i in 0..(1<<LOOKUP_N){
    for j in 0 .. (1<<LOOKUP_K)-1{
      q.push(Fq::from(alpha-ws[j][i]));
    }
    q.push(Fq::from(alpha-t[i]));
  }
  println!("m: {:?}",m);
  println!("t: {:?}",t);
  println!("p: {:?}",p);
  println!("q: {:?}",q);
  verify_rational_sum(p, q);
  println!("successfully verified.");
}

fn main(){
  let ws = vec![vec![Fq::from(1),Fq::from(2),Fq::from(1),Fq::from(1)],
                vec![Fq::from(2),Fq::from(3),Fq::from(1),Fq::from(1)],
                vec![Fq::from(2),Fq::from(3),Fq::from(1),Fq::from(1)],
                vec![Fq::from(4),Fq::from(4),Fq::from(4),Fq::from(4)],
                vec![Fq::from(4),Fq::from(4),Fq::from(4),Fq::from(4)],
                vec![Fq::from(2),Fq::from(3),Fq::from(1),Fq::from(1)],
                vec![Fq::from(4),Fq::from(4),Fq::from(4),Fq::from(4)]];
  let t = vec![Fq::from(1),Fq::from(2),Fq::from(3),Fq::from(4)];
  verify_lookup(ws, t);
}




