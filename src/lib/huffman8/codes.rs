use crate::constants::{FESTError, HUF_LCHAR, HUF_NEXT, HUF_RCHAR};
use crate::huffman8::Node;
use std::fmt::Debug;
use std::slice::SliceIndex;
use std::{cell::RefCell, rc::Rc};

pub struct Codes {
    pub code_tree: Vec<u8>,
    code_mask: Vec<u8>,
}

impl Codes {
    pub fn create_code(num_leafs: usize) -> Result<Self, FESTError> {
        println!("CREATE_CODE");
        let max_nodes = ((((num_leafs) - 1) | 1) + 1) * 2;

        let mut code_tree = vec![0u8; max_nodes];
        let code_mask = vec![0u8; max_nodes];

        unsafe {
            *code_tree.get_unchecked_mut(0) = ((num_leafs as u8) - 1) | 1;
        }

        // *code_mask.get_mut(0)? = 0; // NOTE: In this case, the array is declared with all zeroes, doesn't make sense here

        Ok(Codes {
            code_tree,
            code_mask,
        })
    }

    fn get_code_tree(&self, index: usize) -> Result<&u8, FESTError> {
        let code_tree_length = self.code_tree.len();
        self.code_tree
            .get(index)
            .ok_or(FESTError::CodeTreeIOOB(index.to_string(), code_tree_length))
    }
    fn get_code_mask(&self, index: usize) -> Result<&u8, FESTError> {
        let code_mask_length = self.code_mask.len();

        self.code_mask
            .get(index)
            .ok_or(FESTError::CodeMaskIOOB(index.to_string(), code_mask_length))
    }

    fn mut_code_tree<I>(&mut self, index: I) -> Result<&mut I::Output, FESTError>
    where
        I: SliceIndex<[u8]> + Debug,
    {
        let code_tree_length = self.code_tree.len();
        let index_str = format!("{:#?}", index);

        self.code_tree
            .get_mut(index)
            .ok_or(FESTError::CodeTreeIOOB(index_str, code_tree_length))
    }
    fn mut_code_mask<I>(&mut self, index: I) -> Result<&mut I::Output, FESTError>
    where
        I: SliceIndex<[u8]> + Debug,
    {
        let code_mask_length = self.code_mask.len();
        let index_str = format!("{:#?}", index);

        self.code_mask
            .get_mut(index)
            .ok_or(FESTError::CodeMaskIOOB(index_str, code_mask_length))
    }

    pub fn create_code_branch(
        &mut self,
        root: &Rc<RefCell<Node>>,
        p: usize,
        q: usize,
    ) -> Result<usize, FESTError> {
        let mut q = q;

        if root.borrow().leafs <= (HUF_NEXT as usize + 1) {
            let mut stack: Vec<Rc<RefCell<Node>>> = vec![Rc::clone(root)];

            let mut s = 0;

            // NOTE: "r" was basically equal to "stack.len()" all the time, so it was eliminated
            while s < stack.len() {
                let node = unsafe { Rc::clone(stack.get_unchecked(s)) }; // NOTE: "s" is guaranteed to be less than the stack's length
                let node = node.borrow();

                s += 1;

                if node.leafs == 1 {
                    let index = if s == 1 { p } else { q };
                    *self.mut_code_tree(index)? = node.symbol as u8;
                    *self.mut_code_mask(index)? = 0xFF;

                    if s != 1 {
                        q += 1;
                    }
                } else {
                    let mut mask = 0;

                    // TODO: Use some sort of trait or something to fix this, since if leafs is not 1 then both 'left_son' and 'right_son' are guaranteed to not be None
                    if let (Some(left_son), Some(right_son)) = (&node.left_son, &node.right_son) {
                        if left_son.borrow().leafs == 1 {
                            mask |= HUF_LCHAR;
                        };

                        if right_son.borrow().leafs == 1 {
                            mask |= HUF_RCHAR;
                        };

                        let index = if s == 1 { p } else { q };

                        *self.mut_code_tree(index)? = ((stack.len() - s) >> 1) as u8;
                        *self.mut_code_mask(index)? = mask;

                        if s != 1 {
                            q += 1;
                        };

                        stack.push(Rc::clone(left_son));
                        stack.push(Rc::clone(right_son));
                    } else {
                        unreachable!(
                            "Node with .leafs > 1 has no left_son and right_son...but why?"
                        );
                    }
                };
            }
        } else {
            let mut mask = 0;

            let root = root.borrow();

            if let (Some(left_son), Some(right_son)) = (&root.left_son, &root.right_son) {
                if left_son.borrow().leafs == 1 {
                    mask |= HUF_LCHAR;
                }

                if right_son.borrow().leafs == 1 {
                    mask |= HUF_RCHAR;
                }

                *self.mut_code_tree(p)? = 0;
                *self.mut_code_mask(p)? = mask;

                if left_son.borrow().leafs <= right_son.borrow().leafs {
                    let left_leaves = self.create_code_branch(left_son, q, q + 2)?;
                    self.create_code_branch(right_son, q + 1, q + (left_leaves << 1))?;

                    *self.mut_code_tree(q + 1)? = (left_leaves as u8) - 1;
                } else {
                    let right_leaves = self.create_code_branch(right_son, q + 1, q + 2)?;
                    self.create_code_branch(left_son, q, q + (right_leaves << 1))?;
                    *self.mut_code_tree(q)? = (right_leaves as u8) - 1;
                };
            } else {
                unreachable!("root node should have left_son and right_son, but it doesn't, why?");
            }
        }

        return Ok(root.borrow().leafs);
    }

