use crate::network::Network;
use petgraph::algo::astar;
use wg_2024::network::NodeId;

impl Network {
    pub(super) fn update_reachable_paths(&mut self) {
        let mut leafs = Vec::new();
        for leaf in self.paths_to_leafs.keys() {
            leafs.push(*leaf);
        }

        for leaf in leafs {
            if self.paths_to_leafs.get(&leaf).is_some() {
                let path = astar(
                    &self.topology,
                    self.id,
                    |finish| finish == leaf,
                    |_| 1,
                    |_| 0,
                );
                if let Some((_, path)) = path {
                    self.paths_to_leafs.insert(leaf, Some(path));
                } else {
                    self.paths_to_leafs.insert(leaf, None);
                }
            }
        }
    }

    pub(super) fn update_unreachable_paths(&mut self) {
        let mut leafs = Vec::new();
        for leaf in self.paths_to_leafs.keys() {
            leafs.push(*leaf);
        }

        for leaf in leafs {
            if self.paths_to_leafs.get(&leaf).is_none() {
                let path = astar(
                    &self.topology,
                    self.id,
                    |finish| finish == leaf,
                    |_| 1,
                    |_| 0,
                );
                if let Some((_, path)) = path {
                    self.paths_to_leafs.insert(leaf, Some(path));
                    self.check_queued(leaf);
                }
            }
        }
    }
    pub(super) fn add_path(&mut self, path: &[NodeId], is_last_leaf: bool) -> Result<(), String> {
        if Some(self.id) != path.first().copied() {
            return Err("Path does not start with this node".to_string());
        }

        let windows = path.windows(2);
        let last_index = windows.len() - 1;

        for (i, window) in windows.enumerate() {
            let a = window[0];
            let b = window[1];

            self.topology.add_edge(a, b, 1);
            if i != 0 && !(is_last_leaf && i == last_index) {
                self.topology.add_edge(b, a, 1);
            }
        }

        Ok(())
    }
}
