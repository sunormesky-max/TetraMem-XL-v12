// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
//
// Semantic Understanding Layers (S2-S5):
//   S2: Semantic Embedding — 64-dim vector embedding + KNN search + TF-IDF text search
//   S3: Knowledge Graph — typed relations between memories
//   S4: Concept Abstraction — dream-driven prototype extraction with centroid updates
//   S5: Semantic Query — unified query language

use crate::universe::memory::nlp::{self, TfIdfIndex};
use crate::universe::memory::MemoryAtom;
use std::collections::{HashMap, HashSet};

// ═══════════════════════════════════════════════════════════════════════
// S2: Semantic Embedding Layer (64-dim)
// ═══════════════════════════════════════════════════════════════════════

const EMBED_DIM: usize = 64;
const STAT_DIM: usize = 20;
const HIST_DIM: usize = 16;
const FREQ_DIM: usize = 16;
const META_DIM: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticEmbedding {
    vector: [f64; EMBED_DIM],
}

impl SemanticEmbedding {
    pub fn zero() -> Self {
        Self {
            vector: [0.0; EMBED_DIM],
        }
    }

    pub fn from_data(data: &[f64]) -> Self {
        let mut vec = [0.0f64; EMBED_DIM];
        let n = data.len().max(1) as f64;

        let mut sum = 0.0f64;
        let mut sq_sum = 0.0f64;
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        for &v in data {
            sum += v;
            sq_sum += v * v;
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }
        let mean = sum / n;
        let variance = (sq_sum / n) - (mean * mean);
        let std_dev = variance.sqrt().max(1e-10);
        let range = max_val - min_val;

        vec[0] = mean;
        vec[1] = std_dev;
        vec[2] = min_val;
        vec[3] = max_val;
        vec[4] = range;
        vec[5] = n;

        if data.len() > 1 {
            let mut autocorr = 0.0f64;
            for i in 0..data.len() - 1 {
                autocorr += (data[i] - mean) * (data[i + 1] - mean);
            }
            autocorr /= (data.len() - 1) as f64 * variance.max(1e-10);
            vec[6] = autocorr;
        }

        let nbins = 10usize;
        let mut bins = vec![0usize; nbins];
        let bin_range = range.max(1e-10);
        for &v in data {
            let idx = (((v - min_val) / bin_range * (nbins as f64 - 1.0)).round() as usize)
                .min(nbins - 1);
            bins[idx] += 1;
        }
        let mut entropy = 0.0f64;
        for &count in &bins {
            if count > 0 {
                let p = count as f64 / n;
                entropy -= p * p.log2();
            }
        }
        vec[7] = entropy;

        let mut skewness = 0.0f64;
        let mut kurtosis = 0.0f64;
        for &v in data {
            let d = (v - mean) / std_dev;
            skewness += d * d * d;
            kurtosis += d * d * d * d;
        }
        vec[8] = skewness / n;
        vec[9] = kurtosis / n;

        if data.len() >= 4 {
            let quarter = data.len() / 4;
            let q1: f64 = data[..quarter].iter().sum::<f64>() / quarter as f64;
            let q4_start = data.len() - quarter;
            let q4: f64 = data[q4_start..].iter().sum::<f64>() / quarter as f64;
            vec[10] = q4 - q1;

            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            vec[11] = sorted[data.len() / 2];
        }

        if data.len() > 2 {
            let mut diffs: Vec<f64> = Vec::with_capacity(data.len() - 1);
            for i in 1..data.len() {
                diffs.push((data[i] - data[i - 1]).abs());
            }
            vec[12] = diffs.iter().sum::<f64>() / diffs.len() as f64;
        }

        vec[13] = mean * mean;
        vec[14] = mean * std_dev;
        vec[15] = range * entropy;

        if data.len() >= 10 {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p25 = sorted[data.len() / 4];
            let p75 = sorted[data.len() * 3 / 4];
            vec[16] = p75 - p25;
            vec[17] = sorted[data.len() / 10];
            vec[18] = sorted[data.len() * 9 / 10];
            vec[19] = (sorted[data.len() * 9 / 10] - sorted[data.len() / 10]) / (std_dev + 1e-10);
        }

        let hist_bins = HIST_DIM;
        let mut hist = vec![0usize; hist_bins];
        let hist_range = range.max(1e-10);
        for &v in data {
            let idx = (((v - min_val) / hist_range * (hist_bins as f64 - 1.0)).round() as usize)
                .min(hist_bins - 1);
            hist[idx] += 1;
        }
        for (i, &count) in hist.iter().enumerate() {
            if i + STAT_DIM < EMBED_DIM {
                vec[i + STAT_DIM] = count as f64 / n;
            }
        }

        if data.len() >= 4 {
            let n = data.len() as f64;
            let two_pi_over_n = 2.0 * std::f64::consts::PI / n;
            let mut fft_mag = [0.0f64; FREQ_DIM];
            for (k, slot) in fft_mag
                .iter_mut()
                .enumerate()
                .take(FREQ_DIM.min(data.len() / 2))
            {
                let mut re = 0.0f64;
                let mut im = 0.0f64;
                let k_f64 = k as f64;
                for (t, &val) in data.iter().enumerate() {
                    let angle = two_pi_over_n * k_f64 * t as f64;
                    re += val * angle.cos();
                    im -= val * angle.sin();
                }
                *slot = (re * re + im * im).sqrt() / n;
            }
            for (i, &val) in fft_mag.iter().enumerate().take(FREQ_DIM) {
                let pos = STAT_DIM + HIST_DIM + i;
                if pos < EMBED_DIM {
                    vec[pos] = val;
                }
            }
        } else {
            let freq_base = STAT_DIM + HIST_DIM;
            if data.len() >= 2 {
                let diff0 = data[1] - data[0];
                vec[freq_base] = diff0.abs();
                vec[freq_base + 1] = diff0.signum();
            }
            if data.len() >= 3 {
                let d1 = data[1] - data[0];
                let d2 = data[2] - data[1];
                vec[freq_base + 2] = (d2 - d1).abs();
                vec[freq_base + 3] = d1 * d2;
            }
            let n_f64 = data.len() as f64;
            let gcd_approx = if data.len() >= 2 {
                let r = data[0].fract();
                if r.abs() < 1e-10 {
                    0.0
                } else {
                    r.abs().ln().abs()
                }
            } else {
                0.0
            };
            vec[freq_base + 4] = gcd_approx;
            vec[freq_base + 5] = n_f64.ln();
            let peak_to_peak = vec[4];
            vec[freq_base + 6] = peak_to_peak * vec[6];
            vec[freq_base + 7] = mean * n_f64;
            let ratio_first = if data.len() >= 2 && data[0].abs() > 1e-10 {
                data[1] / data[0]
            } else {
                0.0
            };
            vec[freq_base + 8] = ratio_first;
            vec[freq_base + 9] = ratio_first * vec[6];
            vec[freq_base + 10] = sum / (range + 1e-10);
            vec[freq_base + 11] = (n_f64 - 1.0).max(0.0);
        }

        let meta_base = EMBED_DIM - META_DIM;
        vec[meta_base] = 1.0f64.signum();
        vec[meta_base + 1] = if data.iter().all(|v| *v >= 0.0) {
            1.0
        } else if data.iter().all(|v| *v <= 0.0) {
            -1.0
        } else {
            0.0
        };
        vec[meta_base + 2] = (data.len() as f64).log2();
        vec[meta_base + 3] = if let Some(first) = data.first() {
            first.signum()
        } else {
            0.0
        };
        vec[meta_base + 4] = if let Some(last) = data.last() {
            last.signum()
        } else {
            0.0
        };
        let monotone = data.windows(2).all(|w| w[0] <= w[1]);
        let mono_decr = data.windows(2).all(|w| w[0] >= w[1]);
        vec[meta_base + 5] = if monotone {
            1.0
        } else if mono_decr {
            -1.0
        } else {
            0.0
        };
        let has_zero = data.iter().any(|v| v.abs() < 1e-10);
        vec[meta_base + 6] = if has_zero { 1.0 } else { 0.0 };
        let integer_ratio = data.iter().filter(|v| (v.fract()).abs() < 1e-10).count() as f64 / n;
        vec[meta_base + 7] = integer_ratio;
        let unique_count = {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut count = 1;
            for i in 1..sorted.len() {
                if (sorted[i] - sorted[i - 1]).abs() > 1e-10 {
                    count += 1;
                }
            }
            count as f64
        };
        vec[meta_base + 8] = unique_count / n;
        vec[meta_base + 9] = mean.signum() * std_dev;
        vec[meta_base + 10] = vec[7] * vec[8];
        vec[meta_base + 11] = vec[4] / (std_dev + 1e-10);

        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        Self { vector: vec }
    }

