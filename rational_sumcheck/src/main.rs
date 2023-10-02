use gkr_protocol::circuit::{Circuit, CircuitLayer, Gate, GateType};
use gkr_protocol::{Prover, Verifier, ProverMessage, VerifierMessage};
use ark_std::{UniformRand, test_rng};
use ark_ff::{Fp64, MontBackend, MontConfig, PrimeField};

fn gen_circuit() -> Circuit{
  return Circuit::new(
        vec![
            CircuitLayer::new(
                vec![
                    Gate::new(
                        GateType::Mul,
                        [0, 1],
                    ),
                    Gate::new(
                        GateType::Mul,
                        [2, 3],
                    ),
                ],
            ),
            CircuitLayer::new(
                vec![
                    Gate::new(
                        GateType::Mul,
                        [0, 0],
                    ),
                    Gate::new(
                        GateType::Mul,
                        [1, 1],
                    ),
                    Gate::new(
                        GateType::Mul,
                        [1, 2],
                    ),
                    Gate::new(
                        GateType::Mul,
                        [3, 3],
                    ),
                ],
            ),
        ],
        4,
  );
}

fn test1(){
  let circuit = gen_circuit();

  let layers = circuit.evaluate(&[3, 2, 3, 1]);
  assert_eq!(
    layers.layers,
    vec![vec![36, 6], vec![9, 4, 6, 1], vec![3, 2, 3, 1]]
  );

  // Test that mul_1 evaluates to 0 on all inputs except
  // ((0, 0), (0, 0), (0, 0))
  // ((0, 1), (0, 1), (0, 1))
  // ((1, 0), (0, 1), (1, 0))
  // ((1, 1), (1, 1), (1, 1))
  for a in 0..4 {
    for b in 0..4 {
      for c in 0..4 {
        let expected = ((a == 0 || a == 1) && a == b && a == c)
            || a == 2 && b == 1 && c == 2
            || a == b && b == c && a == 3;
        assert_eq!(circuit.mul_i(1, a, b, c), expected, "{a} {b} {c}");
      }
    }
  }
}

fn test2(){
  let rng = &mut test_rng();
  #[derive(MontConfig)]
  #[modulus = "389"]
  #[generator = "2"]
  struct FrConfig;

  type Fp389 = Fp64<MontBackend<FrConfig, 1>>;

  let circuit = gen_circuit();

  let input = [
      Fp389::from_bigint(3u32.into()).unwrap(),
      Fp389::from_bigint(2u32.into()).unwrap(),
      Fp389::from_bigint(3u32.into()).unwrap(),
      Fp389::from_bigint(1u32.into()).unwrap(),
  ];

  let expected_outputs = [
      Fp389::from_bigint(36u32.into()).unwrap(),
      Fp389::from_bigint(6u32.into()).unwrap(),
  ];

  let mut prover = Prover::new(circuit.clone(), &input);

  // At the start of the protocol Prover sends a function $W_0$
  // mapping output gate labels to output values.
  let circuit_outputs_message = prover.start_protocol();

  assert_eq!(
      circuit_outputs_message,
      ProverMessage::Begin {
          circuit_outputs: expected_outputs.to_vec()
      }
  );

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

fn main(){
  test1();
}
