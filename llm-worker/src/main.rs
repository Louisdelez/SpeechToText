use std::io::{self, BufRead, Write};
use std::num::NonZeroU32;
use std::pin::pin;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    model_path: String,
    prompt: String,
    max_tokens: Option<i32>,
    use_gpu: bool,
}

#[derive(Serialize)]
struct Response {
    text: Option<String>,
    error: Option<String>,
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response {
                    text: None,
                    error: Some(format!("Invalid JSON: {e}")),
                };
                let _ = writeln!(&stdout, "{}", serde_json::to_string(&resp).unwrap());
                continue;
            }
        };

        let resp = match run_inference(&req) {
            Ok(text) => Response {
                text: Some(text),
                error: None,
            },
            Err(e) => Response {
                text: None,
                error: Some(e),
            },
        };

        let _ = writeln!(&stdout, "{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.lock().flush();
    }
}

fn run_inference(req: &Request) -> Result<String, String> {
    let backend = LlamaBackend::init().map_err(|e| format!("Backend init: {e}"))?;

    let n_gpu = if req.use_gpu { 1000 } else { 0 };
    let model_params = LlamaModelParams::default().with_n_gpu_layers(n_gpu);
    let model_params = pin!(model_params);

    let model = LlamaModel::load_from_file(&backend, &req.model_path, &model_params)
        .map_err(|e| format!("Load model: {e}"))?;

    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(4096).unwrap()));
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .map_err(|e| format!("Create context: {e}"))?;

    let tokens = model
        .str_to_token(&req.prompt, AddBos::Never)
        .map_err(|e| format!("Tokenize: {e}"))?;

    eprintln!("[llm-worker] Prompt tokens: {}", tokens.len());

    let mut batch = LlamaBatch::new(4096, 1);
    let last = (tokens.len() - 1) as i32;
    for (i, token) in (0i32..).zip(tokens.into_iter()) {
        batch
            .add(token, i, &[0], i == last)
            .map_err(|e| format!("Batch add: {e}"))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| format!("Decode: {e}"))?;

    let max_gen = req.max_tokens.unwrap_or(512);
    let max_tokens = batch.n_tokens() + max_gen;
    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::dist(1234),
        LlamaSampler::greedy(),
    ]);

    let mut output = String::new();
    let mut n_cur = batch.n_tokens();
    let mut decoder = encoding_rs::UTF_8.new_decoder();

    while n_cur < max_tokens {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);

        if n_cur == batch.n_tokens() {
            eprintln!("[llm-worker] First generated token: {:?} is_eog={}", token, model.is_eog_token(token));
        }

        if model.is_eog_token(token) {
            eprintln!("[llm-worker] EOG at token {}", n_cur);
            break;
        }

        let piece = model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|e| format!("Token to piece: {e}"))?;
        output.push_str(&piece);

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| format!("Batch add: {e}"))?;
        ctx.decode(&mut batch)
            .map_err(|e| format!("Decode: {e}"))?;
        n_cur += 1;
    }

    // Model + context + backend dropped here -> VRAM freed
    Ok(output.trim().to_string())
}