    pub fn from_annotation(atom: &MemoryAtom) -> Self {
        let mut vec = [0.0f64; EMBED_DIM];

        let text_parts: Vec<&str> = vec![
            atom.description().unwrap_or(""),
            atom.category().unwrap_or(""),
            atom.source().unwrap_or(""),
        ];
        let text = text_parts.join(" ");
        let text_emb = nlp::text_to_embedding(&text, atom.importance());

        for (i, &val) in text_emb
            .iter()
            .enumerate()
            .take(META_DIM.min(text_emb.len()))
        {
            let slot = EMBED_DIM - META_DIM + i;
            if slot < EMBED_DIM {
                vec[slot] = val;
            }
        }

        let mut cat_hash: u64 = 0;
        if let Some(c) = atom.category() {
            for b in c.bytes() {
                cat_hash = cat_hash.wrapping_mul(31).wrapping_add(b as u64);
            }
        }
        vec[0] = (cat_hash % 1000) as f64;

        vec[1] = atom.tags().len() as f64;

        let desc_len = atom.description().map(|d| d.len() as f64).unwrap_or(0.0);
        vec[2] = desc_len;

        let mut src_hash: u64 = 0;
        if let Some(s) = atom.source() {
            for b in s.bytes() {
                src_hash = src_hash.wrapping_mul(31).wrapping_add(b as u64);
            }
        }
        vec[3] = (src_hash % 1000) as f64;

        vec[4] = atom.importance();
        vec[5] = atom.data_dim() as f64;

        let tags_text = atom.tags().join(" ");
        let tag_emb = nlp::text_to_embedding(&tags_text, 0.5);
        let tag_copy_len = 6.min(tag_emb.len());
        vec[6..(6 + tag_copy_len)].copy_from_slice(&tag_emb[..tag_copy_len]);

        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        Self { vector: vec }
    }

    pub fn from_data_and_annotation(data: &[f64], atom: &MemoryAtom) -> Self {
        let data_emb = Self::from_data(data);
        let ann_emb = Self::from_annotation(atom);

        let mut combined = [0.0f64; EMBED_DIM];
        for (i, slot) in combined.iter_mut().enumerate().take(EMBED_DIM) {
            *slot = 0.6 * data_emb.vector[i] + 0.4 * ann_emb.vector[i];
        }

        let norm: f64 = combined.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut combined {
                *v /= norm;
            }
        }

        Self { vector: combined }
    }

    pub fn vector(&self) -> &[f64; EMBED_DIM] {
        &self.vector
    }

    pub fn cosine_similarity(&self, other: &Self) -> f64 {
        let mut dot = 0.0f64;
        let mut norm_a = 0.0f64;
        let mut norm_b = 0.0f64;
        for i in 0..EMBED_DIM {
            dot += self.vector[i] * other.vector[i];
            norm_a += self.vector[i] * self.vector[i];
            norm_b += other.vector[i] * other.vector[i];
        }
        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom < 1e-10 {
            return 0.0;
        }
        (dot / denom).clamp(0.0, 1.0)
    }

    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        let mut sum = 0.0f64;
        for i in 0..EMBED_DIM {
            let d = self.vector[i] - other.vector[i];
            sum += d * d;
        }
        sum.sqrt()
    }

    pub fn update_centroid(&self, centroid: &Self, n_members: usize) -> Self {
        let mut vec = [0.0f64; EMBED_DIM];
        let old_weight = n_members as f64;
        let new_weight = 1.0;
        let total = old_weight + new_weight;
        for (i, slot) in vec.iter_mut().enumerate().take(EMBED_DIM) {
            *slot = (centroid.vector[i] * old_weight + self.vector[i] * new_weight) / total;
        }
        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        Self { vector: vec }
    }
}

#[derive(Debug, Clone)]
struct EmbeddingEntry {
    atom_key: AtomKey,
    embedding: SemanticEmbedding,
    category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomKey {
    pub vertices_basis: [[i32; 7]; 4],
    pub vertices_even: [bool; 4],
}

impl AtomKey {
    pub fn from_atom(atom: &MemoryAtom) -> Self {
        let mut basis = [[0i32; 7]; 4];
        let mut even = [false; 4];
        for (i, v) in atom.vertices().iter().enumerate() {
            basis[i] = v.basis();
            even[i] = v.is_even();
        }
        Self {
            vertices_basis: basis,
            vertices_even: even,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnnResult {
    pub atom_key: AtomKey,
    pub similarity: f64,
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct EmbeddingIndex {
    entries: Vec<EmbeddingEntry>,
    key_to_idx: HashMap<AtomKey, usize>,
}

impl EmbeddingIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            key_to_idx: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, key: AtomKey, embedding: SemanticEmbedding, category: Option<String>) {
        if let Some(&idx) = self.key_to_idx.get(&key) {
            self.entries[idx].embedding = embedding;
            self.entries[idx].category = category;
        } else {
            let idx = self.entries.len();
            self.key_to_idx.insert(key.clone(), idx);
            self.entries.push(EmbeddingEntry {
                atom_key: key,
                embedding,
                category,
            });
        }
    }

    pub fn remove(&mut self, key: &AtomKey) {
        if let Some(idx) = self.key_to_idx.remove(key) {
            self.entries.swap_remove(idx);
            if idx < self.entries.len() {
                let swapped_key = self.entries[idx].atom_key.clone();
                self.key_to_idx.insert(swapped_key, idx);
            }
        }
    }

    pub fn get_embedding(&self, key: &AtomKey) -> Option<&SemanticEmbedding> {
        self.key_to_idx
            .get(key)
            .map(|&idx| &self.entries[idx].embedding)
    }

    pub fn search_knn(&self, query: &SemanticEmbedding, k: usize) -> Vec<KnnResult> {
        let mut scored: Vec<KnnResult> = self
            .entries
            .iter()
            .map(|e| KnnResult {
                atom_key: e.atom_key.clone(),
                similarity: query.cosine_similarity(&e.embedding),
                distance: query.euclidean_distance(&e.embedding),
            })
            .collect();
        scored.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }

    pub fn search_knn_with_scores(
        &self,
        query: &SemanticEmbedding,
        k: usize,
    ) -> HashMap<AtomKey, f64> {
        let results = self.search_knn(query, k);
        let mut map = HashMap::with_capacity(results.len());
        for r in results {
            map.insert(r.atom_key, r.similarity);
        }
        map
    }

    pub fn search_by_category(&self, category: &str) -> Vec<(AtomKey, f64)> {
        self.entries
            .iter()
            .filter(|e| {
                e.category
                    .as_ref()
                    .map(|c| c.eq_ignore_ascii_case(category))
                    .unwrap_or(false)
            })
            .map(|e| (e.atom_key.clone(), 1.0))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for EmbeddingIndex {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// S3: Knowledge Graph Layer
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationType {
    IsA,
    PartOf,
    Causes,
    Contradicts,
    SimilarTo,
    RelatedTo,
    Precedes,
    DerivedFrom,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationType::IsA => "is_a",
            RelationType::PartOf => "part_of",
            RelationType::Causes => "causes",
            RelationType::Contradicts => "contradicts",
            RelationType::SimilarTo => "similar_to",
            RelationType::RelatedTo => "related_to",
            RelationType::Precedes => "precedes",
            RelationType::DerivedFrom => "derived_from",
        }
    }

    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s {
            "is_a" => Some(RelationType::IsA),
            "part_of" => Some(RelationType::PartOf),
            "causes" => Some(RelationType::Causes),
            "contradicts" => Some(RelationType::Contradicts),
            "similar_to" => Some(RelationType::SimilarTo),
            "related_to" => Some(RelationType::RelatedTo),
            "precedes" => Some(RelationType::Precedes),
            "derived_from" => Some(RelationType::DerivedFrom),
            _ => None,
        }
    }

    pub fn all() -> &'static [RelationType] {
        &[
            RelationType::IsA,
            RelationType::PartOf,
            RelationType::Causes,
            RelationType::Contradicts,
            RelationType::SimilarTo,
            RelationType::RelatedTo,
            RelationType::Precedes,
            RelationType::DerivedFrom,
        ]
    }

    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            RelationType::SimilarTo | RelationType::Contradicts | RelationType::RelatedTo
        )
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub from: AtomKey,
    pub to: AtomKey,
    pub rel_type: RelationType,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

impl Relation {
    pub fn new(from: AtomKey, to: AtomKey, rel_type: RelationType) -> Self {
        Self {
            from,
            to,
            rel_type,
            weight: 1.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    relations: Vec<Relation>,
    outgoing: HashMap<AtomKey, Vec<usize>>,
    incoming: HashMap<AtomKey, Vec<usize>>,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            relations: Vec::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
        }
    }

    pub fn add_relation(&mut self, relation: Relation) {
        let idx = self.relations.len();
        let from_key = relation.from.clone();
        let to_key = relation.to.clone();
        let rel_type = relation.rel_type;

        self.outgoing.entry(from_key.clone()).or_default().push(idx);
        self.incoming.entry(to_key.clone()).or_default().push(idx);
        self.relations.push(relation);

        if rel_type.is_symmetric() {
            let reverse = Relation {
                from: to_key,
                to: from_key,
                rel_type,
                weight: self.relations[idx].weight,
                metadata: self.relations[idx].metadata.clone(),
            };
            let ridx = self.relations.len();
            self.outgoing
                .entry(reverse.from.clone())
                .or_default()
                .push(ridx);
            self.incoming
                .entry(reverse.to.clone())
                .or_default()
                .push(ridx);
            self.relations.push(reverse);
        }
    }

