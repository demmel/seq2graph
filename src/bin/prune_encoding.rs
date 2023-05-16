use std::{
    collections::{hash_map::Entry, HashMap},
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;
use seq2graph::Encoding;

#[derive(Parser)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut encoding: Encoding<String> =
        serde_json::from_reader(BufReader::new(File::open(args.input)?))?;
    let mut tok2vec =
        encoding
            .tokens
            .iter()
            .enumerate()
            .fold(HashMap::new(), |mut tok2vec, (e, t)| {
                tok2vec.insert(t.clone(), e);
                tok2vec
            });

    loop {
        let (inbound, outbound) = encoding.encoding.windows(2).fold(
            (HashMap::new(), HashMap::new()),
            |(mut inbound, mut outbound), edge| {
                match inbound.entry(edge[1]) {
                    Entry::Occupied(mut e) => {
                        *e.get_mut() += 1;
                    }
                    Entry::Vacant(e) => {
                        e.insert(1);
                    }
                }
                match outbound.entry(edge[0]) {
                    Entry::Occupied(mut e) => {
                        *e.get_mut() += 1;
                    }
                    Entry::Vacant(e) => {
                        e.insert(1);
                    }
                }
                (inbound, outbound)
            },
        );

        let mut new_encoding = vec![];
        let mut pruned = 0;
        let mut skip = false;
        for edge in encoding.encoding.windows(2) {
            if skip {
                skip = false;
                continue;
            }
            let [cur, next] = edge else {panic!{"Should have 2 values in window"}};
            if inbound[next] == 1 && outbound[cur] == 1 {
                pruned += 1;
                skip = true;
                let token = encoding.tokens[*cur].clone() + &encoding.tokens[*next];
                println!(
                    "{} + {} -> {token}",
                    encoding.tokens[*cur], encoding.tokens[*next]
                );
                let encoding = match tok2vec.entry(token.clone()) {
                    Entry::Occupied(e) => *e.get(),
                    Entry::Vacant(e) => {
                        encoding.tokens.push(token);
                        let encoding = encoding.tokens.len() - 1;
                        e.insert(encoding);
                        encoding
                    }
                };
                new_encoding.push(encoding);
            } else {
                new_encoding.push(*cur);
            }
        }

        if pruned == 0 {
            break;
        }

        encoding.encoding = new_encoding;
    }

    if let Some(ref output) = args.output {
        serde_json::to_writer(
            BufWriter::new(
                File::create(output)
                    .with_context(|| format!("Failed to create output file: {output:?}"))?,
            ),
            &encoding,
        )
        .context("Failed to encode encoding to JSON")?;
    } else {
        let json = serde_json::to_string(&encoding).context("Failed to encode encoding to JSON")?;
        println!("{json}");
    }

    Ok(())
}
