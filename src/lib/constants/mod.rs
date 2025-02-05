// Constants
pub const COMP_MAGIC: u32 = 0x434F4D50;
pub const INDE_MAGIC: u32 = 0x494E4445;

pub const CMD_CODE: u32 = 0x28;

pub const HUF_SHIFT: u8 = 1;
pub const HUF_MASK: u8 = 0x80;
pub const HUF_MASK4: u32 = 0x80000000;

pub const HUF_LNODE: u8 = 0;
pub const HUF_RNODE: u8 = 1;

pub const HUF_LCHAR: u8 = 0x80;
pub const HUF_RCHAR: u8 = 0x40;
pub const HUF_NEXT: u8 = 0x3F;

pub const HUF_TREEOFS: usize = 4;

// NOTE: "HUF_MAXSYMBOLS" was removed since it was just "0xFF + 1" instead of being a magic number
// NOTE: "HUF_MAXIM" was removed since it was only used in "pbuf" during compression, but since it is growable instead of fixed-size, this constant doesn't need to exist

// Errors
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum FESTError {
    #[error("Data is not usable by this library")]
    InvalidData,
    #[error("Error when generating checksum: {0}")]
    ChecksumError(String),
    #[error("Error when writing to file: {0}")]
    WriteError(String),
    #[error("Not 32-bit target or higher, won't work")]
    UnsuportedArchitecture,
    #[error("Error transforming bytes of length {0} on offset {1}")]
    BytesToU32Error(usize, usize),
    #[error("Index out of bounds for compressed data at {0} when length is {1}")]
    DecompressDataIOOB(usize, usize),
    #[error("Frequency overflowed for index {0}")]
    FreqOverflow(u8),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Code not found: {0}")]
    CodeNotFound(String),
    #[error("CodeWork not found at index {0}")]
    CodeWorkNotFound(usize),
    #[error("Work not found at key {0}")]
    WorkNotFound(usize),
    #[error("Index out of bounds for code_tree at {0} when length is {1}")]
    CodeTreeIOOB(String, usize),
    #[error("Index out of bounds for code_mask at {0} when length is {1}")]
    CodeMaskIOOB(String, usize),
}
