use crate::constants::{FESTError, CMD_CODE, HUF_MASK, HUF_MASK4, HUF_SHIFT};
use crate::utils::{from_uint32, to_uint32};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Work {
    pub works: HashMap<usize, (usize, Vec<u8>)>,
}

impl Work {
    pub fn create_code_works(tree_data: Vec<(usize, Vec<u8>)>) -> Result<Work, FESTError> {
        println!("CREATE_CODE_WORKS");
        // NOTE: While doing some testing, the behavior of "codes" (Renamed to "works" here) is basically a dictionary, as the indices are the symbols (AKA indexes from "freqs") and the value is the "code" struct (Renamed to "work" here)
        let mut works: HashMap<usize, (usize, Vec<u8>)> = HashMap::with_capacity(tree_data.len());

        // NOTE: "scode" follows "nbits", which starts at 0, so "scodes" just gets appended and "nbits" reflects its size
        for (symbol, scode) in tree_data {
            let mut scode = scode.to_vec();

            let maxbytes = (scode.len() + 7) >> 3;
            let nbits = scode.len();
            let mut code_work = vec![0u8; maxbytes];

            let mut mask = HUF_MASK;
            let mut j = 0;

            while let Some(scode_last) = scode.pop() {
                if scode_last != 0 {
                    *code_work.get_mut(j).ok_or(FESTError::CodeWorkNotFound(j))? |= mask
                }

                mask >>= HUF_SHIFT;

                if mask == 0 {
                    mask = HUF_MASK;
                    j += 1
                }
            }

            works.insert(symbol, (nbits, code_work));
        }

        Ok(Work { works })
    }

    pub fn process_data(self, data: &[u8], code_tree: &[u8]) -> Result<Vec<u8>, FESTError> {
        println!("PROCESS_DATA");
        // NOTE: These operations done to "pbuf" are actually be appends when considering only the written data
        let mut pbuf: Vec<u8> = vec![]; // NOTE: Got rid of "pak_pos", for similar reasons to "num_nodes" but now for pbuf
        pbuf.extend(from_uint32(CMD_CODE | ((data.len() as u32) << 8)));

        pbuf.extend(code_tree); // NOTE: "len" is just the length of "code_tree", for reference look at how "max_nodes" in "Codes::create_code" is calculated

        let mut mask4 = 0u32;
        let mut data = data.to_vec();
        data.reverse();

        while let Some(ch) = data.pop() {
            let ch = ch as usize;

            let (mut nbits, code_work) = self
                .works
                .get(&(ch & 0xFF))
                .ok_or(FESTError::WorkNotFound(ch))?;
            let mut index = 0usize;
            let mut mask = HUF_MASK;

            while nbits > 0 {
                mask4 >>= HUF_SHIFT;

                if mask4 == 0 {
                    mask4 = HUF_MASK4;
                    pbuf.extend(from_uint32(0));
                }

                if (*code_work
                    .get(index)
                    .ok_or(FESTError::CodeWorkNotFound(index))?
                    & mask)
                    != 0
                {
                    let length = pbuf.len();
                    let current = to_uint32(&pbuf, length - 4)?;

                    let val = unsafe { pbuf.get_unchecked_mut((length - 4)..length) };
                    (*val).copy_from_slice(&from_uint32(current | mask4))
                }

                mask >>= HUF_SHIFT;
                if mask == 0 {
                    mask = HUF_MASK;
                    index += 1;
                }

                nbits -= 1;
            }
        }

        Ok(pbuf)
    }
}
