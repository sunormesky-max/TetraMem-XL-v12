// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — Neural Embedding Engine (ONNX)
//
// Granite-embedding-small-english-r2 integration:
//   - 47M params, 384-dim output, Apache 2.0
//   - CLS pooling + L2 normalization
//   - Input/output dimension validation against injection
//   - Graceful degradation when model unavailable
//   - Built-in minimal BPE tokenizer (no external tokenizers crate)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use ort::session::Session;
use ort::value::Tensor;
use serde::Deserialize;
use tracing::{debug, info, warn};

const NEURAL_DIM: usize = 384;
const MAX_INPUT_TOKENS: usize = 512;
const CACHE_CAPACITY: usize = 256;

struct LruCache<K, V> {
    entries: Vec<(K, V)>,
    capacity: usize,
}

impl<K: PartialEq, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(idx) = self.entries.iter().position(|(k, _)| k == key) {
            let entry = self.entries.remove(idx);
            self.entries.push(entry);
            Some(&self.entries.last().unwrap().1)
        } else {
            None
        }
    }

    fn insert(&mut self, key: K, value: V) {
        if let Some(idx) = self.entries.iter().position(|(k, _)| k == &key) {
            self.entries.remove(idx);
        }
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push((key, value));
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("model not loaded: {0}")]
    NotLoaded(String),
    #[error("tokenizer error: {0}")]
    Tokenizer(String),
    #[error("inference error: {0}")]
    Inference(String),
    #[error("output dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("output contains non-finite values at index {index}")]
    NonFiniteOutput { index: usize },
    #[error("input too long: {tokens} tokens > {max}")]
    InputTooLong { tokens: usize, max: usize },
}

// ── Minimal BPE Tokenizer ──────────────────────────────────────────────
// Loads a HuggingFace tokenizer.json (ByteLevel BPE) and encodes text
// into (input_ids, attention_mask) for the Granite / ModernBERT model.

struct BpeTokenizer {
    vocab: HashMap<String, u32>,
    #[allow(dead_code)]
    merges: Vec<(String, String)>,
    merge_rank: HashMap<(String, String), usize>,
    byte_to_char: [char; 256],
    cls_id: u32,
    sep_id: u32,
    unk_id: u32,
    pad_id: u32,
}

#[derive(Deserialize)]
struct TokenizerJson {
    model: TokenizerModel,
    #[serde(default)]
    added_tokens: Vec<AddedToken>,
}

#[derive(Deserialize)]
struct TokenizerModel {
    vocab: HashMap<String, u32>,
    #[serde(default, deserialize_with = "deserialize_merges")]
    merges: Vec<String>,
}

