#[allow(warnings)]
mod bindings;

use bindings::{
    wasi::nn::{
        graph::{load, ExecutionTarget, Graph, GraphEncoding},
        tensor::{Tensor, TensorType},
    },
    Guest,
};
use ndarray::Array;
use rand::prelude::*;
use std::fs;
use tokenizers::Tokenizer;
struct Component;

impl Guest for Component {
    fn storygen() -> String {
        let model_data = fs::read("fixture/modified_model.onnx").unwrap();
        let graph = load(&[model_data], GraphEncoding::Onnx, ExecutionTarget::Cpu).unwrap();
        let exec_context = Graph::init_execution_context(&graph).unwrap();
        let tokenizer = Tokenizer::from_file("fixture/tokenizer.json").unwrap();
        let prompt = "Once upon a time";
        let encoding = tokenizer.encode(prompt, true).unwrap();
        let mut input_ids = encoding.get_ids().to_vec();
        let mut rng = thread_rng();
        let eos_token_id = tokenizer.token_to_id("<EOS>").unwrap_or(0);
        for _ in 0..300 {
            let sequence_length = input_ids.len();
            let dimensions = vec![1, sequence_length as u32];
            let input_f32: Vec<f32> = input_ids.iter().map(|&id| id as f32).collect();
            let mut input_data = Vec::with_capacity(input_f32.len() * 4);
            for &val in &input_f32 {
                input_data.extend_from_slice(&val.to_ne_bytes());
            }
            let tensor = Tensor::new(&dimensions, TensorType::Fp32, &input_data);
            exec_context.set_input("input_ids", tensor).unwrap();
            exec_context.compute().unwrap();
            let output_data = exec_context.get_output("logits").unwrap().data();
            let output_f32 = bytes_to_f32_vec(&output_data);
            let vocab_size = output_f32.len() / sequence_length;
            let start = (sequence_length - 1) * vocab_size;
            let end = sequence_length * vocab_size;
            let last_token_logits = &output_f32[start..end];
            let probabilities = softmax(last_token_logits);
            let dist = rand::distributions::WeightedIndex::new(&probabilities).unwrap();
            let next_token = dist.sample(&mut rng);
            input_ids.push(next_token as u32);
            let generated_text = tokenizer.decode(&input_ids, true).unwrap();
            if generated_text.ends_with("\n\n") {
                break;
            }
            if next_token as u32 == eos_token_id {
                break;
            }
        }
        tokenizer
            .decode(&input_ids, true)
            .unwrap()
            .trim()
            .to_string()
    }
}
pub fn bytes_to_f32_vec(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let logits_array = Array::from_vec(logits.to_vec());
    let exp_logits = logits_array.mapv(f32::exp);
    let sum_exp = exp_logits.sum();
    (exp_logits / sum_exp).to_vec()
}

bindings::export!(Component with_types_in bindings);
