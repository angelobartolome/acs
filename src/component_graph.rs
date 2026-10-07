//! Splits a sketch into Components: groups of constraints that share
//! entities (directly or through others), each solved on its own small
//! system.

use std::collections::HashMap;

use crate::Constraint;
use crate::constraints::reads;

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

/// The Components of a set of constraints.
pub struct ConnectedComponents {
    /// Each inner Vec holds constraint indices (into the original constraint slice) for one component.
    pub components: Vec<Vec<usize>>,
}

/// Partition constraints into connected components by the entities they read,
/// Guides included: a copy must be solved together with whatever moves its
/// Guide, so it follows.
pub fn find_components(constraints: &[Box<dyn Constraint>]) -> ConnectedComponents {
    find_components_of(constraints.iter().map(|c| c.as_ref()))
}

/// [`find_components`] over any sequence of constraints; indices are
/// positions in that sequence.
pub(crate) fn find_components_of<'c>(constraints: impl Iterator<Item = &'c dyn Constraint>) -> ConnectedComponents {
    let entity_ids: Vec<Vec<String>> = constraints
        .map(|c| {
            let mut ids: Vec<String> =
                reads(c).iter().map(|p| p.entity_id().to_string()).collect();
            ids.dedup();
            ids
        })
        .collect();

    let mut uf = UnionFind::new();
    for ids in &entity_ids {
        for id in ids {
            uf.ensure(id);
        }
        for id in ids.iter().skip(1) {
            uf.union(&ids[0], id);
        }
    }

    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, ids) in entity_ids.iter().enumerate() {
        let root = match ids.first() {
            Some(id) => uf.find(id),
            None => format!("__solo_{i}__"),
        };
        groups.entry(root).or_default().push(i);
    }

    ConnectedComponents {
        components: groups.into_values().collect(),
    }
}
