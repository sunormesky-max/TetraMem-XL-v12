// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
//
// Semantic Understanding Layers (S2-S5):
//   S2: Semantic Embedding — vector embedding + KNN search
//   S3: Knowledge Graph — typed relations between memories
//   S4: Concept Abstraction — dream-driven prototype extraction
//   S5: Semantic Query — unified query language

use crate::universe::memory::MemoryAtom;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════
// S2: Semantic Embedding Layer
// ═══════════════════════════════════════════════════════════════════════

const EMBED_DIM: usize = 16;

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
        let range = max_val - min_val;

        vec[0] = mean;
        vec[1] = variance.sqrt();
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
        let std_dev = variance.sqrt().max(1e-10);
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

            let median_idx = data.len() / 2;
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            vec[11] = sorted[median_idx];
        }

        if data.len() > 2 {
            let mut diffs: Vec<f64> = Vec::with_capacity(data.len() - 1);
            for i in 1..data.len() {
                diffs.push((data[i] - data[i - 1]).abs());
            }
            let diff_mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
            vec[12] = diff_mean;
        }

        vec[13] = mean * mean;
        vec[14] = mean * std_dev;
        vec[15] = range * entropy;

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

        let cat_hash = atom
            .category()
            .map(|c| {
                let mut h: u64 = 0;
                for b in c.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h as f64
            })
            .unwrap_or(0.0);
        vec[0] = cat_hash;

        vec[1] = atom.tags().len() as f64;

        let desc_len = atom.description().map(|d| d.len() as f64).unwrap_or(0.0);
        vec[2] = desc_len;

        let src_hash = atom
            .source()
            .map(|s| {
                let mut h: u64 = 0;
                for b in s.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as u64);
                }
                h as f64
            })
            .unwrap_or(0.0);
        vec[3] = src_hash;

        vec[4] = atom.importance();
        vec[5] = atom.data_dim() as f64;

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
            *slot = 0.7 * data_emb.vector[i] + 0.3 * ann_emb.vector[i];
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
        dot / denom
    }

    pub fn euclidean_distance(&self, other: &Self) -> f64 {
        let mut sum = 0.0f64;
        for i in 0..EMBED_DIM {
            let d = self.vector[i] - other.vector[i];
            sum += d * d;
        }
        sum.sqrt()
    }
}

#[derive(Debug, Clone)]
struct EmbeddingEntry {
    atom_key: AtomKey,
    embedding: SemanticEmbedding,
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
}

impl EmbeddingIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn upsert(&mut self, key: AtomKey, embedding: SemanticEmbedding) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.atom_key == key) {
            existing.embedding = embedding;
        } else {
            self.entries.push(EmbeddingEntry {
                atom_key: key,
                embedding,
            });
        }
    }

    pub fn remove(&mut self, key: &AtomKey) {
        self.entries.retain(|e| &e.atom_key != key);
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

    pub fn search_by_category(&self, _category: &str) -> Vec<(AtomKey, f64)> {
        self.entries
            .iter()
            .filter(|_| true)
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
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// S4: Concept Abstraction Layer
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
    members: Vec<AtomKey>,
    category: Option<String>,
    tags: HashMap<String, usize>,
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
        let entries: Vec<&EmbeddingEntry> = index.entries.iter().collect();

        for entry in &entries {
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
                clusters[ci].members.push(entry.atom_key.clone());
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
                let mut tags = HashMap::new();
                if let Some(atom) = atoms_by_key.get(&entry.atom_key) {
                    for tag in atom.tags() {
                        tags.insert(tag.clone(), 1);
                    }
                }
                clusters.push(ConceptCluster {
                    centroid: entry.embedding.clone(),
                    members: vec![entry.atom_key.clone()],
                    category: None,
                    tags,
                });
            }
        }

        clusters.retain(|c| c.members.len() >= self.min_cluster_size);
        clusters.truncate(self.max_concepts);

        self.concepts = clusters
            .into_iter()
            .enumerate()
            .map(|(i, cluster)| {
                let n = cluster.members.len();
                let coherence = if n > 1 {
                    1.0 / (1.0 + (n as f64).ln().max(0.01))
                } else {
                    1.0
                };

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
    SimilarToEmbedding(SemanticEmbedding),
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
            .push(QueryFilter::SimilarToEmbedding(embedding));
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
                QueryFilter::SimilarToEmbedding(e) => Some(e.clone()),
                _ => None,
            });

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

            let similarity = if let Some(ref emb) = query_embedding {
                let knn_results = index.search_knn(emb, index.len().max(1));
                knn_results
                    .iter()
                    .find(|r| r.atom_key == key)
                    .map(|r| r.similarity)
                    .unwrap_or(0.0)
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
        }
    }

    pub fn index_memory(&mut self, atom: &MemoryAtom, data: &[f64]) {
        let key = AtomKey::from_atom(atom);
        let embedding = SemanticEmbedding::from_data_and_annotation(data, atom);
        self.index.upsert(key, embedding);
    }

    pub fn index_memory_data_only(&mut self, atom: &MemoryAtom, data: &[f64]) {
        let key = AtomKey::from_atom(atom);
        let embedding = SemanticEmbedding::from_data(data);
        self.index.upsert(key, embedding);
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
            let results = self
                .index
                .search_knn(&SemanticEmbedding::from_annotation(atom), k);

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
        );
        index.upsert(
            AtomKey::from_atom(&atom_mid),
            SemanticEmbedding::from_data(&[1.0, 5.0, 3.0]),
        );
        index.upsert(
            AtomKey::from_atom(&atom_far),
            SemanticEmbedding::from_data(&[100.0, 200.0, 300.0]),
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
        index.upsert(key.clone(), SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]));
        assert_eq!(index.len(), 1);
        index.upsert(key.clone(), SemanticEmbedding::from_data(&[4.0, 5.0, 6.0]));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn knn_remove_works() {
        let mut index = EmbeddingIndex::new();
        let key = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        index.upsert(key.clone(), SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]));
        assert_eq!(index.len(), 1);
        index.remove(&key);
        assert_eq!(index.len(), 0);
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
            index.upsert(key.clone(), SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]));
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
    fn concept_find_for_embedding() {
        let mut extractor = ConceptExtractor::new()
            .with_similarity_threshold(0.5)
            .with_min_cluster_size(1);
        let mut index = EmbeddingIndex::new();
        let mut atoms_map: HashMap<AtomKey, MemoryAtom> = HashMap::new();
        let mut u = make_test_universe();

        let atom = make_atom_at(&mut u, 0, &[1.0, 2.0, 3.0]);
        let key = AtomKey::from_atom(&atom);
        index.upsert(key.clone(), SemanticEmbedding::from_data(&[1.0, 2.0, 3.0]));
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
    fn combined_embedding_weighted() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut u = make_test_universe();
        let atom = make_test_atom(&mut u, &data);
        let combined = SemanticEmbedding::from_data_and_annotation(&data, &atom);
        let norm: f64 = combined.vector.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-10,
            "combined embedding should be normalized"
        );
    }

    #[test]
    fn knowledge_graph_weight_metadata() {
        let a = AtomKey {
            vertices_basis: [[0; 7]; 4],
            vertices_even: [true; 4],
        };
        let b = AtomKey {
            vertices_basis: [[1; 7]; 4],
            vertices_even: [false; 4],
        };
        let rel = Relation::new(a.clone(), b.clone(), RelationType::Causes)
            .with_weight(0.75)
            .with_metadata("reason", "test");

        assert!((rel.weight - 0.75).abs() < 1e-10);
        assert_eq!(rel.metadata.get("reason"), Some(&"test".to_string()));
    }
}