fn deserialize_merges<'de, D>(de: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MergeOrList {
        Flat(Vec<String>),
        Nested(Vec<[String; 2]>),
    }

    let val = MergeOrList::deserialize(de)?;
    match val {
        MergeOrList::Flat(strings) => Ok(strings),
        MergeOrList::Nested(pairs) => {
            Ok(pairs.into_iter().map(|[a, b]| format!("{a} {b}")).collect())
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AddedToken {
    id: u32,
    content: String,
    #[serde(default)]
    special: bool,
}

fn build_byte_to_char() -> [char; 256] {
    let good: Vec<u8> = (33..127).chain(161..173).chain(174u8..=255).collect();
    let mut bad: Vec<u8> = (0u8..=255).filter(|b| !good.contains(b)).collect();
    bad.sort();

    let mut table = ['\0'; 256];
    for &b in &good {
        table[b as usize] = b as char;
    }
    for (i, &b) in bad.iter().enumerate() {
        table[b as usize] = char::from_u32(256 + i as u32).unwrap_or('\u{fffd}');
    }
    table
}

fn gpt2_pretokenize(text: &str) -> Vec<&str> {
    let mut tokens: Vec<&str> = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut start = 0;

    while start < len {
        let matched_end = if bytes[start] == b'\''
            && start + 1 < len
            && matches!(bytes[start + 1], b's' | b't' | b'm' | b'd')
            && (start + 2 >= len || !bytes[start + 2].is_ascii_alphabetic())
        {
            start + 2
        } else if bytes[start] == b'\''
            && start + 2 < len
            && matches!(bytes[start + 2], b'e' | b'l')
            && matches!(bytes[start + 1], b'r' | b'v' | b'l')
            && (start + 3 >= len || !bytes[start + 3].is_ascii_alphabetic())
        {
            start + 3
        } else if bytes[start] == b' ' && start + 1 < len {
            let cat = classify_byte(bytes[start + 1]);
            if cat == ByteCat::Letter || cat == ByteCat::Digit || cat == ByteCat::Other {
                let mut end = start + 2;
                while end < len && classify_byte(bytes[end]) == cat {
                    end += 1;
                }
                end
            } else {
                let mut end = start + 1;
                while end < len && bytes[end].is_ascii_whitespace() {
                    end += 1;
                }
                end
            }
        } else {
            let cat = classify_byte(bytes[start]);
            match cat {
                ByteCat::Letter | ByteCat::Digit | ByteCat::Other => {
                    let mut end = start + 1;
                    while end < len && classify_byte(bytes[end]) == cat {
                        end += 1;
                    }
                    end
                }
                ByteCat::Space => {
                    let mut end = start + 1;
                    while end < len && bytes[end].is_ascii_whitespace() {
                        end += 1;
                    }
                    end
                }
            }
        };

        tokens.push(&text[start..matched_end]);
        start = matched_end;
    }

    tokens
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ByteCat {
    Letter,
    Digit,
    Other,
    Space,
}

fn classify_byte(b: u8) -> ByteCat {
    if b.is_ascii_alphabetic() {
        ByteCat::Letter
    } else if b.is_ascii_digit() {
        ByteCat::Digit
    } else if b.is_ascii_whitespace() {
        ByteCat::Space
    } else {
        ByteCat::Other
    }
}

impl BpeTokenizer {
    fn from_file(path: &Path) -> Result<Self, EmbeddingError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| EmbeddingError::Tokenizer(format!("read tokenizer.json: {e}")))?;

        let tj: TokenizerJson = serde_json::from_str(&raw)
            .map_err(|e| EmbeddingError::Tokenizer(format!("parse tokenizer.json: {e}")))?;

        let mut vocab = tj.model.vocab;

        for at in &tj.added_tokens {
            vocab.entry(at.content.clone()).or_insert(at.id);
        }

        let merges: Vec<(String, String)> = tj
            .model
            .merges
            .iter()
            .filter_map(|line| {
                if line.contains(' ') {
                    let mut parts = line.splitn(2, ' ');
                    Some((parts.next()?.to_string(), parts.next()?.to_string()))
                } else {
                    None
                }
            })
            .collect();

        let merge_rank: HashMap<(String, String), usize> = merges
            .iter()
            .enumerate()
            .map(|(i, pair)| (pair.clone(), i))
            .collect();

        let unk_id = vocab.get("[UNK]").copied().unwrap_or(0);

        let cls_id = vocab.get("[CLS]").copied().unwrap_or(unk_id);

        let sep_id = vocab.get("[SEP]").copied().unwrap_or(unk_id);

        let pad_id = vocab.get("[PAD]").copied().unwrap_or(sep_id);

        Ok(Self {
            vocab,
            merges,
            merge_rank,
            byte_to_char: build_byte_to_char(),
            cls_id,
            sep_id,
            unk_id,
            pad_id,
        })
    }

    fn bytes_to_bpe_chars(&self, text: &str) -> Vec<String> {
        text.bytes()
            .map(|b| self.byte_to_char[b as usize].to_string())
            .collect()
    }

    fn apply_bpe(&self, symbols: &[String]) -> Vec<String> {
        if symbols.len() <= 1 {
            return symbols.to_vec();
        }

        let mut syms: Vec<String> = symbols.to_vec();

        loop {
            let mut best_rank = usize::MAX;
            let mut best_pos = None;

            for i in 0..syms.len().saturating_sub(1) {
                if let Some(&rank) = self.merge_rank.get(&(syms[i].clone(), syms[i + 1].clone())) {
                    if rank < best_rank {
                        best_rank = rank;
                        best_pos = Some(i);
                    }
                }
            }

            let pos = match best_pos {
                Some(p) => p,
                None => break,
            };

            let merged = format!("{}{}", syms[pos], syms[pos + 1]);
            syms[pos] = merged;
            syms.remove(pos + 1);

            if syms.len() <= 1 {
                break;
            }
        }

        syms
    }

    fn encode(&self, text: &str, max_len: usize) -> (Vec<i64>, Vec<i64>) {
        let chunks = gpt2_pretokenize(text);

        let mut token_ids: Vec<u32> = Vec::new();

        token_ids.push(self.cls_id);

        for chunk in chunks {
            let bpe_chars = self.bytes_to_bpe_chars(chunk);
            let merged = self.apply_bpe(&bpe_chars);

            for token_str in &merged {
                let id = self.vocab.get(token_str).copied().unwrap_or(self.unk_id);
                token_ids.push(id);
            }
        }

        token_ids.push(self.sep_id);

        let effective_len = token_ids.len().min(max_len);
        token_ids.truncate(effective_len);

        let mut input_ids: Vec<i64> = token_ids.iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = vec![1i64; input_ids.len()];

        if input_ids.len() < max_len {
            let pad_count = max_len - input_ids.len();
            for _ in 0..pad_count {
                input_ids.push(self.pad_id as i64);
            }
        }

        let mut full_mask = attention_mask;
        let pad_needed = max_len.saturating_sub(full_mask.len());
        full_mask.extend(std::iter::repeat_n(0i64, pad_needed));

        full_mask.truncate(max_len);
        input_ids.truncate(max_len);

        (input_ids, full_mask)
    }
}

// ── Embedding Engine ───────────────────────────────────────────────────

pub struct EmbeddingEngine {
    session: Session,
    tokenizer: BpeTokenizer,
    model_path: PathBuf,
}

impl std::fmt::Debug for EmbeddingEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingEngine")
            .field("model_path", &self.model_path)
            .field("output_dim", &NEURAL_DIM)
            .finish()
    }
}

