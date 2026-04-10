pub fn find_tree(bitmap: &[u8]) -> Option<usize> {
    let chunks =  bitmap.chunks_exact(8);
    let remainder = chunks.remainder();

    for (i, chunk) in chunks.enumerate() {
        let word = u64::from_le_bytes(chunk.try_into().unwrap());
        if word == u64::MAX { continue; }

        return Some(i * 64 + (!word).trailing_zeros() as usize);
    }

    let base = (bitmap.len() / 8) * 64;
    for (i, &byte) in remainder.iter().enumerate() {
        if byte == 0xFF { continue; }
        return Some(base + i * 8 + (!byte as u64).trailing_zeros() as usize);
    }

    None
}

pub fn set(bitmap: &mut [u8], idx: usize) {
    bitmap[idx / 8] |= 1 << (idx % 8);
}
pub fn clear(bitmap: &mut [u8], idx: usize) {
    bitmap[idx / 8] &= !(1 << (idx % 8));
}
pub fn is_set(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx / 8] & (1 << (idx % 8)) != 0
}
pub fn count_free(bitmap: &[u8]) -> usize {
    bitmap.iter().map(|b| b.count_zeros() as usize).sum()
}
