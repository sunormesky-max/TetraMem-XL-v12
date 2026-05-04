// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
// NLP utilities: TF-IDF text-to-embedding, synonym buckets, contradiction detection

use serde_json::json;
use std::collections::HashMap;

use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

const TFIDF_DIM: usize = 128;

const STOP_WORDS: &[&str] = &[
    "the",
    "a",
    "an",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "have",
    "has",
    "had",
    "do",
    "does",
    "did",
    "will",
    "would",
    "could",
    "should",
    "may",
    "might",
    "shall",
    "can",
    "need",
    "must",
    "ought",
    "to",
    "of",
    "in",
    "for",
    "on",
    "with",
    "at",
    "by",
    "from",
    "as",
    "into",
    "through",
    "during",
    "before",
    "after",
    "above",
    "below",
    "between",
    "out",
    "off",
    "over",
    "under",
    "again",
    "further",
    "then",
    "once",
    "and",
    "but",
    "or",
    "nor",
    "not",
    "so",
    "yet",
    "both",
    "either",
    "neither",
    "each",
    "every",
    "all",
    "any",
    "few",
    "more",
    "most",
    "other",
    "some",
    "such",
    "no",
    "only",
    "own",
    "same",
    "than",
    "too",
    "very",
    "just",
    "because",
    "if",
    "when",
    "where",
    "how",
    "what",
    "which",
    "who",
    "whom",
    "this",
    "that",
    "these",
    "those",
    "it",
    "its",
    "he",
    "she",
    "they",
    "them",
    "we",
    "you",
    "me",
    "my",
    "your",
    "his",
    "her",
    "our",
    "their",
    "de",
    "la",
    "le",
    "les",
    "un",
    "une",
    "des",
    "du",
    "et",
    "en",
    "est",
    "que",
    "qui",
    "dans",
    "pour",
    "sur",
    "pas",
    "avec",
    "plus",
    "的",
    "了",
    "在",
    "是",
    "我",
    "有",
    "和",
    "就",
    "不",
    "人",
    "都",
    "一",
    "一个",
    "上",
    "也",
    "很",
    "到",
    "说",
    "要",
    "去",
    "你",
    "会",
    "着",
    "没有",
    "看",
    "好",
    "自己",
    "这",
    "他",
    "她",
    "它",
    "们",
    "那",
    "些",
    "什么",
    "怎么",
    "如何",
    "为什么",
];

fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut tokens = Vec::new();

    for word in lower.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() || word.len() < 2 {
            continue;
        }
        if STOP_WORDS.contains(&word) {
            continue;
        }
        tokens.push(word.to_string());
        for sw in extract_subwords(word) {
            tokens.push(sw);
        }
    }

    let lower_bytes = lower.as_bytes();
    for i in 0..lower_bytes.len().saturating_sub(2) {
        let end = (i + 3).min(lower_bytes.len());
        let trigram = &lower_bytes[i..end];
        let mut h: u64 = 5381;
        for b in trigram {
            h = h.wrapping_mul(37).wrapping_add(*b as u64);
        }
        tokens.push(format!("_t{}", h % 10000));
    }

    let chars: Vec<char> = lower.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        let bigram = format!("{}{}", chars[i], chars[i + 1]);
        let mut h: u64 = 5381;
        for b in bigram.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        tokens.push(format!("_b{}", h % 10000));
    }

    tokens
}

fn token_hash(token: &str) -> usize {
    let mut h: u64 = 5381;
    for b in token.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    (h as usize) % TFIDF_DIM
}

#[derive(Debug, Clone)]
pub struct TfIdfIndex {
    doc_freq: HashMap<String, usize>,
    doc_count: usize,
}

impl TfIdfIndex {
    pub fn new() -> Self {
        Self {
            doc_freq: HashMap::new(),
            doc_count: 0,
        }
    }

