//
// Haymaker
//

use crate::derive::VarMap;
use crate::gui::Event;
use crate::recipe::Recipe;

use futures::prelude::*;
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    Direction,
};
use std::collections::HashSet;
use std::sync::{mpsc::Sender, Arc};
use std::thread;
use std::time::Duration;
use termion::event::Key;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug)]
struct Task(Duration);

pub type BuildGraph = StableGraph<Arc<Recipe>, ()>;

pub async fn build(
    emit: Sender<Event<Key>>,
    mut graph: BuildGraph,
    nthreads: usize,
) -> eyre::Result<()> {
    let (queue_send, queue_recv) = tokio::sync::mpsc::unbounded_channel();
    let mut results = UnboundedReceiverStream::new(queue_recv)
        .map(execute_task)
        .buffer_unordered(nthreads);

    let mut sent = HashSet::new();
    for node in graph.externals(Direction::Outgoing) {
        let recipe = &graph[node];
        queue_send.send((recipe.clone(), node))?;
        sent.insert(node);
    }

    while let Some(result) = results.next().await {
        let done = match result {
            Ok(done) => done,
            Err(err) => panic!("{}", err),
        };

        let nodes: Vec<_> = graph.neighbors_directed(done, Direction::Incoming).collect();
        graph.remove_node(done);
        emit.send(Event::State)?;

        for node in nodes {
            if sent.contains(&node) {
                continue;
            }

            if graph.neighbors_directed(node, Direction::Outgoing).next().is_none() {
                let recipe = &graph[node];
                queue_send.send((recipe.clone(), node))?;
                sent.insert(node);
            }
        }

        if graph.node_count() == 0 {
            break;
        }
    }

    Ok(())
}

async fn execute_task(task: (Arc<Recipe>, NodeIndex)) -> eyre::Result<NodeIndex> {
    let (recipe, node) = task;

    println!("Running node {}", node.index());
    recipe.print();

    let result = recipe.execute(&VarMap::new()).await;

    /*let status = tokio::process::Command::new("sleep")
    .arg(node.index().to_string())
    .status()
    .await?;*/

    /*if !status.success() {
            eyre::bail!("command failed: {}", status);
    }*/

    //println!("Done executing {:?}!", task);
    //let new_tasks = vec![Task(task.0 + Duration::from_secs(1)), Task(task.0 * 2)];
    Ok(node)
}
