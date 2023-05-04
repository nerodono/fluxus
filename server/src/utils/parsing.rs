use memchr::memchr;

#[rustfmt::skip]
const DEC_TABLE: [u8; 256] = [
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
     0,    1,    2,    3,    4,    5,    6,    7,
     8,    9, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10
];

#[rustfmt::skip]
const HEX_TABLE: [u8; 256] = [
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
     0,    1,    2,    3,    4,    5,    6,    7,
     8,    9, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10,   10,   11,   12,   13,   14,   15, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10,   10,   11,   12,   13,   14,   15, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
  0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10
];

pub fn strip_left_space(value: &[u8]) -> &[u8] {
    value.strip_prefix(b" ").unwrap_or(value)
}

fn is_chunked_val(value: &[u8]) -> bool {
    const VAL: &[u8] = b"CHUNKED";
    const VAL_COMMA: &[u8] = b"CHUNKED,";
    const LEN: usize = VAL.len();

    if value.len() >= LEN {
        value
            .iter()
            .take(LEN + 1)
            .enumerate()
            .all(|(i, &c)| ascii_upper(c) == VAL_COMMA[i])
    } else {
        false
    }
}

pub fn is_chunked_transfer(value: &[u8]) -> bool {
    value
        .split(|&c| c == b',')
        .find(|val| is_chunked_val(strip_left_space(val)))
        .map_or(false, |_| true)
}
pub fn split_key_value(line: &[u8]) -> Option<(&[u8], &[u8])> {
    memchr(b':', line).map(|colon_pos| {
        let (key, value) = line.split_at(colon_pos);
        (key, &value[1..])
    })
}

pub fn root_path(path: &[u8]) -> (&[u8], &[u8]) {
    let Some(slash_pos) = memchr(b'/', path) else {
        return (path, b"");
    };
    let (left, right) = path.split_at(slash_pos);
    (left, &right[1..])
}

#[cfg(test)]
mod tests {
    use super::is_chunked_val;

    #[test]
    fn test_chunked_value() {
        assert!(is_chunked_val(b"chuNked"));
        assert!(is_chunked_val(b"chunKed, not-chunked, lol"));
        assert!(!is_chunked_val(b"NOOOOO, not-chunked, lol"));
    }
}

/// Assumed that right is all uppercase
pub fn case_insensitive_eq_left(left: &[u8], right: &[u8]) -> bool {
    let mut left_len = left.len();
    let right_len = right.len();

    if left_len == right_len {
        while left_len != 0 {
            left_len -= 1;
            // For some reason compiler inserts unnecessary bound-checks
            if ascii_upper(unsafe { *left.get_unchecked(left_len) })
                != unsafe { *right.get_unchecked(left_len) }
            {
                return false;
            }
        }

        true
    } else {
        false
    }
}

pub fn split3_spaces(src: &[u8]) -> Option<(usize, usize, usize)> {
    let space_pos = memchr(b' ', src)?;
    let mid_len = memchr(b' ', &src[space_pos + 1..])?;

    Some((space_pos, mid_len, src.len() - space_pos - mid_len - 2))
}

pub const fn parse_number(buffer: &[u8]) -> Option<u64> {
    let buf_len = buffer.len();
    if buf_len > 20 {
        return None;
    }
    let mut output = 0;
    let mut idx = 0;

    while idx < buf_len {
        output += (match DEC_TABLE[buffer[idx] as usize] {
            0x10 => return None,
            n => n,
        } as u64)
            * 10_u64.pow((buf_len - idx - 1) as u32);
        idx += 1;
    }

    Some(output)
}

pub const fn parse_hex(buffer: &[u8]) -> Option<u32> {
    let buf_len = buffer.len();
    // maximum value is ffffffff
    if buf_len > 8 {
        return None;
    }

    let mut output = 0;
    let mut idx = 0;

    while idx < buf_len {
        output += (match HEX_TABLE[buffer[idx] as usize] {
            0x10 => return None,
            e => e,
        } as u32)
            << ((buf_len - idx - 1) << 2);
        idx += 1;
    }

    Some(output)
}

pub const fn ascii_upper(c: u8) -> u8 {
    c & !(0x20 * c.is_ascii_lowercase() as u8)
}