    pub fn relations_from(&self, key: &AtomKey) -> Vec<&Relation> {
        self.outgoing
            .get(key)
            .map(|indices| indices.iter().map(|&i| &self.relations[i]).collect())
            .unwrap_or_default()
    }

    pub fn relations_to(&self, key: &AtomKey) -> Vec<&Relation> {
        self.incoming
            .get(key)
            .map(|indices| indices.iter().map(|&i| &self.relations[i]).collect())
            .unwrap_or_default()
    }

    pub fn relations_of_type(&self, rel_type: RelationType) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| r.rel_type == rel_type)
            .collect()
    }

    pub fn neighbors(
        &self,
        key: &AtomKey,
        max_depth: usize,
    ) -> Vec<(AtomKey, RelationType, usize)> {
        let mut visited: HashMap<AtomKey, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = vec![(key.clone(), 0usize)];

        while let Some((current, depth)) = queue.pop() {
            if depth > max_depth {
                continue;
            }
            if visited.contains_key(&current) {
                continue;
            }
            visited.insert(current.clone(), depth);

            for rel in self.relations_from(&current) {
                if !visited.contains_key(&rel.to) {
                    result.push((rel.to.clone(), rel.rel_type, depth + 1));
                    if depth < max_depth {
                        queue.push((rel.to.clone(), depth + 1));
                    }
                }
            }
            for rel in self.relations_to(&current) {
                if !visited.contains_key(&rel.from) {
                    result.push((rel.from.clone(), rel.rel_type, depth + 1));
                    if depth < max_depth {
                        queue.push((rel.from.clone(), depth + 1));
                    }
                }
            }
        }

        result
    }

    pub fn remove_relations_for(&mut self, key: &AtomKey) {
        let to_remove: Vec<usize> = self
            .relations
            .iter()
            .enumerate()
            .filter(|(_, r)| &r.from == key || &r.to == key)
            .map(|(i, _)| i)
            .collect();

        for idx in to_remove.into_iter().rev() {
            self.relations.remove(idx);
        }
        self.rebuild_index();
    }

    fn rebuild_index(&mut self) {
        self.outgoing.clear();
        self.incoming.clear();
        for (idx, rel) in self.relations.iter().enumerate() {
            self.outgoing.entry(rel.from.clone()).or_default().push(idx);
            self.incoming.entry(rel.to.clone()).or_default().push(idx);
        }
    }

    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.relations.is_empty()
    }

    pub fn infer_transitive_closure(&mut self, rel_type: RelationType, max_depth: usize) -> usize {
        let mut new_relations = Vec::new();
        let mut current_pairs: HashSet<(AtomKey, AtomKey)> = self
            .relations
            .iter()
            .filter(|r| r.rel_type == rel_type)
            .map(|r| (r.from.clone(), r.to.clone()))
            .collect();

        let mut all_pairs = current_pairs.clone();

        for _depth in 0..max_depth {
            let mut next_pairs = HashSet::new();
            for (a, b) in &current_pairs {
                for (c, d) in &current_pairs {
                    if b == c && !all_pairs.contains(&(a.clone(), d.clone())) {
                        next_pairs.insert((a.clone(), d.clone()));
                    }
                }
            }
            if next_pairs.is_empty() {
                break;
            }
            for (from, to) in &next_pairs {
                new_relations.push(
                    Relation::new(from.clone(), to.clone(), rel_type)
                        .with_weight(0.5)
                        .with_metadata("inferred", "transitive_closure")
                        .with_metadata("depth", format!("{}", _depth + 2)),
                );
            }
            all_pairs.extend(next_pairs.iter().cloned());
            current_pairs = next_pairs;
        }

        let count = new_relations.len();
        for rel in new_relations {
            self.add_relation(rel);
        }
        count
    }

    pub fn infer_isa_inheritance(&mut self) -> usize {
        let isa_chains = self.compute_isa_chains();
        let mut new_relations = Vec::new();

        for (leaf, ancestors) in &isa_chains {
            for ancestor in ancestors {
                let existing_partof = self.relations.iter().any(|r| {
                    &r.from == leaf && &r.to == ancestor && r.rel_type == RelationType::PartOf
                });
                if !existing_partof {
                    new_relations.push(
                        Relation::new(leaf.clone(), ancestor.clone(), RelationType::PartOf)
                            .with_weight(0.6)
                            .with_metadata("inferred", "isa_inheritance"),
                    );
                }

                for leaf_attr in self.relations_from(leaf) {
                    if leaf_attr.rel_type == RelationType::IsA {
                        continue;
                    }
                    if leaf_attr.rel_type.is_symmetric() {
                        continue;
                    }
                    let already = self.relations.iter().any(|r| {
                        &r.from == ancestor
                            && r.to == leaf_attr.to
                            && r.rel_type == leaf_attr.rel_type
                    });
                    if !already {
                        new_relations.push(
                            Relation::new(
                                ancestor.clone(),
                                leaf_attr.to.clone(),
                                leaf_attr.rel_type,
                            )
                            .with_weight(leaf_attr.weight * 0.5)
                            .with_metadata("inferred", "isa_inheritance")
                            .with_metadata("source", "property_promotion"),
                        );
                    }
                }
            }
        }

        let count = new_relations.len();
        for rel in new_relations {
            self.add_relation(rel);
        }
        count
    }

    fn compute_isa_chains(&self) -> HashMap<AtomKey, Vec<AtomKey>> {
        let isa_map: HashMap<AtomKey, AtomKey> = self
            .relations
            .iter()
            .filter(|r| r.rel_type == RelationType::IsA)
            .map(|r| (r.from.clone(), r.to.clone()))
            .collect();

        let mut result: HashMap<AtomKey, Vec<AtomKey>> = HashMap::new();
        for leaf in isa_map.keys() {
            let mut ancestors = Vec::new();
            let mut current = leaf.clone();
            let mut visited = HashSet::new();
            while let Some(parent) = isa_map.get(&current) {
                if visited.contains(parent) {
                    break;
                }
                ancestors.push(parent.clone());
                visited.insert(parent.clone());
                current = parent.clone();
            }
            if !ancestors.is_empty() {
                result.insert(leaf.clone(), ancestors);
            }
        }
        result
    }

    pub fn infer_causes_chains(&mut self, max_length: usize) -> usize {
        let mut new_relations = Vec::new();
        let mut chains: HashMap<AtomKey, Vec<(AtomKey, f64)>> = HashMap::new();

        for rel in &self.relations {
            if rel.rel_type == RelationType::Causes {
                chains
                    .entry(rel.from.clone())
                    .or_default()
                    .push((rel.to.clone(), rel.weight));
            }
        }

        for (start, nexts) in &chains {
            let mut frontier: Vec<(AtomKey, f64, usize)> =
                nexts.iter().map(|(k, w)| (k.clone(), *w, 1usize)).collect();
            let mut visited = HashSet::new();
            visited.insert(start.clone());

            while let Some((current, cum_weight, depth)) = frontier.pop() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current.clone());

                if depth >= max_length {
                    continue;
                }

                if let Some(following) = chains.get(&current) {
                    for (next, edge_w) in following {
                        let new_w = cum_weight * edge_w * 0.7;
                        if new_w < 0.1 {
                            continue;
                        }
                        let already = self.relations.iter().any(|r| {
                            &r.from == start && &r.to == next && r.rel_type == RelationType::Causes
                        });
                        if !already && !visited.contains(next) {
                            new_relations.push(
                                Relation::new(start.clone(), next.clone(), RelationType::Causes)
                                    .with_weight(new_w)
                                    .with_metadata("inferred", "causes_chain")
                                    .with_metadata("chain_length", format!("{}", depth + 1)),
                            );
                        }
                        frontier.push((next.clone(), new_w, depth + 1));
                    }
                }
            }
        }

        let count = new_relations.len();
        for rel in new_relations {
            self.add_relation(rel);
        }
        count
    }

    pub fn infer_similar_transitivity(&mut self, max_hops: usize) -> usize {
        let mut new_relations = Vec::new();
        let adj: HashMap<AtomKey, Vec<(AtomKey, f64)>> = {
            let mut map: HashMap<AtomKey, Vec<(AtomKey, f64)>> = HashMap::new();
            for rel in &self.relations {
                if rel.rel_type == RelationType::SimilarTo {
                    map.entry(rel.from.clone())
                        .or_default()
                        .push((rel.to.clone(), rel.weight));
                }
            }
            map
        };

        for start in adj.keys() {
            let mut frontier = vec![(start.clone(), 0usize, 1.0f64)];
            let mut visited = HashSet::new();
            visited.insert(start.clone());

            while let Some((current, hops, cum_sim)) = frontier.pop() {
                if hops >= max_hops {
                    continue;
                }
                if let Some(neighbors) = adj.get(&current) {
                    for (next, w) in neighbors {
                        if visited.contains(next) {
                            continue;
                        }
                        visited.insert(next.clone());
                        let inferred_sim = cum_sim * w;
                        if inferred_sim < 0.3 {
                            continue;
                        }
                        let already = self.relations.iter().any(|r| {
                            (&r.from == start && &r.to == next)
                                || (&r.from == next && &r.to == start)
                        });
                        if !already {
                            new_relations.push(
                                Relation::new(start.clone(), next.clone(), RelationType::SimilarTo)
                                    .with_weight(inferred_sim)
                                    .with_metadata("inferred", "similar_transitivity")
                                    .with_metadata("hops", format!("{}", hops + 1)),
                            );
                        }
                        frontier.push((next.clone(), hops + 1, inferred_sim));
                    }
                }
            }
        }

        let count = new_relations.len();
        for rel in new_relations {
            self.add_relation(rel);
        }
        count
    }

    pub fn run_full_inference(&mut self) -> InferenceReport {
        let isa = self.infer_isa_inheritance();
        let causes = self.infer_causes_chains(5);
        let similar = self.infer_similar_transitivity(3);
        let transitive_partof = self.infer_transitive_closure(RelationType::PartOf, 4);
        InferenceReport {
            isa_inherited: isa,
            causes_inferred: causes,
            similar_inferred: similar,
            partof_transitive: transitive_partof,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InferenceReport {
    pub isa_inherited: usize,
    pub causes_inferred: usize,
    pub similar_inferred: usize,
    pub partof_transitive: usize,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// S4: Concept Abstraction Layer (with incremental centroid updates)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Concept {
    pub name: String,
    pub category: String,
    pub description: String,
    pub prototype_embedding: SemanticEmbedding,
    pub member_count: usize,
    pub coherence: f64,
    pub tags: Vec<String>,
}

impl std::fmt::Display for Concept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Concept[{}: cat={}, members={}, coherence={:.3}]",
            self.name, self.category, self.member_count, self.coherence
        )
    }
}

