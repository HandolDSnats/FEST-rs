use crate::constants::FESTError;

pub fn to_uint32(bytes: &[u8], offset: usize) -> Result<u32, FESTError> {
    let bytes: [u8; 4] = bytes
        .get(offset..(offset + 4))
        .ok_or(FESTError::BytesToU32Error(bytes.len(), offset))?
        .try_into()
        .map_err(|_| FESTError::BytesToU32Error(bytes.len(), offset))?;

    Ok(u32::from_le_bytes(bytes))
}

pub fn from_uint32(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
