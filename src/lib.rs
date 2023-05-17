use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::Display,
    hash::Hash,
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tabbycat::{
    attributes::label, AttrList, Edge, Graph, GraphBuilder, GraphType, Identity, StmtList,
};

#[derive(Serialize, Deserialize)]
pub struct Encoding<T> {
    pub encoding: Vec<usize>,
    pub tokens: Vec<T>,
}

pub fn null_encoding(text: &str) -> Encoding<String> {
    let mut encoding = Encoding {
        encoding: vec![],
        tokens: vec![],
    };

    let mut tok2enc = HashMap::new();
    for c in text.chars() {
        let s = c.to_string();
        let e = match tok2enc.entry(s.clone()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                encoding.tokens.push(s);
                let encoding = encoding.tokens.len() - 1;
                e.insert(encoding);
                encoding
            }
        };
        encoding.encoding.push(e);
    }

    encoding
}

pub fn encode_with_iterative_run_compression(text: &str) -> Encoding<String> {
    let mut encoding = null_encoding(text);

    while let Some(to_encode) = find_max_compressable_seq(&encoding.encoding) {
        let to_encode_indices: Vec<_> = encoding
            .encoding
            .windows(to_encode.len())
            .enumerate()
            .filter(|(_, v)| *v == to_encode)
            .map(|(i, _)| i)
            .collect();

        encoding.tokens.push(
            to_encode
                .iter()
                .flat_map(|i| encoding.tokens[*i].chars())
                .collect(),
        );
        println!("{}", encoding.tokens.last().unwrap());
        let e = encoding.tokens.len() - 1;

        let mut new_encoding = Vec::with_capacity(encoding.encoding.len());
        let mut prev = 0;
        for i in to_encode_indices {
            if i < prev {
                continue;
            }
            new_encoding.extend_from_slice(&encoding.encoding[prev..i]);
            new_encoding.push(e);
            prev = i + to_encode.len();
        }
        new_encoding.extend_from_slice(&encoding.encoding[prev..]);
        println!(
            "{} -> {} ({})",
            encoding.encoding.len(),
            new_encoding.len(),
            encoding.encoding.len() - new_encoding.len()
        );
        encoding.encoding = new_encoding;
    }

    encoding
}

pub fn find_max_compressable_seq<T>(seq: &[T]) -> Option<&[T]>
where
    T: Eq + Hash,
{
    let mut tokens = HashMap::new();
    let max_len = seq.len() / 2;

    for l in 2..max_len {
        let mut found = false;

        let max_start = seq.len() - l;
        for start in 0..=max_start {
            let end = start + l;
            let snippet = &seq[start..end];
            match tokens.entry(snippet) {
                Entry::Occupied(mut e) => {
                    let (count, last) = e.get_mut();
                    if *last < start - l {
                        *count += 1;
                        *last = start;
                    }
                    found = true;
                }
                Entry::Vacant(e) => {
                    e.insert((1, start));
                }
            }
        }
        if !found {
            break;
        }
    }

    tokens
        .into_iter()
        .filter(|(_, (c, _))| *c > 1)
        .max_by_key(|(tok, (count, _))| tok.len() * count)
        .map(|(t, _)| t)
}

pub fn create_encoding_graph<T>(encoding: &Encoding<T>) -> anyhow::Result<Graph>
where
    T: Display,
{
    let seen: HashSet<_> = encoding.encoding.iter().cloned().collect();

    let graph = GraphBuilder::default()
        .graph_type(GraphType::DiGraph)
        .strict(true)
        .id(Identity::id("G").unwrap())
        .stmts({
            let mut stmts = StmtList::new();
            for (id, node) in encoding.tokens.iter().enumerate() {
                if !seen.contains(&id) {
                    continue;
                }
                stmts = stmts.add_node(
                    Identity::Usize(id),
                    None,
                    Some(AttrList::default().add_pair(label(format!("{node}")))),
                );
            }

            for edge in encoding.encoding.windows(2) {
                stmts = stmts.add_edge(
                    Edge::head_node(Identity::Usize(edge[0]), None)
                        .arrow_to_node(Identity::Usize(edge[1]), None),
                );
            }

            stmts
        })
        .build()
        .map_err(|e| anyhow!(e))?;

    Ok(graph)
}
