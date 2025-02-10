use crate::network::Network;
use petgraph::algo::astar;
use wg_2024::network::NodeId;

impl Network {
    //finds out if nodes that were reachable before are now not reachable anymore
    pub(super) fn update_reachable_paths(&mut self) {
        //gets all known leafs' ids
        let mut leafs = Vec::new();
        for leaf in self.paths_to_leafs.keys() {
            leafs.push(*leaf);
        }

        for leaf in leafs {
            //if it used to be a valid path to the leaf
            if let Some(Some(_)) = self.paths_to_leafs.get(&leaf) {
                //try to see if there's still a valid path to it
                let path = astar(
                    &self.topology,
                    self.id,
                    |finish| finish == leaf,
                    |_| 1,
                    |_| 0,
                );
                if let Some((_, path)) = path {
                    //could be different so if there's still one add it
                    self.paths_to_leafs.insert(leaf, Some(path));
                    //println!("updated path to {}", leaf);
                } else {
                    //remove path from leaf and flood
                    self.paths_to_leafs.insert(leaf, None);
                    //println!("flood: removed route to {}", leaf);
                    self.initiate_flood();
                }
            }
        }
    }

    //finds out if nodes that were unreachable before are now reachable
    pub(super) fn update_unreachable_paths(&mut self) {
        //gets all known leafs' ids
        let mut leafs = Vec::new();
        for leaf in self.paths_to_leafs.keys() {
            leafs.push(*leaf);
        }

        for leaf in leafs {
            //if there's no known valid path to the leaf
            if let Some(None) = self.paths_to_leafs.get(&leaf) {
                //println!("trying to discover new paths to {}", leaf);
                //try to find one
                let path = astar(
                    &self.topology,
                    self.id,
                    |finish| finish == leaf,
                    |_| 1,
                    |_| 0,
                );
                //if found add path to known paths and try sending any queued messages for that leaf
                if let Some((_, path)) = path {
                    self.paths_to_leafs.insert(leaf, Some(path.clone()));
                    //println!("new path to {}: {:?}", leaf, path);
                    self.check_queued(leaf);
                }
            }
        }
    }
    //adds partial path to topology
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
            //println!("topology: {:?}", self.topology);
        }

        Ok(())
    }
}
