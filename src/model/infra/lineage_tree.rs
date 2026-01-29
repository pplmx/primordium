use crate::model::history::Legend;
use petgraph::graph::{DiGraph, NodeIndex};
use primordium_data::Entity;
use std::collections::HashMap;
use uuid::Uuid;

/// A node in the Ancestry Tree representing an organism (living or dead).
pub struct AncestryNode {
    pub id: Uuid,
    pub name: String,
    pub generation: u32,
    pub trophic_potential: f32,
    pub offspring_count: u32,
    pub is_alive: bool,
}

/// The "Tree of Life" graph representing macroevolutionary branching.
pub struct AncestryTree {
    pub graph: DiGraph<AncestryNode, ()>,
    id_map: HashMap<Uuid, NodeIndex>,
}

impl Default for AncestryTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AncestryTree {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            id_map: HashMap::new(),
        }
    }

    /// Build tree from historical legends and current population.
    pub fn build(legends: &[Legend], living: &[Entity]) -> Self {
        let mut tree = Self::new();

        // 1. Add all legends (Historical nodes)
        for l in legends {
            let node = AncestryNode {
                id: l.id,
                name: format!("L-{}", &l.id.to_string()[..4]),
                generation: l.generation,
                trophic_potential: 0.5, // Default for old legends without this field
                offspring_count: l.offspring_count,
                is_alive: false,
            };
            let idx = tree.graph.add_node(node);
            tree.id_map.insert(l.id, idx);
        }

        // 2. Add all living entities
        for e in living {
            if tree.id_map.contains_key(&e.id) {
                continue;
            } // Already in (e.g. archived while alive)

            let node = AncestryNode {
                id: e.id,
                name: e.name(),
                generation: e.metabolism.generation,
                trophic_potential: e.metabolism.trophic_potential,
                offspring_count: e.metabolism.offspring_count,
                is_alive: true,
            };
            let idx = tree.graph.add_node(node);
            tree.id_map.insert(e.id, idx);
        }

        // 3. Connect parents to children
        // Loop through legends
        for l in legends {
            if let Some(p_id) = l.parent_id {
                if let (Some(&p_idx), Some(&c_idx)) =
                    (tree.id_map.get(&p_id), tree.id_map.get(&l.id))
                {
                    tree.graph.add_edge(p_idx, c_idx, ());
                }
            }
        }

        // Loop through living
        for e in living {
            if let Some(p_id) = e.parent_id {
                if let (Some(&p_idx), Some(&c_idx)) =
                    (tree.id_map.get(&p_id), tree.id_map.get(&e.id))
                {
                    tree.graph.add_edge(p_idx, c_idx, ());
                }
            }
        }

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
            let (from, to) = self.graph.edge_endpoints(edge).unwrap();
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\";\n",
                self.graph[from].id, self.graph[to].id
            ));
        }

        dot.push_str("}\n");
        dot
    }
}
