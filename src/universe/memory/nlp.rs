// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
// NLP utilities: text-to-embedding, synonym buckets, contradiction detection

use serde_json::json;

use crate::universe::coord::Coord7D;
use crate::universe::hebbian::HebbianMemory;
use crate::universe::memory::MemoryAtom;
use crate::universe::node::DarkUniverse;

pub fn text_to_embedding(text: &str, importance: f64) -> Vec<f64> {
    let dim = 28usize;
    let mut vec = vec![0.0f64; dim];

    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && w.len() > 1)
        .collect();

    let stop_words: &[&str] = &[
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
        "need", "must", "ought", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
        "into", "through", "during", "before", "after", "above", "below", "between", "out", "off",
        "over", "under", "again", "further", "then", "once", "and", "but", "or", "nor", "not",
        "so", "yet", "both", "either", "neither", "each", "every", "all", "any", "few", "more",
        "most", "other", "some", "such", "no", "only", "own", "same", "than", "too", "very",
        "just", "because", "if", "when", "where", "how", "what", "which", "who", "whom", "this",
        "that", "these", "those", "it", "its", "he", "she", "they", "them", "we", "you", "me",
        "my", "your", "his", "her", "our", "their",
    ];

    for word in &words {
        if stop_words.contains(word) {
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