    pub fn add_document(&mut self, text: &str) {
        self.doc_count += 1;
        let tokens = tokenize(text);
        let mut seen = std::collections::HashSet::new();
        for token in &tokens {
            if seen.insert(token.clone()) {
                *self.doc_freq.entry(token.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn remove_document(&mut self, text: &str) {
        if self.doc_count == 0 {
            return;
        }
        self.doc_count = self.doc_count.saturating_sub(1);
        let tokens = tokenize(text);
        let mut seen = std::collections::HashSet::new();
        for token in &tokens {
            if seen.insert(token.clone()) {
                if let Some(count) = self.doc_freq.get_mut(token) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.doc_freq.remove(token);
                    }
                }
            }
        }
    }

    pub fn compute_tfidf(&self, text: &str) -> Vec<f64> {
        let tokens = tokenize(text);
        if tokens.is_empty() || self.doc_count == 0 {
            return vec![0.0; TFIDF_DIM];
        }

        let mut tf: HashMap<String, f64> = HashMap::new();
        let total = tokens.len() as f64;
        for token in &tokens {
            *tf.entry(token.clone()).or_insert(0.0) += 1.0;
        }
        for v in tf.values_mut() {
            *v /= total;
        }

        let mut vec = vec![0.0f64; TFIDF_DIM];

        for (token, &term_freq) in &tf {
            let df = *self.doc_freq.get(token).unwrap_or(&1).max(&1) as f64;
            let idf = ((self.doc_count as f64 + 1.0) / (df + 1.0)).ln() + 1.0;
            let tfidf = term_freq * idf;

            let slot = token_hash(token);
            vec[slot] += tfidf;

            let mut h: u64 = 5381;
            for b in token.bytes() {
                h = h.wrapping_mul(37).wrapping_add(b as u64);
            }
            vec[(h as usize) % TFIDF_DIM] += tfidf * 0.5;
            vec[(h.wrapping_mul(53) as usize) % TFIDF_DIM] += tfidf * 0.3;

            if let Some(bucket) = synonym_bucket(token) {
                let b = bucket as usize;
                vec[b % TFIDF_DIM] += tfidf * 2.0;
                vec[(b * 7 + 3) % TFIDF_DIM] += tfidf * 1.5;
                vec[(b * 13 + 7) % TFIDF_DIM] += tfidf * 1.0;
            }
        }

        vec[0] += tokens.len() as f64 * 0.01;
        vec[1] += text.len() as f64 * 0.001;

        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        vec
    }

    pub fn doc_count(&self) -> usize {
        self.doc_count
    }

    pub fn vocab_size(&self) -> usize {
        self.doc_freq.len()
    }
}

impl Default for TfIdfIndex {
    fn default() -> Self {
        Self::new()
    }
}

pub fn text_to_tfidf_embedding(text: &str, index: &TfIdfIndex) -> Vec<f64> {
    index.compute_tfidf(text)
}

pub fn text_to_embedding(text: &str, importance: f64) -> Vec<f64> {
    let dim = TFIDF_DIM;
    let mut vec = vec![0.0f64; dim];

    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && w.len() > 1)
        .collect();

    for word in &words {
        if STOP_WORDS.contains(word) {
            continue;
        }

        let subwords = extract_subwords(word);
        let synonym_hash = synonym_bucket(word);

        for sw in &subwords {
            let mut h: u64 = 5381;
            for b in sw.as_bytes() {
                h = h.wrapping_mul(33).wrapping_add(*b as u64);
            }
            let s1 = (h as usize) % dim;
            let s2 = (h.wrapping_mul(37) as usize) % dim;
            let s3 = (h.wrapping_mul(53) as usize) % dim;
            let s4 = (h.wrapping_mul(59) as usize) % dim;
            vec[s1] += 1.0;
            vec[s2] += 0.8;
            vec[s3] += 0.5;
            vec[s4] += 0.3;
        }

        if let Some(bucket) = synonym_hash {
            let b = bucket as usize;
            vec[b % dim] += 2.0;
            vec[(b * 7 + 3) % dim] += 1.5;
            vec[(b * 13 + 7) % dim] += 1.0;
        }
    }

    let lower_bytes = lower.as_bytes();
    for i in 0..lower_bytes.len().saturating_sub(2) {
        let end = (i + 3).min(lower_bytes.len());
        let trigram = &lower_bytes[i..end];
        let mut h: u64 = 5381;
        for b in trigram {
            h = h.wrapping_mul(37).wrapping_add(*b as u64);
        }
        vec[(h as usize) % dim] += 0.4;
    }

    let chars: Vec<char> = lower.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        let bigram = format!("{}{}", chars[i], chars[i + 1]);
        let mut h: u64 = 5381;
        for b in bigram.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        vec[(h.wrapping_mul(41) as usize) % dim] += 0.3;
    }

