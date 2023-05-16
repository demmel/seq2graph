use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use seq2graph::Encoding;
use tabbycat::{attributes::label, AttrList, Edge, GraphBuilder, GraphType, Identity, StmtList};

#[derive(Parser)]
struct Args {
    encoding_file: PathBuf,
    dot_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let encoding: Encoding<String> =
        serde_json::from_reader(BufReader::new(File::open(args.encoding_file)?))?;
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
                    Some(AttrList::default().add_pair(label(node))),
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
        .build()?;

    let mut writer = BufWriter::new(File::create(args.dot_file)?);
    writeln!(writer, "{graph}")?;

    Ok(())
}