#[derive(Debug, Clone)]
struct ConceptCluster {
    centroid: SemanticEmbedding,
    sum_vector: [f64; EMBED_DIM],
    members: Vec<AtomKey>,
    category: Option<String>,
    tags: HashMap<String, usize>,
    #[allow(dead_code)]
    total_variance: f64,
}

impl ConceptCluster {
    fn new(embedding: SemanticEmbedding, key: AtomKey) -> Self {
        let mut sum_vector = [0.0f64; EMBED_DIM];
        for (i, &v) in embedding.vector.iter().enumerate() {
            sum_vector[i] = v;
        }
        Self {
            centroid: embedding.clone(),
            sum_vector,
            members: vec![key],
            category: None,
            tags: HashMap::new(),
            total_variance: 0.0,
        }
    }

    fn add_member(&mut self, key: AtomKey, embedding: &SemanticEmbedding) {
        self.members.push(key);
        let n = self.members.len() as f64;
        for (i, &v) in embedding.vector.iter().enumerate() {
            self.sum_vector[i] += v;
        }
        let mut new_centroid = [0.0f64; EMBED_DIM];
        for (i, &s) in self.sum_vector.iter().enumerate() {
            new_centroid[i] = s / n;
        }
        let norm: f64 = new_centroid.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut new_centroid {
                *v /= norm;
            }
        }
        self.centroid = SemanticEmbedding {
            vector: new_centroid,
        };
    }

    fn compute_coherence(&self, index: &EmbeddingIndex) -> f64 {
        if self.members.len() < 2 {
            return 1.0;
        }
        let mut total_sim = 0.0f64;
        let mut count = 0usize;
        for key in &self.members {
            if let Some(emb) = index.get_embedding(key) {
                total_sim += self.centroid.cosine_similarity(emb);
                count += 1;
            }
        }
        if count == 0 {
            return 0.0;
        }
        total_sim / count as f64
    }
}

#[derive(Debug, Clone)]
pub struct ConceptExtractor {
    similarity_threshold: f64,
    min_cluster_size: usize,
    max_concepts: usize,
    concepts: Vec<Concept>,
}

impl ConceptExtractor {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.85,
            min_cluster_size: 2,
            max_concepts: 100,
            concepts: Vec::new(),
        }
    }

    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_min_cluster_size(mut self, size: usize) -> Self {
        self.min_cluster_size = size.max(1);
        self
    }

    pub fn extract_concepts(
        &mut self,
        index: &EmbeddingIndex,
        atoms_by_key: &HashMap<AtomKey, &MemoryAtom>,
    ) -> &[Concept] {
        let mut clusters: Vec<ConceptCluster> = Vec::new();

        for entry in &index.entries {
            let mut best_cluster: Option<usize> = None;
            let mut best_sim = self.similarity_threshold;

            for (ci, cluster) in clusters.iter().enumerate() {
                let sim = entry.embedding.cosine_similarity(&cluster.centroid);
                if sim > best_sim {
                    best_sim = sim;
                    best_cluster = Some(ci);
                }
            }

            if let Some(ci) = best_cluster {
                clusters[ci].add_member(entry.atom_key.clone(), &entry.embedding);
                if let Some(atom) = atoms_by_key.get(&entry.atom_key) {
                    if let Some(cat) = atom.category() {
                        if clusters[ci].category.is_none() {
                            clusters[ci].category = Some(cat.to_string());
                        }
                    }
                    for tag in atom.tags() {
                        *clusters[ci].tags.entry(tag.clone()).or_insert(0) += 1;
                    }
                }
            } else {
                let mut cluster =
                    ConceptCluster::new(entry.embedding.clone(), entry.atom_key.clone());
                if let Some(atom) = atoms_by_key.get(&entry.atom_key) {
                    for tag in atom.tags() {
                        cluster.tags.insert(tag.clone(), 1);
                    }
                }
                clusters.push(cluster);
            }
        }

        clusters.retain(|c| c.members.len() >= self.min_cluster_size);
        clusters.truncate(self.max_concepts);

        self.concepts = clusters
            .into_iter()
            .enumerate()
            .map(|(i, cluster)| {
                let n = cluster.members.len();
                let coherence = cluster.compute_coherence(index);

                let category = cluster
                    .category
                    .unwrap_or_else(|| format!("auto_cluster_{}", i));

                let mut top_tags: Vec<(String, usize)> = cluster.tags.into_iter().collect();
                top_tags.sort_by_key(|b| std::cmp::Reverse(b.1));
                let top_tags: Vec<String> = top_tags.into_iter().take(5).map(|(t, _)| t).collect();

                Concept {
                    name: format!("concept_{}", i),
                    category,
                    description: format!("Auto-extracted concept with {} members", n),
                    prototype_embedding: cluster.centroid,
                    member_count: n,
                    coherence,
                    tags: top_tags,
                }
            })
            .collect();

        &self.concepts
    }

    pub fn concepts(&self) -> &[Concept] {
        &self.concepts
    }

    pub fn find_concept_for(&self, embedding: &SemanticEmbedding) -> Option<&Concept> {
        let mut best: Option<&Concept> = None;
        let mut best_sim = 0.0;
        for concept in &self.concepts {
            let sim = embedding.cosine_similarity(&concept.prototype_embedding);
            if sim > best_sim {
                best_sim = sim;
                best = Some(concept);
            }
        }
        best
    }
}

impl Default for ConceptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// S5: Semantic Query Language
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum QueryFilter {
    Category(String),
    Tag(String),
    TagAny(Vec<String>),
    TagAll(Vec<String>),
    ImportanceRange(f64, f64),
    Source(String),
    RelationType(RelationType),
    RelatedTo(AtomKey),
    SimilarToData(Vec<f64>),
    SimilarToEmbedding(Box<SemanticEmbedding>),
    MinSimilarity(f64),
    MaxResults(usize),
    ConceptName(String),
}

#[derive(Debug, Clone)]
pub struct SemanticQuery {
    filters: Vec<QueryFilter>,
}

