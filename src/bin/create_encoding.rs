use std::{fs::File, io::BufWriter, path::PathBuf};

use anyhow::Context;
use clap::Parser;

use seq2graph::null_encoding;

#[derive(Parser)]
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let text = std::fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read contents {:?} to string", args.input))?;

    let encoding = null_encoding(&text);

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