impl EmbeddingEngine {
    pub fn new(model_dir: &Path) -> Result<Self, EmbeddingError> {
        let onnx_path = model_dir.join("model_quantized.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !onnx_path.exists() {
            return Err(EmbeddingError::NotLoaded(format!(
                "ONNX model not found: {}",
                onnx_path.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(EmbeddingError::NotLoaded(format!(
                "tokenizer not found: {}",
                tokenizer_path.display()
            )));
        }

        validate_onnx_header(&onnx_path)?;

        info!(
            "Loading neural embedding model from {}",
            model_dir.display()
        );

        let session = Session::builder()
            .and_then(|mut b| b.commit_from_file(&onnx_path))
            .map_err(|e| EmbeddingError::Inference(format!("session create: {e}")))?;

        let tokenizer = BpeTokenizer::from_file(&tokenizer_path)?;

        let input_count = session.inputs().len();
        let output_count = session.outputs().len();
        debug!(
            "ONNX session ready: {} inputs, {} outputs",
            input_count, output_count
        );

        for (i, info) in session.inputs().iter().enumerate() {
            debug!("  input[{}]: name={}", i, info.name());
        }
        for (i, info) in session.outputs().iter().enumerate() {
            debug!("  output[{}]: name={}", i, info.name());
        }

        Ok(Self {
            session,
            tokenizer,
            model_path: model_dir.to_path_buf(),
        })
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f64>, EmbeddingError> {
        let (input_ids, attention_mask) = self.tokenizer.encode(text, MAX_INPUT_TOKENS);

        let token_count = attention_mask.iter().filter(|&&m| m == 1).count();
        if token_count > MAX_INPUT_TOKENS {
            return Err(EmbeddingError::InputTooLong {
                tokens: token_count,
                max: MAX_INPUT_TOKENS,
            });
        }

        let seq_len = input_ids.len();
        let shape = vec![1i64, seq_len as i64];

        let input_ids_tensor = Tensor::from_array((shape.clone(), input_ids))
            .map_err(|e| EmbeddingError::Inference(format!("input_ids tensor: {e}")))?;

        let attention_mask_tensor = Tensor::from_array((shape, attention_mask))
            .map_err(|e| EmbeddingError::Inference(format!("attention_mask tensor: {e}")))?;

        let output_name = self.session.outputs()[0].name().to_string();
        let input_names: Vec<String> = self
            .session
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();

        let outputs = if input_names.len() >= 2 {
            self.session.run(ort::inputs! {
                input_names[0].as_str() => input_ids_tensor,
                input_names[1].as_str() => attention_mask_tensor,
            })
        } else {
            self.session.run(ort::inputs![input_ids_tensor])
        }
        .map_err(|e| EmbeddingError::Inference(format!("run: {e}")))?;

        let output = outputs
            .get(&output_name)
            .ok_or_else(|| EmbeddingError::Inference("missing output tensor".into()))?;

        let (shape, data) = output
            .try_extract_tensor::<f32>()
            .map_err(|e| EmbeddingError::Inference(format!("extract: {e}")))?;

        if shape.len() != 3 {
            return Err(EmbeddingError::DimensionMismatch {
                expected: 3,
                got: shape.len(),
            });
        }

        let hidden_dim = shape[2] as usize;
        if hidden_dim != NEURAL_DIM {
            return Err(EmbeddingError::DimensionMismatch {
                expected: NEURAL_DIM,
                got: hidden_dim,
            });
        }

        let mut result = Vec::with_capacity(NEURAL_DIM);
        for (i, &v) in data.iter().take(NEURAL_DIM).enumerate() {
            if !v.is_finite() {
                return Err(EmbeddingError::NonFiniteOutput { index: i });
            }
            result.push(v as f64);
        }

        let norm: f64 = result.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in result.iter_mut() {
                *v /= norm;
            }
        }

        debug!(
            "Neural embedding computed: {} tokens -> {} dims",
            token_count, NEURAL_DIM
        );
        Ok(result)
    }

    pub fn output_dim() -> usize {
        NEURAL_DIM
    }
}

fn validate_onnx_header(path: &Path) -> Result<(), EmbeddingError> {
    let bytes = std::fs::read(path).map_err(|e| EmbeddingError::NotLoaded(format!("read: {e}")))?;

    if bytes.len() < 4 {
        return Err(EmbeddingError::NotLoaded("file too small".into()));
    }

    if bytes[0] != 0x08 {
        return Err(EmbeddingError::NotLoaded(format!(
            "invalid ONNX header: {:02X} (expected protobuf format starting with 08)",
            bytes[0]
        )));
    }

    Ok(())
}

pub struct EmbeddingEngineHandle {
    inner: Option<Arc<RwLock<EmbeddingEngine>>>,
    cache: RwLock<LruCache<String, Vec<f64>>>,
}

impl EmbeddingEngineHandle {
    pub fn disabled() -> Self {
        Self {
            inner: None,
            cache: RwLock::new(LruCache::new(CACHE_CAPACITY)),
        }
    }

    pub fn try_load(model_dir: &Path) -> Self {
        match EmbeddingEngine::new(model_dir) {
            Ok(engine) => {
                info!("Neural embedding engine loaded successfully");
                Self {
                    inner: Some(Arc::new(RwLock::new(engine))),
                    cache: RwLock::new(LruCache::new(CACHE_CAPACITY)),
                }
            }
            Err(e) => {
                warn!("Neural embedding engine unavailable: {e}");
                warn!("Falling back to hand-crafted 64-dim features only");
                Self {
                    inner: None,
                    cache: RwLock::new(LruCache::new(CACHE_CAPACITY)),
                }
            }
        }
    }

    pub fn embed(&self, text: &str) -> Option<Vec<f64>> {
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(cached) = cache.get(&text.to_string()) {
                return Some(cached.clone());
            }
        }
        match &self.inner {
            Some(engine_arc) => {
                let mut engine = engine_arc.write().unwrap();
                match engine.embed(text) {
                    Ok(v) => {
                        self.cache
                            .write()
                            .unwrap()
                            .insert(text.to_string(), v.clone());
                        Some(v)
                    }
                    Err(e) => {
                        warn!("Neural embedding failed: {e}");
                        None
                    }
                }
            }
            None => None,
        }
    }

    pub fn is_available(&self) -> bool {
        self.inner.is_some()
    }

    pub fn output_dim() -> usize {
        NEURAL_DIM
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_onnx_header_accepts_valid() {
        let valid_header: Vec<u8> = vec![0x08, 0x0A, 0x00, 0x00];
        let path = std::env::temp_dir().join("test_valid.onnx");
        std::fs::write(&path, &valid_header).unwrap();
        assert!(validate_onnx_header(&path).is_ok());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn validate_onnx_header_rejects_invalid() {
        let invalid: Vec<u8> = vec![0x3C, 0x68, 0x74, 0x6D];
        let path = std::env::temp_dir().join("test_invalid.onnx");
        std::fs::write(&path, &invalid).unwrap();
        assert!(validate_onnx_header(&path).is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn handle_disabled_returns_none() {
        let handle = EmbeddingEngineHandle::disabled();
        assert!(!handle.is_available());
        assert!(handle.embed("test").is_none());
    }

    #[test]
    fn neural_dim_is_384() {
        assert_eq!(EmbeddingEngine::output_dim(), 384);
    }

    #[test]
    fn byte_to_char_mapping_covers_all_bytes() {
        let table = build_byte_to_char();
        for b in 0u8..=255 {
            let c = table[b as usize];
            assert_ne!(c, '\0', "byte {b} should map to a real character");
        }
        assert_eq!(table[32], '\u{0120}', "space should map to Ġ");
        assert_eq!(table[b'A' as usize], 'A');
        assert_eq!(table[b'!' as usize], '!');
    }

    #[test]
    fn bpe_encode_basic() {
        let model_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("models")
            .join("granite-embedding-small");

        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            eprintln!("Skipping: tokenizer.json not found");
            return;
        }

        let tok = BpeTokenizer::from_file(&tokenizer_path).expect("load tokenizer");

        assert_eq!(tok.cls_id, 50281);
        assert_eq!(tok.sep_id, 50282);
        assert_eq!(tok.unk_id, 50280);
        assert_eq!(tok.pad_id, 50283);

        let (ids, mask) = tok.encode("Hello world", 64);
        assert_eq!(ids.len(), 64);
        assert_eq!(mask.len(), 64);

        assert_eq!(ids[0], 50281, "first token should be [CLS]");

        let non_pad: Vec<i64> = mask.iter().filter(|&&m| m == 1).copied().collect();
        assert!(non_pad.len() > 3, "should have CLS + tokens + SEP");

        let last_real = non_pad.len();
        assert_eq!(ids[last_real - 1], 50282, "last real token should be [SEP]");

        assert!(
            mask[last_real..].iter().all(|&m| m == 0),
            "padding mask should be 0"
        );
        assert!(
            ids[last_real..].iter().all(|&id| id == 50283),
            "padding ids should be [PAD]"
        );
    }

    #[test]
    fn pretokenize_basic() {
        let chunks = gpt2_pretokenize("Hello world");
        assert_eq!(chunks, vec!["Hello", " world"]);

        let chunks2 = gpt2_pretokenize("It's a test.");
        assert!(chunks2.contains(&"It"));
        assert!(chunks2.iter().any(|c| c.starts_with(' ')));
    }

    #[test]
    #[ignore]
    fn load_model_and_embed() {
        let model_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("models")
            .join("granite-embedding-small");

        if !model_dir.join("model_quantized.onnx").exists() {
            eprintln!("Skipping: model not found at {:?}", model_dir);
            return;
        }

        let engine_result = EmbeddingEngine::new(&model_dir);
        match &engine_result {
            Ok(e) => eprintln!("Engine loaded: {:?}", e),
            Err(e) => eprintln!("Engine failed: {e}"),
        }
        let engine = engine_result.expect("Engine should load");
        let handle = EmbeddingEngineHandle {
            inner: Some(Arc::new(RwLock::new(engine))),
            cache: RwLock::new(LruCache::new(CACHE_CAPACITY)),
        };

        let result = handle.embed("The weather is lovely today.");
        assert!(result.is_some(), "Embedding should succeed");
        let vec = result.unwrap();
        assert_eq!(vec.len(), NEURAL_DIM);

        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Embedding should be L2-normalized, got norm = {norm}"
        );

        let result2 = handle.embed("It's so sunny outside!");
        assert!(result2.is_some());
        let vec2 = result2.unwrap();

        let dot: f64 = vec.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        assert!(
            dot > 0.7,
            "Similar sentences should have high cosine similarity, got {dot}"
        );

        let result3 = handle.embed("He drove to the stadium.");
        assert!(result3.is_some());
        let vec3 = result3.unwrap();

        let dot2: f64 = vec.iter().zip(vec3.iter()).map(|(a, b)| a * b).sum();
        assert!(
            dot > dot2,
            "Similar pair ({dot}) should score higher than dissimilar pair ({dot2})"
        );
    }
}