impl SemanticQuery {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.filters.push(QueryFilter::Category(cat.into()));
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.filters.push(QueryFilter::Tag(tag.into()));
        self
    }

    pub fn tag_any(mut self, tags: Vec<String>) -> Self {
        self.filters.push(QueryFilter::TagAny(tags));
        self
    }

    pub fn tag_all(mut self, tags: Vec<String>) -> Self {
        self.filters.push(QueryFilter::TagAll(tags));
        self
    }

    pub fn importance_range(mut self, min: f64, max: f64) -> Self {
        self.filters.push(QueryFilter::ImportanceRange(min, max));
        self
    }

    pub fn source(mut self, src: impl Into<String>) -> Self {
        self.filters.push(QueryFilter::Source(src.into()));
        self
    }

    pub fn related_to(mut self, key: AtomKey, rel_type: RelationType) -> Self {
        self.filters.push(QueryFilter::RelatedTo(key));
        self.filters.push(QueryFilter::RelationType(rel_type));
        self
    }

    pub fn similar_to_data(mut self, data: Vec<f64>) -> Self {
        self.filters.push(QueryFilter::SimilarToData(data));
        self
    }

    pub fn similar_to_embedding(mut self, embedding: SemanticEmbedding) -> Self {
        self.filters
            .push(QueryFilter::SimilarToEmbedding(Box::new(embedding)));
        self
    }

    pub fn min_similarity(mut self, threshold: f64) -> Self {
        self.filters.push(QueryFilter::MinSimilarity(threshold));
        self
    }

    pub fn max_results(mut self, n: usize) -> Self {
        self.filters.push(QueryFilter::MaxResults(n));
        self
    }

    pub fn concept(mut self, name: impl Into<String>) -> Self {
        self.filters.push(QueryFilter::ConceptName(name.into()));
        self
    }

    pub fn execute(
        &self,
        atoms: &[MemoryAtom],
        index: &EmbeddingIndex,
        graph: &KnowledgeGraph,
        concepts: &[Concept],
    ) -> Vec<QueryHit> {
        let max_results = self
            .filters
            .iter()
            .find_map(|f| match f {
                QueryFilter::MaxResults(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(50);

        let min_sim = self
            .filters
            .iter()
            .find_map(|f| match f {
                QueryFilter::MinSimilarity(t) => Some(*t),
                _ => None,
            })
            .unwrap_or(0.0);

        let query_embedding: Option<SemanticEmbedding> =
            self.filters.iter().find_map(|f| match f {
                QueryFilter::SimilarToData(data) => Some(SemanticEmbedding::from_data(data)),
                QueryFilter::SimilarToEmbedding(e) => Some((**e).clone()),
                _ => None,
            });

        let similarity_map: Option<HashMap<AtomKey, f64>> = query_embedding
            .as_ref()
            .map(|emb| index.search_knn_with_scores(emb, index.len().max(1)));

        let related_keys: Option<Vec<AtomKey>> = self.filters.iter().find_map(|f| {
            if let QueryFilter::RelatedTo(key) = f {
                let neighbors = graph.neighbors(key, 3);
                Some(neighbors.into_iter().map(|(k, _, _)| k).collect())
            } else {
                None
            }
        });

        let concept_keys: Option<Vec<AtomKey>> = self.filters.iter().find_map(|f| {
            if let QueryFilter::ConceptName(name) = f {
                let prototype = concepts.iter().find(|c| c.name == *name)?;
                let knn = index.search_knn(&prototype.prototype_embedding, max_results);
                Some(knn.into_iter().map(|r| r.atom_key).collect())
            } else {
                None
            }
        });

        let filter_category: Option<&str> = self.filters.iter().find_map(|f| match f {
            QueryFilter::Category(c) => Some(c.as_str()),
            _ => None,
        });

        let filter_tag: Option<&str> = self.filters.iter().find_map(|f| match f {
            QueryFilter::Tag(t) => Some(t.as_str()),
            _ => None,
        });

        let filter_tag_any: Option<&Vec<String>> = self.filters.iter().find_map(|f| match f {
            QueryFilter::TagAny(ts) => Some(ts),
            _ => None,
        });

        let filter_tag_all: Option<&Vec<String>> = self.filters.iter().find_map(|f| match f {
            QueryFilter::TagAll(ts) => Some(ts),
            _ => None,
        });

        let filter_source: Option<&str> = self.filters.iter().find_map(|f| match f {
            QueryFilter::Source(s) => Some(s.as_str()),
            _ => None,
        });

        let filter_importance: Option<(f64, f64)> = self.filters.iter().find_map(|f| match f {
            QueryFilter::ImportanceRange(lo, hi) => Some((*lo, *hi)),
            _ => None,
        });

        let mut hits: Vec<QueryHit> = Vec::new();

        for atom in atoms {
            let key = AtomKey::from_atom(atom);

            if let Some(cat) = filter_category {
                if atom.category() != Some(cat) {
                    continue;
                }
            }

            if let Some(tag) = filter_tag {
                if !atom.has_tag(tag) {
                    continue;
                }
            }

            if let Some(tags) = filter_tag_any {
                if !tags.iter().any(|t| atom.has_tag(t)) {
                    continue;
                }
            }

            if let Some(tags) = filter_tag_all {
                if !tags.iter().all(|t| atom.has_tag(t)) {
                    continue;
                }
            }

            if let Some(src) = filter_source {
                if atom.source() != Some(src) {
                    continue;
                }
            }

            if let Some((lo, hi)) = filter_importance {
                let imp = atom.importance();
                if imp < lo || imp > hi {
                    continue;
                }
            }

            if let Some(keys) = &related_keys {
                if !keys.contains(&key) {
                    continue;
                }
            }

            if let Some(keys) = &concept_keys {
                if !keys.contains(&key) {
                    continue;
                }
            }

            let similarity = if let Some(ref sim_map) = similarity_map {
                *sim_map.get(&key).unwrap_or(&0.0)
            } else {
                1.0
            };

            if similarity < min_sim {
                continue;
            }

            hits.push(QueryHit {
                atom_key: key,
                similarity,
                matched_filters: self.summarize_matched_filters(atom),
            });

            if hits.len() >= max_results {
                break;
            }
        }

        if query_embedding.is_some() {
            hits.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        hits
    }

    fn summarize_matched_filters(&self, _atom: &MemoryAtom) -> Vec<String> {
        self.filters.iter().map(|f| format!("{:?}", f)).collect()
    }
}

impl Default for SemanticQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct QueryHit {
    pub atom_key: AtomKey,
    pub similarity: f64,
    pub matched_filters: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// Unified Semantic Engine
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SemanticConfig {
    pub embedding_dim: usize,
    pub knn_default_k: usize,
    pub concept_similarity_threshold: f64,
    pub concept_min_size: usize,
    pub auto_link_similarity: f64,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            embedding_dim: EMBED_DIM,
            knn_default_k: 10,
            concept_similarity_threshold: 0.85,
            concept_min_size: 2,
            auto_link_similarity: 0.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticReport {
    pub embeddings_indexed: usize,
    pub relations_total: usize,
    pub concepts_extracted: usize,
    pub auto_links_created: usize,
}

pub struct SemanticEngine {
    config: SemanticConfig,
    index: EmbeddingIndex,
    graph: KnowledgeGraph,
    concept_extractor: ConceptExtractor,
    tfidf_index: TfIdfIndex,
}

impl SemanticEngine {
    pub fn new(config: SemanticConfig) -> Self {
        let concept_extractor = ConceptExtractor::new()
            .with_similarity_threshold(config.concept_similarity_threshold)
            .with_min_cluster_size(config.concept_min_size);
        Self {
            config,
            index: EmbeddingIndex::new(),
            graph: KnowledgeGraph::new(),
            concept_extractor,
            tfidf_index: TfIdfIndex::new(),
        }
    }

    pub fn index_memory(&mut self, atom: &MemoryAtom, data: &[f64]) {
        let key = AtomKey::from_atom(atom);
        let embedding = SemanticEmbedding::from_data_and_annotation(data, atom);
        let category = atom.category().map(String::from);
        self.index.upsert(key, embedding, category);
        self.index_tfidf_text(atom);
    }

    pub fn index_memory_data_only(&mut self, atom: &MemoryAtom, data: &[f64]) {
        let key = AtomKey::from_atom(atom);
        let embedding = SemanticEmbedding::from_data(data);
        let category = atom.category().map(String::from);
        self.index.upsert(key, embedding, category);
    }

    fn index_tfidf_text(&mut self, atom: &MemoryAtom) {
        let tags_joined = atom.tags().join(" ");
        let text_parts: Vec<&str> = vec![
            atom.description().unwrap_or(""),
            atom.category().unwrap_or(""),
            &tags_joined,
        ];
        let text = text_parts.join(" ");
        if !text.trim().is_empty() {
            self.tfidf_index.add_document(&text);
        }
    }

    pub fn unindex_memory(&mut self, atom: &MemoryAtom) {
        let key = AtomKey::from_atom(atom);
        self.index.remove(&key);
        self.graph.remove_relations_for(&key);
    }

    pub fn add_relation(&mut self, relation: Relation) {
        self.graph.add_relation(relation);
    }

    pub fn auto_link_similar(&mut self, atoms: &[MemoryAtom]) -> usize {
        let threshold = self.config.auto_link_similarity;
        let k = self.config.knn_default_k;
        let mut links = 0;

        for atom in atoms {
            let key = AtomKey::from_atom(atom);
            let embedding = SemanticEmbedding::from_data_and_annotation(&[atom.importance()], atom);
            let results = self.index.search_knn(&embedding, k);

            for result in results {
                if result.similarity >= threshold && result.atom_key != key {
                    let existing = self.graph.relations_from(&key);
                    let already = existing
                        .iter()
                        .any(|r| r.to == result.atom_key && r.rel_type == RelationType::SimilarTo);
                    if !already {
                        self.graph.add_relation(
                            Relation::new(key.clone(), result.atom_key, RelationType::SimilarTo)
                                .with_weight(result.similarity),
                        );
                        links += 1;
                    }
                }
            }
        }

        links
    }

    pub fn extract_concepts(&mut self, atoms_by_key: &HashMap<AtomKey, &MemoryAtom>) -> &[Concept] {
        self.concept_extractor
            .extract_concepts(&self.index, atoms_by_key)
    }

    pub fn query(&self) -> SemanticQuery {
        SemanticQuery::new()
    }

    pub fn execute_query(&self, q: &SemanticQuery, atoms: &[MemoryAtom]) -> Vec<QueryHit> {
        q.execute(
            atoms,
            &self.index,
            &self.graph,
            self.concept_extractor.concepts(),
        )
    }

    pub fn search_similar(&self, data: &[f64], k: usize) -> Vec<KnnResult> {
        let embedding = SemanticEmbedding::from_data(data);
        self.index.search_knn(&embedding, k)
    }

    pub fn search_similar_by_annotation(&self, atom: &MemoryAtom, k: usize) -> Vec<KnnResult> {
        let embedding = SemanticEmbedding::from_annotation(atom);
        self.index.search_knn(&embedding, k)
    }

    pub fn search_by_text(&self, text: &str, k: usize) -> Vec<KnnResult> {
        let text_vec = nlp::text_to_tfidf_embedding(text, &self.tfidf_index);
        let mut vec = [0.0f64; EMBED_DIM];
        for (i, slot) in vec
            .iter_mut()
            .enumerate()
            .take(EMBED_DIM.min(text_vec.len()))
        {
            *slot = text_vec[i];
        }
        let norm: f64 = vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        let embedding = SemanticEmbedding { vector: vec };
        self.index.search_knn(&embedding, k)
    }

    pub fn find_relations(&self, atom: &MemoryAtom) -> Vec<&Relation> {
        let key = AtomKey::from_atom(atom);
        let mut all = self.graph.relations_from(&key);
        all.extend(self.graph.relations_to(&key));
        all
    }

    pub fn find_concept(&self, atom: &MemoryAtom) -> Option<&Concept> {
        let embedding = SemanticEmbedding::from_annotation(atom);
        self.concept_extractor.find_concept_for(&embedding)
    }

    pub fn report(&self) -> SemanticReport {
        SemanticReport {
            embeddings_indexed: self.index.len(),
            relations_total: self.graph.relation_count(),
            concepts_extracted: self.concept_extractor.concepts().len(),
            auto_links_created: 0,
        }
    }

    pub fn index_ref(&self) -> &EmbeddingIndex {
        &self.index
    }

    pub fn graph_ref(&self) -> &KnowledgeGraph {
        &self.graph
    }

    pub fn concepts_ref(&self) -> &[Concept] {
        self.concept_extractor.concepts()
    }

    pub fn tfidf_index_ref(&self) -> &TfIdfIndex {
        &self.tfidf_index
    }

    pub fn tfidf_index_mut(&mut self) -> &mut TfIdfIndex {
        &mut self.tfidf_index
    }

    pub fn search_multihop(
        &self,
        text: &str,
        k: usize,
        max_hops: usize,
        hebbian: &crate::universe::hebbian::HebbianMemory,
        mems: &[MemoryAtom],
    ) -> Vec<MultihopResult> {
        let initial = self.search_by_text(text, k);
        if initial.is_empty() || max_hops == 0 {
            return initial
                .into_iter()
                .map(|r| MultihopResult {
                    atom_key: r.atom_key,
                    similarity: r.similarity,
                    distance: r.distance,
                    hop: 0,
                    path: vec![],
                })
                .collect();
        }

        let key_to_anchor: HashMap<AtomKey, crate::universe::coord::Coord7D> = mems
            .iter()
            .map(|m| {
                let key = AtomKey::from_atom(m);
                (key, *m.anchor())
            })
            .collect();

        let anchor_to_key: HashMap<crate::universe::coord::Coord7D, AtomKey> = mems
            .iter()
            .map(|m| {
                let key = AtomKey::from_atom(m);
                (*m.anchor(), key)
            })
            .collect();

        let mut seen: HashMap<AtomKey, (f64, usize, Vec<AtomKey>)> = HashMap::new();
        for r in &initial {
            seen.insert(r.atom_key.clone(), (r.similarity, 0, vec![]));
        }

        let mut frontier: Vec<(AtomKey, usize, f64)> = initial
            .iter()
            .map(|r| (r.atom_key.clone(), 0, r.similarity))
            .collect();

        while let Some((current_key, hop, base_sim)) = frontier.pop() {
            if hop >= max_hops {
                continue;
            }

            let Some(coord) = key_to_anchor.get(&current_key) else {
                continue;
            };

            let neighbors = hebbian.get_neighbors(coord);
            for (neighbor_coord, edge_weight) in &neighbors {
                let Some(neighbor_key) = anchor_to_key.get(neighbor_coord) else {
                    continue;
                };

                let combined_sim = base_sim * edge_weight * 0.8;
                if combined_sim < 0.01 {
                    continue;
                }

                let new_hop = hop + 1;
                let should_update = match seen.get(neighbor_key) {
                    Some(&(existing_sim, existing_hop, _)) => {
                        combined_sim > existing_sim || new_hop < existing_hop
                    }
                    None => true,
                };

                if should_update {
                    let mut path_to_current = seen
                        .get(&current_key)
                        .map(|(_, _, p)| p.clone())
                        .unwrap_or_default();
                    path_to_current.push(current_key.clone());

                    seen.insert(
                        neighbor_key.clone(),
                        (combined_sim, new_hop, path_to_current),
                    );
                    frontier.push((neighbor_key.clone(), new_hop, combined_sim));
                }
            }
        }

        let mut results: Vec<MultihopResult> = seen
            .into_iter()
            .map(|(key, (sim, hop, path))| MultihopResult {
                atom_key: key,
                similarity: sim,
                distance: 1.0 - sim,
                hop,
                path,
            })
            .collect();
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(k);
        results
    }

    pub fn find_analogies_semantic(
        &self,
        memories: &[MemoryAtom],
        threshold: f64,
    ) -> Vec<SemanticAnalogy> {
        let keys: Vec<AtomKey> = memories.iter().map(AtomKey::from_atom).collect();
        let mut results = Vec::new();

        for i in 0..keys.len() {
            let Some(emb_a) = self.index.get_embedding(&keys[i]) else {
                continue;
            };
            for j in (i + 1)..keys.len() {
                let Some(emb_b) = self.index.get_embedding(&keys[j]) else {
                    continue;
                };
                let sim = emb_a.cosine_similarity(emb_b);
                if sim >= threshold {
                    results.push(SemanticAnalogy {
                        from: keys[i].clone(),
                        to: keys[j].clone(),
                        similarity: sim,
                    });
                }
            }
        }

        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    pub fn sync_after_dream(
        &mut self,
        surviving_memories: &[MemoryAtom],
        universe: &crate::universe::node::DarkUniverse,
    ) -> DreamSyncReport {
        let _before_index = self.index.len();
        let before_relations = self.graph.relation_count();

        let surviving_keys: HashSet<AtomKey> =
            surviving_memories.iter().map(AtomKey::from_atom).collect();

        let indexed_keys: Vec<AtomKey> = self
            .index
            .entries
            .iter()
            .map(|e| e.atom_key.clone())
            .collect();
        let mut removed = 0usize;
        for key in &indexed_keys {
            if !surviving_keys.contains(key) {
                self.index.remove(key);
                self.graph.remove_relations_for(key);
                removed += 1;
            }
        }

        for atom in surviving_memories {
            let key = AtomKey::from_atom(atom);
            if self.index.get_embedding(&key).is_none() {
                if let Ok(data) = crate::universe::memory::MemoryCodec::decode(universe, atom) {
                    self.index_memory(atom, &data);
                }
            } else {
                if let Ok(data) = crate::universe::memory::MemoryCodec::decode(universe, atom) {
                    let new_emb = SemanticEmbedding::from_data_and_annotation(&data, atom);
                    let category = atom.category().map(String::from);
                    self.index.upsert(key, new_emb, category);
                }
            }
        }

        self.auto_link_similar(surviving_memories);

        DreamSyncReport {
            embeddings_removed: removed,
            embeddings_indexed: self.index.len(),
            relations_before: before_relations,
            relations_after: self.graph.relation_count(),
        }
    }

    pub fn run_inference(&mut self) -> InferenceReport {
        self.graph.run_full_inference()
    }

    pub fn search_by_text_emotional(
        &self,
        text: &str,
        k: usize,
        pad: Option<[f64; 3]>,
    ) -> Vec<KnnResult> {
        let mut results = self.search_by_text(text, k * 2);
        if let Some(pad_vals) = pad {
            let pleasure = pad_vals[0];
            let arousal = pad_vals[1];
            let dominance = pad_vals[2];

            let emotion_boost = 1.0 + pleasure * 0.15 + arousal * 0.1;
            let emotion_penalty = 1.0 - (1.0 - dominance) * 0.2;

            for result in &mut results {
                let raw_sim = result.similarity;
                let adjusted = raw_sim * emotion_boost * emotion_penalty;
                result.similarity = adjusted.clamp(0.0, 1.0);
                result.distance = 1.0 - result.similarity;
            }

            results.sort_by(|a, b| {
                b.similarity
                    .partial_cmp(&a.similarity)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        results.truncate(k);
        results
    }

    pub fn extract_concepts_emotional(
        &mut self,
        atoms_by_key: &HashMap<AtomKey, &MemoryAtom>,
        pad: Option<[f64; 3]>,
    ) -> &[Concept] {
        if let Some(pad_vals) = pad {
            let arousal = pad_vals[1].abs();
            let threshold_adjustment = arousal * 0.1;
            let adjusted_threshold =
                (self.config.concept_similarity_threshold - threshold_adjustment).clamp(0.5, 0.99);
            self.concept_extractor = ConceptExtractor::new()
                .with_similarity_threshold(adjusted_threshold)
                .with_min_cluster_size(self.config.concept_min_size);
        }
        self.concept_extractor
            .extract_concepts(&self.index, atoms_by_key)
    }
}

#[derive(Debug, Clone)]
pub struct MultihopResult {
    pub atom_key: AtomKey,
    pub similarity: f64,
    pub distance: f64,
    pub hop: usize,
    pub path: Vec<AtomKey>,
}

#[derive(Debug, Clone)]
pub struct SemanticAnalogy {
    pub from: AtomKey,
    pub to: AtomKey,
    pub similarity: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DreamSyncReport {
    pub embeddings_removed: usize,
    pub embeddings_indexed: usize,
    pub relations_before: usize,
    pub relations_after: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universe::core::coord::Coord7D;
    use crate::universe::memory::MemoryCodec;
    use crate::universe::node::DarkUniverse;

    fn make_test_universe() -> DarkUniverse {
        DarkUniverse::new(100000.0)
    }

    fn make_test_atom(universe: &mut DarkUniverse, data: &[f64]) -> MemoryAtom {
        let anchor = Coord7D::new_even([0; 7]);
        MemoryCodec::encode(universe, &anchor, data).unwrap()
    }

    fn make_atom_at(universe: &mut DarkUniverse, offset: i32, data: &[f64]) -> MemoryAtom {
        let anchor = Coord7D::new_even([offset * 20, 0, 0, 0, 0, 0, 0]);
        MemoryCodec::encode(universe, &anchor, data).unwrap()
    }

    #[test]
    fn embedding_from_data_is_normalized() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let emb = SemanticEmbedding::from_data(&data);
        let norm: f64 = emb.vector.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-10,
            "embedding should be unit-normalized, got norm={}",
            norm
        );
    }

    #[test]
    fn identical_data_same_embedding() {
        let data = vec![1.0, 2.0, 3.0];
        let a = SemanticEmbedding::from_data(&data);
        let b = SemanticEmbedding::from_data(&data);
        let sim = a.cosine_similarity(&b);
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "identical data should have similarity 1.0, got {}",
            sim
        );
    }

    #[test]
    fn different_data_lower_similarity() {
        let a = SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]);
        let b = SemanticEmbedding::from_data(&[100.0, 200.0, 300.0]);
        let sim = a.cosine_similarity(&b);
        assert!(sim < 1.0, "different data should have similarity < 1.0");
    }

    #[test]
    fn cosine_similarity_symmetric() {
        let a = SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]);
        let b = SemanticEmbedding::from_data(&[4.0, 5.0, 6.0]);
        assert!((a.cosine_similarity(&b) - b.cosine_similarity(&a)).abs() < 1e-10);
    }

    #[test]
    fn knn_returns_sorted_by_similarity() {
        let mut index = EmbeddingIndex::new();
        let mut u = make_test_universe();

        let base_data = vec![1.0, 2.0, 3.0];
        let atom_close = make_atom_at(&mut u, 1, &[1.0, 2.1, 3.0]);
        let atom_mid = make_atom_at(&mut u, 2, &[1.0, 5.0, 3.0]);
        let atom_far = make_atom_at(&mut u, 3, &[100.0, 200.0, 300.0]);

        index.upsert(
            AtomKey::from_atom(&atom_close),
            SemanticEmbedding::from_data(&[1.0, 2.1, 3.0]),
            None,
        );
        index.upsert(
            AtomKey::from_atom(&atom_mid),
            SemanticEmbedding::from_data(&[1.0, 5.0, 3.0]),
            None,
        );
        index.upsert(
            AtomKey::from_atom(&atom_far),
            SemanticEmbedding::from_data(&[100.0, 200.0, 300.0]),
            None,
        );

        let query = SemanticEmbedding::from_data(&base_data);
        let results = index.search_knn(&query, 3);

        assert_eq!(results.len(), 3);
        assert!(results[0].similarity >= results[1].similarity);
        assert!(results[1].similarity >= results[2].similarity);
    }

    #[test]
    fn knn_upsert_replaces() {
        let mut index = EmbeddingIndex::new();
        let key = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        index.upsert(
            key.clone(),
            SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]),
            None,
        );
        assert_eq!(index.len(), 1);
        index.upsert(
            key.clone(),
            SemanticEmbedding::from_data(&[4.0, 5.0, 6.0]),
            None,
        );
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn knn_remove_works() {
        let mut index = EmbeddingIndex::new();
        let key = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        index.upsert(
            key.clone(),
            SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]),
            None,
        );
        assert_eq!(index.len(), 1);
        index.remove(&key);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn search_by_category_works() {
        let mut index = EmbeddingIndex::new();
        let key_a = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        let key_b = AtomKey {
            vertices_basis: [[1; 7]; 4],
            vertices_even: [false; 4],
        };
        index.upsert(
            key_a.clone(),
            SemanticEmbedding::from_data(&[1.0]),
            Some("rust".to_string()),
        );
        index.upsert(
            key_b.clone(),
            SemanticEmbedding::from_data(&[2.0]),
            Some("python".to_string()),
        );

        let rust = index.search_by_category("rust");
        assert_eq!(rust.len(), 1);
        assert_eq!(rust[0].0, key_a);
    }

    #[test]
    fn knowledge_graph_basic_operations() {
        let mut graph = KnowledgeGraph::new();
        let a = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        let b = AtomKey {
            vertices_basis: [[1; 7]; 4],
            vertices_even: [false; 4],
        };

        graph.add_relation(Relation::new(a.clone(), b.clone(), RelationType::Causes));

        assert_eq!(graph.relation_count(), 1);
        let from_a = graph.relations_from(&a);
        assert_eq!(from_a.len(), 1);
        assert_eq!(from_a[0].rel_type, RelationType::Causes);

        let to_b = graph.relations_to(&b);
        assert_eq!(to_b.len(), 1);
    }

    #[test]
    fn knowledge_graph_symmetric_relations() {
        let mut graph = KnowledgeGraph::new();
        let a = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        let b = AtomKey {
            vertices_basis: [[1; 7]; 4],
            vertices_even: [false; 4],
        };

        graph.add_relation(Relation::new(a.clone(), b.clone(), RelationType::SimilarTo));

        assert_eq!(graph.relation_count(), 2);
        assert_eq!(graph.relations_from(&a).len(), 1);
        assert_eq!(graph.relations_from(&b).len(), 1);
    }

    #[test]
    fn knowledge_graph_neighbors() {
        let mut graph = KnowledgeGraph::new();
        let a = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        let b = AtomKey {
            vertices_basis: [[1; 7]; 4],
            vertices_even: [false; 4],
        };
        let c = AtomKey {
            vertices_basis: [[2; 7]; 4],
            vertices_even: [true; 4],
        };

        graph.add_relation(Relation::new(a.clone(), b.clone(), RelationType::Causes));
        graph.add_relation(Relation::new(b.clone(), c.clone(), RelationType::Causes));

        let neighbors = graph.neighbors(&a, 2);
        assert!(neighbors.len() >= 2);
    }

    #[test]
    fn knowledge_graph_remove_cascade() {
        let mut graph = KnowledgeGraph::new();
        let a = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        let b = AtomKey {
            vertices_basis: [[1; 7]; 4],
            vertices_even: [false; 4],
        };

        graph.add_relation(Relation::new(a.clone(), b.clone(), RelationType::Causes));
        assert_eq!(graph.relation_count(), 1);

        graph.remove_relations_for(&a);
        assert_eq!(graph.relation_count(), 0);
    }

    #[test]
    fn concept_extraction_basic() {
        let mut extractor = ConceptExtractor::new()
            .with_similarity_threshold(0.5)
            .with_min_cluster_size(1);
        let mut index = EmbeddingIndex::new();
        let mut atoms_map: HashMap<AtomKey, MemoryAtom> = HashMap::new();

        let mut u = make_test_universe();
        for i in 0..5 {
            let atom = make_atom_at(&mut u, i, &[1.0, 2.0, 3.0]);
            let key = AtomKey::from_atom(&atom);
            index.upsert(
                key.clone(),
                SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]),
                None,
            );
            atoms_map.insert(key, atom);
        }

        let refs: HashMap<AtomKey, &MemoryAtom> =
            atoms_map.iter().map(|(k, v)| (k.clone(), v)).collect();
        let concepts = extractor.extract_concepts(&index, &refs).to_vec();
        assert!(!concepts.is_empty(), "should extract at least one concept");
        assert!(
            concepts[0].member_count >= 1,
            "concept should have at least 1 member"
        );
    }

    #[test]
    fn concept_centroid_updates() {
        let mut extractor = ConceptExtractor::new()
            .with_similarity_threshold(0.5)
            .with_min_cluster_size(1);
        let mut index = EmbeddingIndex::new();
        let mut atoms_map: HashMap<AtomKey, MemoryAtom> = HashMap::new();
        let mut u = make_test_universe();

        let atom = make_atom_at(&mut u, 0, &[1.0, 2.0, 3.0]);
        let key = AtomKey::from_atom(&atom);
        index.upsert(
            key.clone(),
            SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]),
            None,
        );
        atoms_map.insert(key, atom);

        let refs: HashMap<AtomKey, &MemoryAtom> =
            atoms_map.iter().map(|(k, v)| (k.clone(), v)).collect();
        extractor.extract_concepts(&index, &refs);

        let query = SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]);
        let found = extractor.find_concept_for(&query);
        assert!(found.is_some(), "should find concept for matching data");
    }

    #[test]
    fn semantic_query_by_tag() {
        let mut engine = SemanticEngine::new(SemanticConfig::default());
        let mut u = DarkUniverse::new(500000.0);

        let atom1 = make_atom_at(&mut u, 1, &[1.0, 2.0, 3.0]);
        let atom2 = make_atom_at(&mut u, 2, &[4.0, 5.0, 6.0]);

        let mut atom1_mut = atom1;
        atom1_mut.add_tag("test");
        let mut atom2_mut = atom2;
        atom2_mut.add_tag("other");

        engine.index_memory(&atom1_mut, &[1.0, 2.0, 3.0]);
        engine.index_memory(&atom2_mut, &[4.0, 5.0, 6.0]);

        let atoms = vec![atom1_mut, atom2_mut];
        let hits = engine.query().tag("test".to_string()).execute(
            &atoms,
            engine.index_ref(),
            engine.graph_ref(),
            engine.concepts_ref(),
        );

        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn semantic_query_by_importance() {
        let mut engine = SemanticEngine::new(SemanticConfig::default());
        let mut u = DarkUniverse::new(500000.0);

        let mut atom1 = make_atom_at(&mut u, 1, &[1.0, 2.0, 3.0]);
        atom1.set_importance(0.9);
        let mut atom2 = make_atom_at(&mut u, 2, &[4.0, 5.0, 6.0]);
        atom2.set_importance(0.1);

        engine.index_memory(&atom1, &[1.0, 2.0, 3.0]);
        engine.index_memory(&atom2, &[4.0, 5.0, 6.0]);

        let atoms = vec![atom1, atom2];
        let hits = engine.query().importance_range(0.8, 1.0).execute(
            &atoms,
            engine.index_ref(),
            engine.graph_ref(),
            engine.concepts_ref(),
        );

        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn semantic_query_similar_to_data() {
        let mut engine = SemanticEngine::new(SemanticConfig::default());
        let mut u = DarkUniverse::new(500000.0);

        let atom1 = make_atom_at(&mut u, 1, &[1.0, 2.0, 3.0]);
        let atom2 = make_atom_at(&mut u, 2, &[100.0, 200.0, 300.0]);

        engine.index_memory(&atom1, &[1.0, 2.0, 3.0]);
        engine.index_memory(&atom2, &[100.0, 200.0, 300.0]);

        let atoms = vec![atom1, atom2];
        let hits = engine
            .query()
            .similar_to_data(vec![1.0, 2.0, 3.0])
            .min_similarity(0.5)
            .execute(
                &atoms,
                engine.index_ref(),
                engine.graph_ref(),
                engine.concepts_ref(),
            );

        assert!(!hits.is_empty());
        assert!(hits[0].similarity > hits.last().unwrap().similarity || hits.len() == 1);
    }

    #[test]
    fn semantic_engine_full_workflow() {
        let mut engine = SemanticEngine::new(SemanticConfig::default());
        let mut u = DarkUniverse::new(1000000.0);
        let mut atoms = Vec::new();
        let mut data_sets = Vec::new();

        for i in 0..5 {
            let data: Vec<f64> = (0..7).map(|d| (i * 7 + d) as f64 * 0.5).collect();
            let mut atom = make_atom_at(&mut u, i, &data);
            atom.add_tag(format!("cluster_{}", i % 2));
            atom.set_category(if i % 2 == 0 { "even" } else { "odd" });
            engine.index_memory(&atom, &data);
            atoms.push(atom);
            data_sets.push(data);
        }

        let a_key = AtomKey::from_atom(&atoms[0]);
        let b_key = AtomKey::from_atom(&atoms[1]);
        engine.add_relation(Relation::new(a_key, b_key, RelationType::RelatedTo));

        let report = engine.report();
        assert_eq!(report.embeddings_indexed, 5);
        assert!(report.relations_total > 0);

        let results = engine.search_similar(&data_sets[0], 3);
        assert!(!results.is_empty());

        let atoms_by_key: HashMap<AtomKey, &MemoryAtom> =
            atoms.iter().map(|a| (AtomKey::from_atom(a), a)).collect();
        let _concepts = engine.extract_concepts(&atoms_by_key).to_vec();

        let cat_hits = engine.execute_query(&SemanticQuery::new().category("even"), &atoms);
        assert_eq!(cat_hits.len(), 3);
    }

    #[test]
    fn search_by_text_basic() {
        let mut engine = SemanticEngine::new(SemanticConfig::default());
        let mut u = DarkUniverse::new(500000.0);

        let mut atom1 = make_atom_at(&mut u, 1, &[1.0, 2.0, 3.0]);
        atom1.set_description("machine learning algorithm");
        atom1.set_category("ml");
        let mut atom2 = make_atom_at(&mut u, 2, &[4.0, 5.0, 6.0]);
        atom2.set_description("cooking recipes for dinner");
        atom2.set_category("food");

        engine.index_memory(&atom1, &[1.0, 2.0, 3.0]);
        engine.index_memory(&atom2, &[4.0, 5.0, 6.0]);

        let results = engine.search_by_text("machine learning", 5);
        assert!(!results.is_empty(), "should find results for text search");
    }

    #[test]
    fn relation_type_roundtrip() {
        for rt in RelationType::all() {
            let s = rt.as_str();
            let rt2 = RelationType::from_str_lossy(s);
            assert_eq!(Some(*rt), rt2, "roundtrip failed for {:?}", rt);
        }
    }

    #[test]
    fn symmetric_types_correct() {
        assert!(RelationType::SimilarTo.is_symmetric());
        assert!(RelationType::Contradicts.is_symmetric());
        assert!(RelationType::RelatedTo.is_symmetric());
        assert!(!RelationType::IsA.is_symmetric());
        assert!(!RelationType::Causes.is_symmetric());
        assert!(!RelationType::PartOf.is_symmetric());
        assert!(!RelationType::Precedes.is_symmetric());
        assert!(!RelationType::DerivedFrom.is_symmetric());
    }

    #[test]
    fn embedding_from_annotation() {
        let mut u = make_test_universe();
        let mut atom = make_test_atom(&mut u, &[1.0, 2.0, 3.0]);
        atom.set_category("test_cat");
        atom.add_tag("t1");
        atom.set_source("unit_test");
        atom.set_description("a test memory");

        let emb = SemanticEmbedding::from_annotation(&atom);
        let norm: f64 = emb.vector.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-10 || norm < 1e-10,
            "annotation embedding should be normalized or zero"
        );
    }

    #[test]
    fn centroid_update_works() {
        let a = SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]);
        let b = SemanticEmbedding::from_data(&[2.0, 3.0, 4.0]);
        let centroid = b.update_centroid(&a, 1);
        let sim_a = centroid.cosine_similarity(&a);
        let sim_b = centroid.cosine_similarity(&b);
        assert!(sim_a > 0.9, "centroid should be similar to a");
        assert!(sim_b > 0.9, "centroid should be similar to b");
    }

    #[test]
    fn embedding_dimension_is_64() {
        assert_eq!(EMBED_DIM, 64);
        let emb = SemanticEmbedding::zero();
        assert_eq!(emb.vector.len(), 64);
    }
}
