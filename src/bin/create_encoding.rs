use std::collections::{hash_map::Entry, HashMap};
use std::fs::File;
use std::hash::Hash;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;

use seq2graph::Encoding;

#[derive(Parser)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let text = std::fs::read_to_string(args.input)
        .expect("should be able to read contents of file to string");

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

    if let Some(ref output) = args.output {
        serde_json::to_writer(BufWriter::new(File::create(output).unwrap()), &encoding).unwrap();
    } else {
        let json = serde_json::to_string(&encoding).unwrap();
        println!("{json}");
    }
}

fn find_max_compressable_seq<T>(seq: &[T]) -> Option<&[T]>
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
