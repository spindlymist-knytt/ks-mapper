use std::collections::HashMap;

use petgraph::{
    prelude::*,
    unionfind::UnionFind,
    visit::{IntoNodeReferences, NodeIndexable}
};

use crate::{screen_map::ScreenMap, Position};
use super::{Partition, PartitionStrategy};

pub struct IslandsStrategy {
    pub max_size: (u64, u64),
    pub max_dist: u64,
}

impl PartitionStrategy for IslandsStrategy {
    fn partitions(&self, screens: &ScreenMap) -> Result<Vec<Partition>, anyhow::Error> {
        let positions = screens.iter_positions()
            .copied()
            .collect();
        let partition = Partition::new(positions);

        if is_partition_too_large(&partition, self.max_size) {
            Ok(partition_recursively(partition, self.max_size, self.max_dist))
        }
        else {
            Ok(vec![partition])
        }
    }
}

fn partition_recursively(partition: Partition, max_size: (u64, u64), max_dist: u64) -> Vec<Partition> {
    let mut partitions = Vec::new();


    let graph = partition_into_graph(partition, max_dist);
    for partition in graph_into_partitions(graph) {
        if is_partition_too_large(&partition, max_size) {
            let subpartitions = partition_recursively(partition, max_size, attenuate(max_dist));
            partitions.extend(subpartitions);
        }
        else {
            partitions.push(partition);
        }
    }

    partitions
}

fn is_partition_too_large(partition: &Partition, max_size: (u64, u64)) -> bool {
    let size = partition.bounds().size();

    size.0 > max_size.0
        || size.1 > max_size.1
}

fn attenuate(max_dist: u64) -> u64 {
    if max_dist > 5 {
        max_dist / 2
    }
    else {
        max_dist - 1
    }
}

fn partition_into_graph(partition: Partition, max_dist: u64) -> UnGraph<Position, u64> {
    let n_screens = partition.len();
    let mut graph = UnGraph::with_capacity(n_screens, n_screens);

    for pos in partition {
        let node = graph.add_node(pos);

        for other_node in graph.node_indices() {
            let dist = {
                let other_pos = graph[other_node];
                let dist_x = pos.0.abs_diff(other_pos.0);
                let dist_y = pos.1.abs_diff(other_pos.1);
                dist_x.saturating_add(dist_y)
            };

            if dist <= max_dist {
                graph.add_edge(node, other_node, dist);
            }
        }
    }

    graph
}

fn graph_into_partitions(graph: UnGraph<Position, u64>) -> Vec<Partition> {
    let mut vertex_sets = UnionFind::new(graph.node_bound());
    for edge in graph.edge_references() {
        vertex_sets.union(edge.source(), edge.target());
    }
    
    let mut partitions = HashMap::<NodeIndex, Vec<Position>>::new();
    for (node, pos) in graph.node_references() {
        let parent = vertex_sets.find(node);
        let partition = partitions.entry(parent)
            .or_insert_with(Vec::new);
        partition.push(*pos);
    }

    partitions.into_values()
        .map(Partition::new)
        .collect()
}
