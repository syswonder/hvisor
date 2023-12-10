pub fn sign_extend(value: u64, size: u64) -> u64 {
    let shift = 32 - size;
    (((value << shift) as i32) >> shift) as u64
}
