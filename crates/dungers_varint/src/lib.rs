use std::io;

pub const CONTINUE_BIT: u8 = 0x80;
pub const PAYLOAD_BITS: u8 = 0x7f;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("malformed varint")]
    MalformedVarint,
}

pub type Result<T> = std::result::Result<T, Error>;

// ZigZag Transform:  Encodes signed integers so that they can be effectively
// used with varint encoding.
//
// varint operates on unsigned integers, encoding smaller numbers into fewer
// bytes.  If you try to use it on a signed integer, it will treat this number
// as a very large unsigned integer, which means that even small signed numbers
// like -1 will take the maximum number of bytes (10) to encode.  ZigZagEncode()
// maps signed integers to unsigned in such a way that those with a small
// absolute value will have smaller encoded values, making them appropriate for
// encoding using varint.
//
//       int32 ->     uint32
// -------------------------
//           0 ->          0
//          -1 ->          1
//           1 ->          2
//          -2 ->          3
//         ... ->        ...
//  2147483647 -> 4294967294 -2147483648 -> 4294967295
//
//        >> encode >>
//        << decode <<

#[inline(always)]
pub fn zigzag_encode64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

#[inline(always)]
pub fn zigzag_decode64(n: u64) -> i64 {
    (n >> 1) as i64 ^ -((n & 1) as i64)
}

// Each byte in the varint has a continuation bit that indicates if the byte
// that follows it is part of the varint. This is the most significant bit (MSB)
// of the byte. The lower 7 bits are a payload; the resulting integer is built
// by appending together the 7-bit payloads of its constituent bytes.
// This allows variable size numbers to be stored with tolerable
// efficiency. Numbers sizes that can be stored for various numbers of
// encoded bits are:
//  8-bits: 0-127
// 16-bits: 128-16383
// 24-bits: 16384-2097151
// 32-bits: 2097152-268435455
// 40-bits: 268435456-0xFFFFFFFF

/// returns the max size (in bytes) of varint encoded number for `T`, assuming `T` is an integer
/// type.
pub const fn max_varint_size<T>() -> usize {
    // The longest varint encoding for an integer uses 7 bits per byte.
    (size_of::<T>() * 8 + 6) / 7
}

#[inline]
pub fn write_uvarint64<W: io::Write>(mut w: W, mut value: u64) -> Result<usize> {
    let mut buf = [0u8; max_varint_size::<u64>()];
    let mut count = 0;

    loop {
        if value < CONTINUE_BIT as u64 {
            unsafe {
                *buf.get_unchecked_mut(count) = value as u8;
            }
            count += 1;
            break;
        }

        unsafe {
            *buf.get_unchecked_mut(count) =
                ((value & PAYLOAD_BITS as u64) | CONTINUE_BIT as u64) as u8;
        }
        value >>= 7;
        count += 1;
    }

    w.write_all(&buf[..count])?;
    Ok(count)
}

#[inline]
pub fn write_varint64<W: io::Write>(w: W, value: i64) -> Result<usize> {
    write_uvarint64(w, zigzag_encode64(value))
}

#[inline]
pub fn read_uvarint64<R: io::Read>(rdr: &mut R) -> Result<(u64, usize)> {
    let mut buf = [0u8; 1];

    // NOTE: small values are more common then large ones, this is a performance win.
    rdr.read_exact(&mut buf)?;
    let byte = unsafe { *buf.get_unchecked(0) };
    if (byte & CONTINUE_BIT) == 0 {
        return Ok((byte as u64, 1));
    }

    let mut value = (byte & PAYLOAD_BITS) as u64;
    for count in 1..max_varint_size::<u64>() {
        rdr.read_exact(&mut buf)?;
        let byte = unsafe { *buf.get_unchecked(0) };
        if (byte & CONTINUE_BIT) == 0 {
            value |= (byte as u64) << (count * 7);
            return Ok((value, count));
        }
        value |= ((byte & PAYLOAD_BITS) as u64) << (count * 7);
    }

    Err(Error::MalformedVarint)
}

#[inline]
pub fn read_varint64<R: io::Read>(rdr: &mut R) -> Result<(i64, usize)> {
    read_uvarint64(rdr).map(|(value, n)| (zigzag_decode64(value), n))
}