    vec[0] = words.len() as f64 * 0.1;
    vec[1] = lower.len() as f64 * 0.01;
    vec[2] = importance;

    let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm > 1e-10 {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}

pub fn embedding_dim() -> usize {
    TFIDF_DIM
}

pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let min_len = a.len().min(b.len());
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for i in 0..min_len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-10 {
        return 0.0;
    }
    (dot / denom).clamp(0.0, 1.0)
}

pub fn extract_subwords(word: &str) -> Vec<String> {
    let mut subs = Vec::new();
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();

    if n >= 2 {
        for start in 0..n.saturating_sub(1) {
            let end = (start + 3).min(n);
            if end - start >= 2 {
                subs.push(chars[start..end].iter().collect());
            }
        }
        subs.push(format!("<{}>", word));
    }

    subs.push(word.to_string());

    subs
}

pub fn synonym_bucket(word: &str) -> Option<u64> {
    let buckets: &[&[&str]] = &[
        &[
            "prefer",
            "like",
            "love",
            "enjoy",
            "favor",
            "fancy",
            "adore",
            "appreciate",
        ],
        &["dislike", "hate", "detest", "loathe", "abhor", "despise"],
        &[
            "good",
            "great",
            "excellent",
            "fine",
            "nice",
            "wonderful",
            "amazing",
            "awesome",
            "fantastic",
        ],
        &[
            "bad", "poor", "terrible", "awful", "horrible", "dreadful", "worst",
        ],
        &[
            "big", "large", "huge", "vast", "enormous", "massive", "giant", "immense",
        ],
        &[
            "small", "tiny", "little", "mini", "micro", "compact", "minor",
        ],
        &[
            "fast", "quick", "rapid", "swift", "speedy", "prompt", "hasty",
        ],
        &["slow", "sluggish", "gradual", "steady", "leisurely"],
        &[
            "happy",
            "glad",
            "joyful",
            "cheerful",
            "pleased",
            "delighted",
            "content",
        ],
        &[
            "sad",
            "unhappy",
            "depressed",
            "miserable",
            "gloomy",
            "sorrowful",
        ],
        &[
            "important",
            "significant",
            "crucial",
            "vital",
            "essential",
            "critical",
            "key",
        ],
        &[
            "think", "believe", "consider", "suppose", "assume", "guess", "reckon",
        ],
        &[
            "know",
            "understand",
            "comprehend",
            "grasp",
            "realize",
            "recognize",
        ],
        &["use", "utilize", "employ", "apply", "operate", "leverage"],
        &[
            "make",
            "create",
            "build",
            "construct",
            "produce",
            "generate",
            "develop",
        ],
        &["help", "assist", "support", "aid", "facilitate", "enable"],
        &["need", "require", "demand", "want", "desire", "wish"],
        &[
            "change",
            "modify",
            "alter",
            "adjust",
            "transform",
            "update",
            "convert",
        ],
        &["start", "begin", "launch", "initiate", "commence", "open"],
        &[
            "stop",
            "end",
            "finish",
            "complete",
            "conclude",
            "terminate",
            "halt",
        ],
        &["work", "function", "operate", "perform", "run", "execute"],
        &[
            "system",
            "platform",
            "framework",
            "engine",
            "architecture",
            "infrastructure",
        ],
        &[
            "data",
            "information",
            "knowledge",
            "facts",
            "details",
            "records",
        ],
        &["user", "client", "customer", "person", "human", "people"],
        &["dark", "night", "shadow", "dim", "black", "obscure"],
        &["light", "bright", "luminous", "clear", "vivid", "radiant"],
        &[
            "mode",
            "setting",
            "option",
            "preference",
            "configuration",
            "theme",
        ],
        &["memory", "recall", "remember", "store", "retain", "record"],
        &["learn", "study", "acquire", "absorb", "train", "educate"],
        &[
            "search", "find", "look", "seek", "discover", "explore", "query",
        ],
        &[
            "show",
            "display",
            "present",
            "reveal",
            "exhibit",
            "demonstrate",
        ],
        &["hide", "conceal", "mask", "cover", "obscure", "cloak"],
        &[
            "connect",
            "link",
            "join",
            "associate",
            "bind",
            "attach",
            "relate",
        ],
        &[
            "error", "bug", "fault", "defect", "issue", "problem", "mistake",
        ],
        &[
            "fix", "repair", "correct", "resolve", "patch", "solve", "debug",
        ],
        &[
            "new", "fresh", "recent", "latest", "modern", "current", "novel",
        ],
        &[
            "old", "ancient", "outdated", "legacy", "obsolete", "vintage",
        ],
        &[
            "simple",
            "easy",
            "basic",
            "straightforward",
            "plain",
            "elementary",
        ],
        &[
            "complex",
            "complicated",
            "intricate",
            "elaborate",
            "sophisticated",
        ],
        &[
            "safe",
            "secure",
            "protected",
            "guarded",
            "reliable",
            "stable",
        ],
        &[
            "danger",
            "risk",
            "threat",
            "hazard",
            "peril",
            "vulnerability",
        ],
    ];

    for (i, bucket) in buckets.iter().enumerate() {
        if bucket.contains(&word) {
            return Some(i as u64 * 17 + 3);
        }
        if bucket.iter().any(|&w| {
            word.len() >= 4 && w.len() >= 4 && (word.starts_with(w) || w.starts_with(word))
        }) {
            return Some(i as u64 * 17 + 3);
        }
    }

    None
}

