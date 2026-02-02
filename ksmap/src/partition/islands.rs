use std::ops::RangeInclusive;

use petgraph::{
    prelude::*,
    unionfind::UnionFind,
    visit::{IntoNodeReferences, NodeIndexable}
};
use libks::ScreenCoord;
use rustc_hash::FxHashMap;

use crate::{partition::{grid, merge_redundant_partitions}, screen_map::ScreenMap};
use super::{Partition, Partitioner};

pub struct IslandsPartitioner {
    pub max_size: (u64, u64),
    pub gap: RangeInclusive<u64>,
    pub force: bool,
}

impl Default for IslandsPartitioner {
    fn default() -> Self {
        Self {
            max_size: (48000, 48000),
            gap: 1..=20,
            force: false,
        }
    }
}

impl Partitioner for IslandsPartitioner {
    fn partitions(&self, screens: &ScreenMap) -> Vec<Partition> {
        let positions = screens.iter_positions()
            .copied()
            .collect();
        let partition = Partition::new(positions);
        
        if !self.force
            && !is_partition_too_large(&partition, self.max_size)
        {
            return vec![partition];
        }
        
        let mut partitions = partition_recursively(partition, self.max_size, *self.gap.start(), *self.gap.end());
        merge_redundant_partitions(&mut partitions);
        partitions
    }
}

fn partition_recursively(partition: Partition, max_size: (u64, u64), min_gap: u64, max_gap: u64) -> Vec<Partition> {
    let mut partitions = Vec::new();

    let graph = partition_into_graph(partition, max_gap);
    for partition in graph_into_partitions(graph) {
        let is_too_large = is_partition_too_large(&partition, max_size);
        if is_too_large && max_gap > min_gap {
            // Reduce the gap and try again
            let new_max_gap = attenuate_max_gap(min_gap, max_gap);
            let subpartitions = partition_recursively(partition, max_size, min_gap, new_max_gap);
            partitions.extend(subpartitions);
        }
        else if is_too_large && max_gap == min_gap {
            // We can't reduce the gap anymore, so switch to the grid strategy
            let (rows, cols) = grid::calc_grid_dimensions(&partition.bounds, max_size);
            let positions = partition.positions.iter();
            let subpartitions = grid::partitions_from_grid(positions, &partition.bounds, rows, cols);
            partitions.extend(subpartitions);
        }
        else {
            partitions.push(partition);
        }
    }

    partitions
}

fn is_partition_too_large(partition: &Partition, max_size: (u64, u64)) -> bool {
    partition.bounds.width() > max_size.0
        || partition.bounds.height() > max_size.1
}

fn attenuate_max_gap(min_gap: u64, max_gap: u64) -> u64 {
    let diff = max_gap - min_gap;
    if diff > 5 {
        min_gap + diff / 2
    }
    else {
        u64::max(min_gap, max_gap.saturating_sub(1))
    }
}

fn partition_into_graph(partition: Partition, max_gap: u64) -> UnGraph<ScreenCoord, u64> {
    let n_screens = partition.len();
    let mut graph = UnGraph::with_capacity(n_screens, n_screens);

    for pos in partition {
        let node = graph.add_node(pos);

        for other_node in graph.node_indices() {
            let dist = {
                let other_pos = &graph[other_node];
                let dist_x = pos.0.abs_diff(other_pos.0) as u64;
                let dist_y = pos.1.abs_diff(other_pos.1) as u64;
                dist_x.saturating_add(dist_y)
            };

            if dist <= max_gap {
                graph.add_edge(node, other_node, dist);
            }
        }
    }

    graph
}

fn graph_into_partitions(graph: UnGraph<ScreenCoord, u64>) -> Vec<Partition> {
    let mut vertex_sets = UnionFind::new(graph.node_bound());
    for edge in graph.edge_references() {
        vertex_sets.union(edge.source(), edge.target());
    }
    
    let mut partitions = FxHashMap::<NodeIndex, Vec<ScreenCoord>>::default();
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
