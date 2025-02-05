mod checksum;
mod constants;
mod huffman8;
mod utils;

use checksum::get_checksum;
use constants::{
    FESTError, COMP_MAGIC, HUF_LCHAR, HUF_MASK4, HUF_NEXT, HUF_RCHAR, HUF_SHIFT, HUF_TREEOFS,
    INDE_MAGIC,
};
use huffman8::{Codes, Freqs, Node, Work};
use std::{fs::File, io::Write};
use utils::{from_uint32, to_uint32};

#[derive(Debug)]
pub struct FESData {
    pub raw: Vec<u8>,
    pub is_compressed: bool,
    is_chapter: bool,
}

impl FESData {
    pub fn process_data(raw: &[u8]) -> Result<FESData, FESTError> {
        match to_uint32(raw, 0)? {
            COMP_MAGIC => Ok(FESData {
                raw: raw.to_vec(),
                is_compressed: true,
                is_chapter: false,
            }),
            INDE_MAGIC => Ok(FESData {
                raw: raw.to_vec(),
                is_compressed: false,
                is_chapter: false,
            }),
            _ => match to_uint32(raw, 0xC0)? {
                COMP_MAGIC => Ok(FESData {
                    raw: raw.to_vec(),
                    is_compressed: true,
                    is_chapter: true,
                }),
                INDE_MAGIC => Ok(FESData {
                    raw: raw.to_vec(),
                    is_compressed: false,
                    is_chapter: true,
                }),
                _ => Err(FESTError::InvalidData),
            },
        }
    }

    pub fn decompress(self) -> Result<Self, FESTError> {
        if !self.is_compressed {
            return Ok(self);
        }

        let header = if self.is_chapter {
            self.raw.get(..0xC0).ok_or(FESTError::InvalidData)?.to_vec()
        } else {
            vec![]
        };

        let data = match self.is_chapter {
            true => self.raw.get(0xD0..).ok_or(FESTError::InvalidData)?,
            false => self.raw.get(0x10..).ok_or(FESTError::InvalidData)?,
        };

        let mut raw = vec![];
        raw.extend(header);
        raw.extend(decompress(data)?);

        Ok(FESData {
            raw,
            is_compressed: false,
            is_chapter: self.is_chapter,
        })
    }

    pub fn compress(self) -> Result<Self, FESTError> {
        if self.is_compressed {
            return Ok(self);
        }

        let header = {
            match self.is_chapter {
                true => self.raw.get(..0xC0).ok_or(FESTError::InvalidData)?.to_vec(),
                false => vec![],
            }
        };

        let data = {
            match self.is_chapter {
                true => self.raw.get(0xC0..).ok_or(FESTError::InvalidData)?,
                false => &self.raw,
            }
        };

        let mut raw = vec![];
        raw.extend(&header);
        raw.extend(from_uint32(COMP_MAGIC));
        raw.extend(from_uint32(2));
        raw.extend(from_uint32(data.len() as u32));

        let checksum = get_checksum(0, &header)?;
        let checksum = get_checksum(checksum, data)?;

        raw.extend(from_uint32(checksum));
        raw.extend(compress(data)?);

        Ok(FESData {
            raw,
            is_compressed: true,
            is_chapter: self.is_chapter,
        })
    }

    pub fn write_to(&self, file_name: &str) -> Result<(), FESTError> {
        let mut file = File::create(file_name).map_err(|t| FESTError::WriteError(t.to_string()))?;

        file.write_all(&self.raw)
            .map_err(|t| FESTError::WriteError(t.to_string()))
    }
}

fn decompress(data: &[u8]) -> Result<Vec<u8>, FESTError> {
    let header = to_uint32(data, 0)?;
    let num_bits = (header & 0xF) as u8;
    let mut decompressed = vec![0u8; (header >> 8) as usize];

    let mut pak_pos: usize = {
        let value = *data
            .get(4)
            .ok_or(FESTError::DecompressDataIOOB(4, data.len()))? as u32;
        let value = 4 + ((value + 1) << 1);

        value
            .try_into()
            .map_err(|_| FESTError::UnsuportedArchitecture)?
    };
    let mut raw_pos: usize = 0;
    let mut mask4: u32 = 0;
    let mut pos: &u8 = data
        .get(HUF_TREEOFS + 1)
        .ok_or(FESTError::DecompressDataIOOB(HUF_TREEOFS + 1, data.len()))?;
    let mut next: usize = 0;
    let mut nbits: u8 = 0;

    let mut code = to_uint32(data, pak_pos)?;
    while raw_pos < decompressed.len() {
        mask4 >>= HUF_SHIFT;

        if mask4 == 0 {
            if (pak_pos + 3) >= data.len() {
                break;
            }

            code = to_uint32(data, pak_pos)?;
            pak_pos += 4;
            mask4 = HUF_MASK4;
        }

        next += (((pos & HUF_NEXT) + 1) << 1) as usize;

        let ch;
        if (code & mask4) == 0 {
            ch = pos & HUF_LCHAR;
            pos = data
                .get(HUF_TREEOFS + next)
                .ok_or(FESTError::DecompressDataIOOB(
                    HUF_TREEOFS + next,
                    data.len(),
                ))?;
        } else {
            ch = pos & HUF_RCHAR;
            pos = data
                .get(HUF_TREEOFS + next + 1)
                .ok_or(FESTError::DecompressDataIOOB(
                    HUF_TREEOFS + next + 1,
                    data.len(),
                ))?;
        }

        if ch != 0 {
            let byte = decompressed
                .get_mut(raw_pos)
                .ok_or(FESTError::DecompressDataIOOB(raw_pos, data.len()))?;
            *byte |= pos << (nbits as u32);

            nbits = (nbits + num_bits) & 7;

            if nbits == 0 {
                raw_pos += 1;
            }

            pos = data
                .get(HUF_TREEOFS + 1)
                .ok_or(FESTError::DecompressDataIOOB(HUF_TREEOFS + 1, data.len()))?;
            next = 0;
        }
    }

    Ok(decompressed)
}

fn compress(data: &[u8]) -> Result<Vec<u8>, FESTError> {
    let Freqs { freqs, num_leafs } = Freqs::create_freqs(data)?;
    let tree = Node::create_tree(&freqs, num_leafs)?;

    let mut codes = Codes::create_code(num_leafs)?;
    codes.create_code_branch(
        tree.last()
            .ok_or(FESTError::NodeNotFound("Root node".to_string()))?,
        1,
        2,
    )?; // NOTE: We can omit "root_tree" since it is not used anywhere else, it is just there to kickstart the recursive function, and since, as described before, "tree.len()" is the same as "num_nodes", we can omit the substraction and directly use the last element
    codes.update_code()?;
    let work = {
        // NOTE: Since "num_leafs" could be calculated from the original size of "tree", maybe the inverse could be done here to disappear "num_leafs" completely
        let tree_data = unsafe { tree.get_unchecked(0..num_leafs) };
        let tree_data = tree_data
            .iter()
            .map(|node| {
                let node = node.borrow();
                (node.symbol, node.get_scode())
            })
            .collect::<Vec<(usize, Vec<u8>)>>();

        Work::create_code_works(tree_data)?
    };

    work.process_data(data, &codes.code_tree)
}
