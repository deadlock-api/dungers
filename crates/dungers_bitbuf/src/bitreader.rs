#[cfg(feature = "varint")]
use dungers_varint::{max_varint_size, zigzag_decode64, CONTINUE_BIT, PAYLOAD_BITS};

use crate::{Error, Result, EXTRA_MASKS};

pub struct BitReader<'a> {
    data_bits: usize,
    data: &'a [u64],
    cur_bit: usize,
}

impl<'a> BitReader<'a> {
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        // make sure that alignment is correct.
        debug_assert!(data.len() % 8 == 0);

        Self {
            data_bits: data.len() << 3,
            // SAFETY: transmuting data into a slice of u64s is safe here because BitReader
            // requires the input data to be 8-byte aligned, which is enforced by the debug_assert
            // above.
            data: unsafe { std::mem::transmute(data) },
            cur_bit: 0,
        }
    }

    #[inline(always)]
    pub fn num_bits_left(&self) -> usize {
        self.data_bits - self.cur_bit
    }

    #[inline(always)]
    pub fn num_bytes_left(&self) -> usize {
        self.num_bits_left() >> 3
    }

    #[inline(always)]
    pub fn num_bits_read(&self) -> usize {
        self.cur_bit
    }

    #[inline(always)]
    pub fn num_bytes_read(&self) -> usize {
        (self.cur_bit + 7) >> 3
    }

    /// seek to a specific bit.
    pub fn seek(&mut self, bit: usize) -> Result<()> {
        if bit > self.data_bits {
            return Err(Error::Overflow);
        }
        self.cur_bit = bit;
        Ok(())
    }

    /// seek to an offset from the current position.
    pub fn seek_relative(&mut self, bit_delta: isize) -> Result<usize> {
        let bit = self.cur_bit as isize + bit_delta;
        if bit < 0 {
            return Err(Error::Overflow);
        }
        self.seek(bit as usize)?;
        Ok(self.cur_bit)
    }

    #[inline(always)]
    pub unsafe fn read_ubit64_unchecked(&mut self, num_bits: usize) -> u64 {
        // make sure that the requested number of bits to read is in bounds of u64.
        debug_assert!(num_bits <= 64);
        // make sure that there's enough bits left
        debug_assert!(self.num_bits_left() >= self.num_bits_left());

        // SAFETY: asserts above ensure that we'll not go out of bounds; but they will be gone in
        // release builds.

        let block1_idx = self.cur_bit >> 6;

        let mut block1 = unsafe { *self.data.get_unchecked(block1_idx) };
        // get the bits we're interested in
        block1 >>= self.cur_bit & 63;

        self.cur_bit += num_bits;
        let mut ret = block1;

        // does it span this block?
        if (self.cur_bit - 1) >> 6 == block1_idx {
            ret &= EXTRA_MASKS[num_bits];
        } else {
            let extra_bits = self.cur_bit & 63;

            let mut block2 = unsafe { *self.data.get_unchecked(block1_idx + 1) };
            block2 &= EXTRA_MASKS[extra_bits];

            // no need to mask since we hit the end of the dword.
            // shift the second dword's part into the high bits.
            ret |= block2 << (num_bits - extra_bits);
        }

        ret
    }

    /// read_ubit64 reads the specified number of bits into a `u64`. the function can read up to a
    /// maximum of 64 bits at a time. if the `num_bits` exceeds the number of remaining bits, the
    /// function returns an [`Error::Overflow`] error.
    #[inline]
    pub fn read_ubit64(&mut self, num_bits: usize) -> Result<u64> {
        // make sure that the requested number of bits to read is in bounds of u64.
        debug_assert!(num_bits <= 64);

        if self.num_bits_left() < num_bits {
            return Err(Error::Overflow);
        }

        // SAFETY: assert and check above ensure that we'll not go out of bounds.
        unsafe { Ok(self.read_ubit64_unchecked(num_bits)) }
    }

    #[inline(always)]
    pub unsafe fn read_bool_unchecked(&mut self) -> bool {
        // ensure that there's at least one bit left
        debug_assert!(self.num_bits_left() >= 1);

        // SAFETY: assert above ensures that we'll not go out of bounds; but it will be gone in
        // release builds.

        let one_bit =
            unsafe { self.data.get_unchecked(self.cur_bit >> 6) } >> (self.cur_bit & 63) & 1;
        self.cur_bit += 1;

        one_bit == 1
    }

    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        if self.num_bits_left() < 1 {
            return Err(Error::Overflow);
        }

        // SAFETY: check above ensures that we'll not go out of bounds.
        unsafe { Ok(self.read_bool_unchecked()) }
    }

    #[inline(always)]
    pub unsafe fn read_byte_unchecked(&mut self) -> u8 {
        // NOTE: there's no point to assert anything here because read_bits_unchecked contains all
        // the necessary debug assertions.
        self.read_ubit64_unchecked(8) as u8
    }

    #[inline]
    pub fn read_byte(&mut self) -> Result<u8> {
        self.read_ubit64(8).map(|result| result as u8)
    }

    // NOTE: ref impl for varints:
    // https://github.com/rust-lang/rust/blob/e5b3e68abf170556b9d56c6f9028318e53c9f06b/compiler/rustc_serialize/src/leb128.rs

    // TODO: varint funcs can be faster

    #[cfg(feature = "varint")]
    pub unsafe fn read_uvarint64_unchecked(&mut self) -> u64 {
        let byte = self.read_byte_unchecked();
        if (byte & CONTINUE_BIT) == 0 {
            return byte as u64;
        }

        let mut value = (byte & 0x7f) as u64;
        for count in 1..=max_varint_size::<u64>() {
            let byte = self.read_byte_unchecked();
            if (byte & CONTINUE_BIT) == 0 {
                value |= (byte as u64) << (count * 7);
                return value;
            }
            value |= ((byte & PAYLOAD_BITS) as u64) << (count * 7);
        }

        panic!("{}", Error::MalformedVarint)
    }

    #[cfg(feature = "varint")]
    pub fn read_uvarint64(&mut self) -> Result<u64> {
        let byte = self.read_byte()?;
        if (byte & CONTINUE_BIT) == 0 {
            return Ok(byte as u64);
        }

        let mut value = (byte & 0x7f) as u64;
        for count in 1..=max_varint_size::<u64>() {
            let byte = self.read_byte()?;
            if (byte & CONTINUE_BIT) == 0 {
                value |= (byte as u64) << (count * 7);
                return Ok(value);
            }
            value |= ((byte & PAYLOAD_BITS) as u64) << (count * 7);
        }

        Err(Error::MalformedVarint)
    }

    #[cfg(feature = "varint")]
    pub unsafe fn read_varint64_unchecked(&mut self) -> i64 {
        zigzag_decode64(self.read_uvarint64_unchecked())
    }

    #[cfg(feature = "varint")]
    pub fn read_varint64(&mut self) -> Result<i64> {
        self.read_uvarint64().map(zigzag_decode64)
    }
}
