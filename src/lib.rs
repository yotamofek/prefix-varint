mod overflow;
mod rest_buf;

use std::{
    io::{self, Read, Write},
    mem::size_of,
};

pub use crate::overflow::OverflowError;

#[inline]
fn read_size_from_prefix(byte: u8) -> Result<usize, OverflowError> {
    let size = byte.trailing_zeros() as usize;

    if size > size_of::<u64>() {
        Err(OverflowError::new(size))
    } else {
        Ok(size)
    }
}

#[inline]
pub fn read_prefix(byte: u8) -> Result<(u64, usize), OverflowError> {
    let size = read_size_from_prefix(byte)?;

    Ok(((u64::from(byte) >> (size + 1)), size))
}

#[inline]
pub fn read_varint<R>(mut rdr: R) -> io::Result<u64>
where
    R: Read,
{
    let prefix_byte = {
        let mut buf = [u8::MIN];
        rdr.read_exact(&mut buf)?;

        buf[0]
    };

    let (mut int, size) = read_prefix(prefix_byte)?;

    let rest = {
        let mut buf = rest_buf::RestBuf::new(size);
        rdr.read_exact(buf.as_mut())?;
        buf
    };

    let pad = 8_u8.min(size as u8 + 1);

    for (i, byte) in (1_u8..).zip(rest) {
        int |= u64::from(byte) << usize::from((i * 8) - pad)
    }

    Ok(int)
}

fn calc_varint_size(int: u64) -> usize {
    let bits = match int.checked_next_power_of_two() {
        Some(pow) => match pow {
            pow if pow > int => pow,
            pow => pow + pow,
        }
        .trailing_zeros(),
        None => u64::BITS - 1,
    } as usize;

    1.max(bits / 7 + usize::from(bits % 7 > 0))
}

#[inline]
pub fn write_varint<W: Write>(mut int: u64, mut f: W) -> io::Result<usize> {
    let size = calc_varint_size(int) as u8;

    // number of bits packed into first byte (excluding bits needed for marker)
    let packed_bits = 8_u8.saturating_sub(size);

    let prefix = {
        let int = if packed_bits == 0 {
            0
        } else {
            ((int & ((2_u64.pow(u32::from(packed_bits + 1))) - 1)) << size) as u8
        };

        int | (1_u8.checked_shl(u32::from(size) - 1).unwrap_or(0))
    };

    int >>= packed_bits;

    f.write_all(&[prefix])?;

    for _ in 0..(size - 1) {
        let byte: u8 = (int & u64::from(u8::MAX)) as u8;
        f.write_all(&[byte])?;

        int >>= 8;
    }

    Ok(size.into())
}

#[cfg(test)]
mod tests;
