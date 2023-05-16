use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use seq2graph::{create_encoding_graph, Encoding};

#[derive(Parser)]
struct Args {
    encoding_file: PathBuf,
    dot_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let encoding: Encoding<String> =
        serde_json::from_reader(BufReader::new(File::open(args.encoding_file)?))?;

    let graph = create_encoding_graph(&encoding)?;

    let mut writer = BufWriter::new(File::create(args.dot_file)?);
    writeln!(writer, "{graph}")?;

    Ok(())
}
