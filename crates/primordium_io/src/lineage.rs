use petgraph::graph::{DiGraph, NodeIndex};
use primordium_data::Legend;
use std::collections::HashMap;
use uuid::Uuid;

/// A node in the Ancestry Tree representing an organism (living or dead).
pub struct AncestryNode {
    /// Unique identifier for the node.
    pub id: Uuid,
    /// Display name of the entity.
    pub name: String,
    /// Generational depth.
    pub generation: u32,
    /// Trophic level (0.0=Herbivore, 1.0=Carnivore).
    pub trophic_potential: f32,
    /// Number of offspring produced.
    pub offspring_count: u32,
    /// Whether the entity is currently part of the living population.
    pub is_alive: bool,
}

/// The "Tree of Life" graph representing macroevolutionary branching.
pub struct AncestryTree {
    /// Directed graph of ancestry nodes.
    pub graph: DiGraph<AncestryNode, ()>,
    id_map: HashMap<Uuid, NodeIndex>,
}

impl Default for AncestryTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AncestryTree {
    /// Creates an empty ancestry tree.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            id_map: HashMap::new(),
        }
    }

    /// Build tree from historical legends and current population.
    pub fn build(legends: &[Legend], living: &[primordium_data::Genotype]) -> Self {
        let mut tree = Self::new();

        // 1. Add all legends (Historical nodes)
        for l in legends {
            let node = AncestryNode {
                id: l.id,
                name: format!("L-{}", &l.id.to_string()[..4]),
                generation: l.generation,
                trophic_potential: 0.5,
                offspring_count: l.offspring_count,
                is_alive: false,
            };
            let idx = tree.graph.add_node(node);
            tree.id_map.insert(l.id, idx);
        }

        // 2. Add all living genotypes
        for g in living {
            if tree.id_map.contains_key(&g.lineage_id) {
                continue;
            }

            let node = AncestryNode {
                id: g.lineage_id,
                name: format!("D-{}", &g.lineage_id.to_string()[..4]),
                generation: 1, // Approximation as we don't have metabolic info here
                trophic_potential: g.trophic_potential,
                offspring_count: 0,
                is_alive: true,
            };
            let idx = tree.graph.add_node(node);
            tree.id_map.insert(g.lineage_id, idx);
        }

        // 3. Connect parents to children
        for l in legends {
            if let Some(p_id) = l.parent_id {
                if let (Some(&p_idx), Some(&c_idx)) =
                    (tree.id_map.get(&p_id), tree.id_map.get(&l.id))
                {
                    tree.graph.add_edge(p_idx, c_idx, ());
                }
            }
        }

        // Connect legends to living genotypes if possible (by lineage_id)
        // This is an approximation since living genotypes are aggregated by lineage

        tree
    }

    /// Export the tree to Graphviz DOT format.
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph TreeOfLife {\n");
        dot.push_str("  node [shape=box, style=filled, fontname=\"Arial\"];\n");

        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            let color = if node.is_alive {
                "#e1f5fe" // Light blue
            } else {
                "#eeeeee" // Gray
            };

            let trophic_color = if node.trophic_potential < 0.3 {
                "green"
            } else if node.trophic_potential > 0.7 {
                "red"
            } else {
                "orange"
            };

            dot.push_str(&format!(
                "  \"{}\" [label=\"{} (Gen {})\\nOffspring: {}\", fillcolor=\"{}\", color=\"{}\", penwidth=2];\n",
                node.id, node.name, node.generation, node.offspring_count, color, trophic_color
            ));
        }

        for edge in self.graph.edge_indices() {
            if let Some((from, to)) = self.graph.edge_endpoints(edge) {
                dot.push_str(&format!(
                    " \"{}\" -> \"{}\";\n",
                    self.graph[from].id, self.graph[to].id
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }
}
