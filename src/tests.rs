use std::iter;

use rand::{thread_rng, Rng as _};

use super::*;

struct AssertEofCursor<T>
where
    T: AsRef<[u8]>,
{
    cursor: io::Cursor<T>,
}

impl<T> AssertEofCursor<T>
where
    T: AsRef<[u8]>,
{
    fn new(inner: T) -> Self {
        Self {
            cursor: io::Cursor::new(inner),
        }
    }
}

impl<T> Read for AssertEofCursor<T>
where
    T: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl<T> Drop for AssertEofCursor<T>
where
    T: AsRef<[u8]>,
{
    fn drop(&mut self) {
        assert_eq!(
            self.cursor.position(),
            self.cursor.get_ref().as_ref().len() as u64,
            "read cursor not fully consumed"
        );
    }
}

#[test]
fn test_read_size_from_prefix() -> Result<(), OverflowError> {
    assert_eq!(read_size_from_prefix(0b1010_1011)?, 0);
    assert_eq!(read_size_from_prefix(0b1010_1010)?, 1);
    assert_eq!(read_size_from_prefix(0b1010_1100)?, 2);
    assert_eq!(read_size_from_prefix(0b1011_1000)?, 3);
    assert_eq!(read_size_from_prefix(0b1011_0000)?, 4);
    assert_eq!(read_size_from_prefix(0b1010_0000)?, 5);
    assert_eq!(read_size_from_prefix(0b1100_0000)?, 6);
    assert_eq!(read_size_from_prefix(0b1000_0000)?, 7);
    assert_eq!(read_size_from_prefix(0b0000_0000)?, 8);

    Ok(())
}

#[test]
fn test_read_prefix() -> Result<(), OverflowError> {
    assert_eq!(read_prefix(0b1010_1011)?, (0b101_0101, 0));
    assert_eq!(read_prefix(0b1010_1010)?, (0b10_1010, 1));
    assert_eq!(read_prefix(0b1010_1100)?, (0b1_0101, 2));
    assert_eq!(read_prefix(0b1011_1000)?, (0b1011, 3));
    assert_eq!(read_prefix(0b1011_0000)?, (0b101, 4));
    assert_eq!(read_prefix(0b1010_0000)?, (0b10, 5));
    assert_eq!(read_prefix(0b1100_0000)?, (0b1, 6));
    assert_eq!(read_prefix(0b1000_0000)?, (0b0, 7));
    assert_eq!(read_prefix(0b0000_0000)?, (0b0, 8));

    Ok(())
}

#[test]
fn test_read_varint() -> io::Result<()> {
    macro_rules! test_varint_read {
        ($bytes:expr, ok = $expected:expr) => {
            assert_eq!(read_varint(AssertEofCursor::new($bytes))?, $expected);

            let mut cursor = io::Cursor::new($bytes.to_vec());
            cursor.get_mut().splice(0..0, iter::repeat(0).take(5));
            cursor.set_position(5);

            assert_eq!(read_varint_from_slice(&mut cursor)?, $expected);
            assert_eq!(cursor.position(), $bytes.len() as u64 + 5);
        };
        ($bytes:expr, error_kind = $error_kind:expr) => {
            assert_eq!(
                read_varint(AssertEofCursor::new($bytes))
                    .unwrap_err()
                    .kind(),
                $error_kind
            );
            assert_eq!(
                read_varint_from_slice(&mut io::Cursor::new($bytes))
                    .unwrap_err()
                    .kind(),
                $error_kind
            );
        };
    }

    test_varint_read!([0b1010_1011], ok = 0b101_0101);

    test_varint_read!([0b1010_1010, 0b1010_1010], ok = 0b10_1010_1010_1010);

    test_varint_read!([0b1010_1010], error_kind = io::ErrorKind::UnexpectedEof);

    test_varint_read!(
        [0b1010_1100, 0b1010_1010, 0b1010_1010,],
        ok = 0b1_0101_0101_0101_0101_0101
    );
    test_varint_read!(
        [0b1010_1100, 0b1010_1010],
        error_kind = io::ErrorKind::UnexpectedEof
    );

    test_varint_read!(
        [0b1010_1000, 0b1010_1010, 0b1010_1010, 0b1010_1010],
        ok = 0b1010_1010_1010_1010_1010_1010_1010
    );
    test_varint_read!(
        [0b1010_1000, 0b1010_1010, 0b1010_1010],
        error_kind = io::ErrorKind::UnexpectedEof
    );

    test_varint_read!(
        [
            0b1011_0000,
            0b1010_1010,
            0b1010_1010,
            0b1010_1010,
            0b1010_1010
        ],
        ok = 0b101_0101_0101_0101_0101_0101_0101_0101_0101
    );
    test_varint_read!(
        [0b1011_0000, 0b1010_1010, 0b1010_1010, 0b1010_1010],
        error_kind = io::ErrorKind::UnexpectedEof
    );

    test_varint_read!(
        [
            0,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
        ],
        ok = u64::MAX
    );

    test_varint_read!(
        [
            0b1000_0000,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            u8::MAX,
        ],
        ok = 2_u64.pow(56) - 1
    );

    Ok(())
}

#[test]
fn test_calc_varint_size() {
    assert_eq!(calc_varint_size(u64::MIN), 1);

    assert_eq!(calc_varint_size((2_u64.pow(7)) - 1), 1);
    assert_eq!(calc_varint_size(2_u64.pow(7)), 2);

    assert_eq!(calc_varint_size(u64::MAX), size_of::<u64>() + 1);
}

#[test]
fn test_roundtrip() -> io::Result<()> {
    let mut buf = vec![];

    let mut rng = thread_rng();

    for n in vec![u64::MIN, u64::MAX]
        .into_iter()
        .chain(iter::from_fn(move || rng.gen()).take(150_000))
    {
        write_varint(n, &mut buf)?;
        assert_eq!(read_varint(AssertEofCursor::new(&buf)).unwrap(), n);
        buf.clear();
    }

    Ok(())
}