pub fn detect_contradictions(
    new_content: &str,
    new_anchor: &Coord7D,
    memories: &[MemoryAtom],
    _universe: &DarkUniverse,
    hebbian: &HebbianMemory,
) -> Vec<serde_json::Value> {
    let mut contradictions = Vec::new();
    let new_phys = new_anchor.physical();

    let contradict_pairs: &[&[&str]] = &[
        &["should", "should not", "must not", "never"],
        &["always", "never", "sometimes", "rarely"],
        &["good", "bad", "terrible", "awful"],
        &["like", "hate", "dislike", "loathe"],
        &["agree", "disagree", "oppose", "reject"],
        &["true", "false", "wrong", "incorrect"],
        &["yes", "no"],
        &["possible", "impossible"],
        &["easy", "hard", "difficult", "complex"],
        &["safe", "dangerous", "risky", "unsafe"],
    ];

    let new_lower = new_content.to_lowercase();
    let mut new_sentiment_group: Option<usize> = None;
    'outer: for (gi, group) in contradict_pairs.iter().enumerate() {
        for word in *group {
            if new_lower.contains(word) {
                new_sentiment_group = Some(gi);
                break 'outer;
            }
        }
    }

    if new_sentiment_group.is_none() {
        return contradictions;
    }

    let sg = new_sentiment_group.unwrap();
    let group = contradict_pairs[sg];

    let neighbors = hebbian.get_neighbors(new_anchor);
    let mut candidates: Vec<&MemoryAtom> = Vec::new();

    for mem in memories.iter() {
        let mp = mem.anchor().physical();
        let d =
            (new_phys[0] - mp[0]).abs() + (new_phys[1] - mp[1]).abs() + (new_phys[2] - mp[2]).abs();
        if d < 200 && d > 0 {
            candidates.push(mem);
        }
    }

    for (coord, _weight) in &neighbors {
        if let Some(mem) = memories.iter().find(|m| m.anchor() == coord) {
            if !candidates.iter().any(|c| c.anchor() == mem.anchor()) {
                candidates.push(mem);
            }
        }
    }

    for mem in candidates.iter().take(10) {
        let desc = match mem.description() {
            Some(d) => d.to_lowercase(),
            None => continue,
        };

        let mut has_same = false;
        let mut has_opposite = false;
        for (wi, word) in group.iter().enumerate() {
            if new_lower.contains(word) {
                for (wj, other_word) in group.iter().enumerate() {
                    if desc.contains(other_word) {
                        if wi == wj {
                            has_same = true;
                        } else if wi / 2 != wj / 2 {
                            has_opposite = true;
                        }
                    }
                }
            }
        }

        if has_opposite && !has_same {
            let edge_w = hebbian.get_bias(new_anchor, mem.anchor());
            contradictions.push(json!({
                "conflict_with": desc,
                "anchor": format!("{}", mem.anchor()),
                "edge_weight": edge_w,
                "confidence": if edge_w > 0.5 { "high" } else { "medium" },
            }));
        }
    }

    contradictions
}

