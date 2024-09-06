pub fn vec_to_i32(vec: Vec<u8>) -> Option<i32> {
    if vec.len() == 4 {
        let byte_array: [u8; 4] = vec.try_into().ok()?;
        Some(i32::from_le_bytes(byte_array))
    } else {
        None
    }
}
pub fn i32_to_vec(value: i32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