    pub fn update_code(&mut self) -> Result<(), FESTError> {
        println!("UPDATE_CODE");
        let max_code_index = ((*self.get_code_tree(0)? as usize) + 1) << 1;

        let mut code_index = 1;

        while code_index < max_code_index {
            if (*self.get_code_mask(code_index)? != 0xFF_u8)
                && *self.get_code_tree(code_index)? > HUF_NEXT
            {
                let increment;
                if ((code_index & 1) != 0) && (*self.get_code_tree(code_index - 1)? == HUF_NEXT) {
                    code_index -= 1;
                    increment = 1;
                } else if ((code_index & 1) == 0)
                    && (*self.get_code_tree(code_index + 1)? == HUF_NEXT)
                {
                    code_index += 1;
                    increment = 1;
                } else {
                    increment = self.get_code_tree(code_index)? - HUF_NEXT;
                }

                let n1 = (((code_index as u8) >> 1) + 1) + self.get_code_tree(code_index)?;
                let n0 = n1 - increment;

                let l1 = (n1 as usize) << 1;
                let l0 = (n0 as usize) << 1;

                let tmp0 =
                    u16::from_le_bytes([*self.get_code_tree(l1)?, *self.get_code_tree(l1 + 1)?]);
                let tmp1 =
                    u16::from_le_bytes([*self.get_code_mask(l1)?, *self.get_code_mask(l1 + 1)?]);

                // TODO: I refuse to belive I have to do this in Rust, I'm sure there's a better way I just haven't realized yet
                let mut j = l1;
                while j > l0 {
                    *self.mut_code_tree(j)? = *self.get_code_tree(j - 2)?;
                    *self.mut_code_tree(j + 1)? = *self.get_code_tree(j - 1)?;

                    *self.mut_code_mask(j)? = *self.get_code_mask(j - 2)?;
                    *self.mut_code_mask(j + 1)? = *self.get_code_mask(j - 1)?;

                    j -= 2;
                }

                (*self.mut_code_tree(l0..l0 + 2)?).copy_from_slice(&tmp0.to_le_bytes());
                (*self.mut_code_mask(l0..l0 + 2)?).copy_from_slice(&tmp1.to_le_bytes());

                *self.mut_code_tree(code_index)? -= increment;

                for j in (code_index + 1)..l0 {
                    if *self.get_code_mask(j)? != 0xFF {
                        let k = ((j as u8) >> 1) + 1 + *self.get_code_tree(j)?;

                        if n0 <= k && k < n1 {
                            *self.mut_code_tree(j)? += 1;
                        }
                    }
                }

                if *self.get_code_mask(l0)? != 0xFF {
                    *self.mut_code_tree(l0)? += increment;
                }

                if *self.get_code_mask(l0 + 1)? != 0xFF {
                    *self.mut_code_tree(l0 + 1)? += increment;
                }

                for j in (l0 + 2)..(l1 + 2) {
                    if *self.get_code_mask(j)? != 0xFF {
                        let k = ((j as u8) >> 1) + 1 + *self.get_code_tree(j)?;

                        if k > n1 {
                            *self.mut_code_tree(j)? -= 1;
                        }
                    }
                }

                code_index = (code_index | 1) - 2;
            }

            code_index += 1;
        }

        for i in 1..(((*self.get_code_tree(0)? as usize) + 1) << 1) {
            if *self.get_code_mask(i)? != 0xFF {
                *self.mut_code_tree(i)? |= *self.get_code_mask(i)?
            }
        }

        Ok(())
    }
}