pub fn text_to_anchor(text: &str) -> Coord7D {
    let lower = text.to_lowercase();
    let mut h1: u64 = 5381;
    let mut h2: u64 = 5271;
    let mut h3: u64 = 65537;
    for b in lower.as_bytes() {
        h1 = h1.wrapping_mul(33).wrapping_add(*b as u64);
        h2 = h2.wrapping_mul(37).wrapping_add(*b as u64);
        h3 = h3.wrapping_mul(41).wrapping_add(*b as u64);
    }
    let ax = (h1 as i32).abs() % 10000;
    let ay = (h2 as i32).abs() % 10000;
    let az = (h3 as i32).abs() % 10000;
    Coord7D::new_even([ax, ay, az, 0, 0, 0, 0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tfidf_basic() {
        let mut index = TfIdfIndex::new();
        index.add_document("machine learning is great");
        index.add_document("deep learning neural networks");
        index.add_document("rust programming language");

        let vec = index.compute_tfidf("machine learning");
        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-10,
            "should be normalized, got {}",
            norm
        );
    }

    #[test]
    fn tfidf_discriminates_topics() {
        let mut index = TfIdfIndex::new();
        index.add_document("the cat sat on the mat");
        index.add_document("dogs are loyal animals");
        index.add_document("programming in rust is fun");

        let cat_vec = index.compute_tfidf("cat mat");
        let rust_vec = index.compute_tfidf("rust programming");
        let sim = cosine_similarity(&cat_vec, &rust_vec);
        assert!(
            sim < 0.9,
            "unrelated topics should have low similarity, got {}",
            sim
        );
    }

    #[test]
    fn tfidf_matches_similar_docs() {
        let mut index = TfIdfIndex::new();
        index.add_document("machine learning algorithms");
        index.add_document("deep learning networks");
        index.add_document("cooking recipes for dinner");

        let vec1 = index.compute_tfidf("machine learning");
        let vec2 = index.compute_tfidf("learning algorithms");
        let sim = cosine_similarity(&vec1, &vec2);
        assert!(
            sim > 0.5,
            "similar docs should have high similarity, got {}",
            sim
        );
    }

    #[test]
    fn chinese_tokenization() {
        let mut index = TfIdfIndex::new();
        index.add_document("这是一条测试记忆");
        index.add_document("这是另一条数据");

        let vec = index.compute_tfidf("测试记忆");
        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(norm > 0.0, "Chinese text should produce non-zero embedding");
    }

    #[test]
    fn synonym_bucket_works() {
        assert!(synonym_bucket("good").is_some());
        assert!(synonym_bucket("xyzzy").is_none());
    }

    #[test]
    fn text_to_anchor_deterministic() {
        let a = text_to_anchor("hello world");
        let b = text_to_anchor("hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn text_to_embedding_dim() {
        let vec = text_to_embedding("test", 0.5);
        assert_eq!(vec.len(), TFIDF_DIM);
    }

    #[test]
    fn cosine_sim_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn remove_document_updates_counts() {
        let mut index = TfIdfIndex::new();
        index.add_document("hello world test");
        assert_eq!(index.doc_count(), 1);
        index.remove_document("hello world test");
        assert_eq!(index.doc_count(), 0);
    }
}
