// public/bitvec.h
// static int bitsForBitnum[]
pub(crate) const BITS_FOR_BIT_NUM: [u64; 64] = {
    let mut bits_for_bit_num = [0; 64];
    let mut i: usize = 0;
    while i < 64 {
        bits_for_bit_num[i] = 1 << i;
        i += 1;
    }
    bits_for_bit_num
};

#[inline(always)]
pub const fn get_bit_for_bit_num(bit_num: usize) -> u64 {
    BITS_FOR_BIT_NUM[bit_num & 63]
}

// tier1/bitbuf.cpp
// uint32 g_BitWriteMasks[32][33];
pub(crate) const BIT_WRITE_MASKS: [[u64; 65]; 64] = {
    let mut bit_write_masks = [[0; 65]; 64];
    let mut start_bit = 0;
    while start_bit < 64 {
        let mut bits_left = 0;
        while bits_left < 65 {
            let end_bit = start_bit + bits_left;
            bit_write_masks[start_bit][bits_left] = get_bit_for_bit_num(start_bit) - 1;
            if end_bit < 64 {
                bit_write_masks[start_bit][bits_left] |= !(get_bit_for_bit_num(end_bit) - 1);
            }
            bits_left += 1;
        }
        start_bit += 1;
    }
    bit_write_masks
};

// tier1/bitbuf.cpp
// uint32 g_ExtraMasks[32];
pub(crate) const EXTRA_MASKS: [u64; 65] = {
    let mut extra_masks = [0; 65];
    let mut mask_bit = 0;
    while mask_bit < 65 {
        extra_masks[mask_bit] = if mask_bit == 64 {
            u64::MAX
        } else {
            get_bit_for_bit_num(mask_bit) - 1
        };
        mask_bit += 1;
    }
    extra_masks
};

// TOOD: move varint stuff into its own create possibly and put BitWriter's and BitReader's varint
// methods behind the feature flag maybe?

#[inline(always)]
pub(crate) fn zigzag_encode32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

#[inline(always)]
pub(crate) fn zigzag_encode64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

#[inline(always)]
pub(crate) fn zigzag_decode32(n: u32) -> i32 {
    (n >> 1) as i32 ^ -((n & 1) as i32)
}

#[inline(always)]
pub(crate) fn zigzag_decode64(n: u64) -> i64 {
    (n >> 1) as i64 ^ -((n & 1) as i64)
}
