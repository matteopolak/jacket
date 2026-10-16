//! Parse Linux's own-process mapping permissions without touching mapped data.
//! Paths are byte strings and are deliberately not decoded or interpreted.

pub(super) fn region(maps: &[u8], at: u64, write: bool) -> Option<(u64, u64)> {
    for line in maps
        .split(|&byte| byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let mut fields = line
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|field| !field.is_empty());
        let range = std::str::from_utf8(fields.next()?).ok()?;
        let permissions = fields.next()?;
        if !matches!(
            permissions,
            [b'r' | b'-', b'w' | b'-', b'x' | b'-', b'p' | b's']
        ) {
            return None;
        }
        let (start, end) = range.split_once('-')?;
        let start = u64::from_str_radix(start, 16).ok()?;
        let end = u64::from_str_radix(end, 16).ok()?;
        if start >= end {
            return None;
        }
        if start <= at && at < end {
            return (permissions[0] == b'r' && (!write || permissions[1] == b'w'))
                .then_some((start, end));
        }
    }
    None
}
