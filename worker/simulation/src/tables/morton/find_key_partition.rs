//! Find the index of the partition where `key` _might_ reside.
//! This is the index of the first item in the `skiplist` that is greater than the `key`
//!
use super::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_mm_cmpgt_epi32, _mm_movemask_epi8, _mm_set_epi32, _popcnt64};

#[cfg(target_arch = "x86_64")]
#[inline]
pub fn find_key_partition(skiplist: &SkipList, key: MortonKey) -> usize {
    unsafe { find_key_partition_sse2(&skiplist, key) }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
pub fn find_key_partition(skiplist: &SkipList, key: MortonKey) -> usize {
    let key = key.0 as i32;
    for (i, skip) in skiplist.0.iter().enumerate() {
        if skip > &key {
            return i;
        }
    }
    SKIP_LEN
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn find_key_partition_sse2(skiplist: &SkipList, key: MortonKey) -> usize {
    let key = key.0 as i32;
    let keys4 = _mm_set_epi32(key, key, key, key);

    // set every 32 bits to 0xFFFF if key > skip else sets it to 0x0000
    let cmp0 = _mm_cmpgt_epi32(keys4, skiplist.0[0]);
    let cmp1 = _mm_cmpgt_epi32(keys4, skiplist.0[1]);
    let cmp2 = _mm_cmpgt_epi32(keys4, skiplist.0[2]);
    let cmp3 = _mm_cmpgt_epi32(keys4, skiplist.0[3]);

    // create a mask from the most significant bit of each 8bit element
    let mask0 = _mm_movemask_epi8(cmp0);
    let mask1 = _mm_movemask_epi8(cmp1);
    let mask2 = _mm_movemask_epi8(cmp2);
    let mask3 = _mm_movemask_epi8(cmp3);

    // count the number of bits set to 1
    let index4 = _popcnt64(((mask0 as i64) << 32) | mask1 as i64)
        + _popcnt64(((mask2 as i64) << 32) | mask3 as i64);

    // every key in skip list is counted 4 times.
    (index4 as usize) >> 2
}
