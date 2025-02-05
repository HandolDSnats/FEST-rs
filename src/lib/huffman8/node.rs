use crate::constants::{FESTError, HUF_LNODE, HUF_RNODE};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub symbol: usize,
    pub weight: usize,
    pub leafs: usize,
    pub parent: Option<Rc<RefCell<Node>>>,
    pub left_son: Option<Rc<RefCell<Node>>>,
    pub right_son: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(symbol: usize, weight: usize, leafs: usize) -> Self {
        Node {
            symbol,
            weight,
            leafs,
            parent: None,
            left_son: None,
            right_son: None,
        }
    }

    pub fn get_scode(&self) -> Vec<u8> {
        if let Some(parent) = self.parent.as_ref() {
            let scodes = parent.borrow().get_scode();
            let scode = if parent.borrow().left_son.as_ref().unwrap().borrow().eq(self) {
                HUF_LNODE
            } else {
                HUF_RNODE
            };

            let mut result = vec![scode];
            result.extend(scodes);
            return result;
        }

        vec![]
    }

    pub fn create_tree(
        freqs: &[usize],
        num_leafs: usize,
    ) -> Result<Vec<Rc<RefCell<Node>>>, FESTError> {
        println!("CREATE_TREE");
        // NOTE: When creating the tree, this means that "tree.len()" on first initialization is exactly the same as "num_leafs", so this may be removable
        let mut tree: Vec<Rc<RefCell<Node>>> = freqs
            .iter()
            .enumerate()
            .filter(|&(_, &freq)| freq > 0)
            .map(|(i, &freq)| Rc::new(RefCell::new(Node::new(i, freq, 1))))
            .collect(); // NOTE: "num_node" was technically always the length of "tree" if we only accounted for how many Nodes were actually there in the array, so it can be ommited

        // NOTE: Once this loop ends, since we're appending to "tree", then "tree.len()" is increasing one by one, so by the end of the loop, "tree.len()" is equal to the original "num_nodes", which will be useful later since there will be no need for it to be it's own variable
        while tree.len() < (2 * num_leafs - 1) {
            // NOTE: The entire loop was just to find the two nodes with the lowest weights, though "tree" must remain in the same order
            let mut sorted_tree = tree
                .iter()
                .filter(|node| node.borrow().parent.is_none())
                .collect::<Vec<_>>();
            sorted_tree.sort_by_key(|node| node.borrow().weight);

            let left_node = sorted_tree
                .first()
                .ok_or(FESTError::NodeNotFound("Left Node".to_string()))?;
            let right_node = sorted_tree
                .get(1)
                .ok_or(FESTError::NodeNotFound("Right Node".to_string()))?;

            let node = Rc::new(RefCell::new(Node {
                symbol: 0xFF + tree.len() - num_leafs + 1,
                weight: left_node.borrow().weight + right_node.borrow().weight,
                leafs: left_node.borrow().leafs + right_node.borrow().leafs,
                parent: None,
                left_son: Some(Rc::clone(left_node)),
                right_son: Some(Rc::clone(right_node)),
            }));

            left_node.borrow_mut().parent = Some(Rc::clone(&node));
            right_node.borrow_mut().parent = Some(Rc::clone(&node));

            tree.push(node);
        }

        Ok(tree)
    }
}
