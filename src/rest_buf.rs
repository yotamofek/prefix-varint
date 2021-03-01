use std::{iter::FusedIterator, mem::size_of};

const MAX_SIZE: usize = size_of::<u64>() + 1;

pub(crate) struct RestBuf {
    size: usize,
    buf: [u8; MAX_SIZE],
}

pub(crate) struct RestBufOwningIter {
    pos: usize,
    rest: RestBuf,
}

impl RestBuf {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            buf: [0; MAX_SIZE],
        }
    }
}

impl Iterator for RestBufOwningIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.rest.size {
            return None;
        }

        let res = self.rest.buf[self.pos];

        self.pos += 1;

        Some(res)
    }
}

impl FusedIterator for RestBufOwningIter {}

impl AsMut<[u8]> for RestBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[0..self.size]
    }
}

impl IntoIterator for RestBuf {
    type Item = u8;

    type IntoIter = RestBufOwningIter;

    fn into_iter(self) -> Self::IntoIter {
        RestBufOwningIter { pos: 0, rest: self }
    }
}
