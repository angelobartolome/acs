use std::collections::HashMap;

use crate::{Constraint, ConstraintType, ParameterManager};

struct UnionFind {
    parent: HashMap<String, String>,
    rank: HashMap<String, u32>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    fn ensure(&mut self, id: &str) {
        if !self.parent.contains_key(id) {
            self.parent.insert(id.to_string(), id.to_string());
            self.rank.insert(id.to_string(), 0);
        }
    }

    fn find(&mut self, id: &str) -> String {
        let parent = self.parent[id].clone();
        if parent == id {
            return id.to_string();
        }
        let root = self.find(&parent);
        self.parent.insert(id.to_string(), root.clone());
        root
    }

    fn union(&mut self, a: &str, b: &str) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        let rank_a = self.rank[&ra];
        let rank_b = self.rank[&rb];
        if rank_a < rank_b {
            self.parent.insert(ra, rb);
        } else if rank_a > rank_b {
            self.parent.insert(rb, ra);
        } else {
            self.parent.insert(rb, ra.clone());
            *self.rank.get_mut(&ra).unwrap() += 1;
        }
    }
}

pub struct ConnectedComponents {
    /// Each inner Vec holds constraint indices (into the original constraint slice) for one component.
    pub components: Vec<Vec<usize>>,
}

/// Partition constraints into connected components by shared entity IDs.
pub fn find_components(constraint_types: &[ConstraintType]) -> ConnectedComponents {
    let mut uf = UnionFind::new();

    for ct in constraint_types {
        let ids = ct.entity_ids();
        for id in &ids {
            uf.ensure(id);
        }
        if ids.len() >= 2 {
            for id in &ids[1..] {
                uf.union(ids[0], id);
            }
        }
    }

    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, ct) in constraint_types.iter().enumerate() {
        let ids = ct.entity_ids();
        let root = if ids.is_empty() {
            format!("__solo_{i}__")
        } else {
            uf.find(ids[0])
        };
        groups.entry(root).or_default().push(i);
    }

    ConnectedComponents {
        components: groups.into_values().collect(),
    }
}

/// Returns true if every constraint in this component has residual L-inf ≤ tolerance.
pub fn is_component_satisfied(
    indices: &[usize],
    constraints: &[Box<dyn Constraint>],
    param_manager: &ParameterManager,
    tolerance: f64,
) -> bool {
    indices
        .iter()
        .all(|&i| constraints[i].residual(param_manager).amax() <= tolerance)
}
