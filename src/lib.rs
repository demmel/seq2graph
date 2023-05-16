use std::{collections::HashSet, fmt::Display};

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
