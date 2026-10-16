//! Private Build_Options transport between compilers sharing a workspace registry.
//! The Jai facade supplies offsets from reflection, so no target layout is hardcoded.
//! Strings/views own their contents here; opaque user pointers retain borrowed identity.
use crate::interp::{Interp, Trap};

#[derive(Clone)]
pub(super) struct Snapshot {
    bytes: Vec<u8>,
    strings: Vec<(usize, Vec<u8>)>,
    arrays: Vec<(usize, Vec<Vec<u8>>)>,
}

fn fetch(interp: &Interp, address: u64, size: usize) -> Result<Vec<u8>, Trap> {
    interp
        .read(address, size)
        .ok_or_else(|| Trap::new("unreadable build options"))
}

fn pair(interp: &Interp, address: u64) -> Result<(usize, u64), Trap> {
    let (count, data) = interp
        .read_pair(address)
        .ok_or_else(|| Trap::new("unreadable build option view"))?;
    let count = usize::try_from(count).map_err(|_| Trap::new("invalid build option count"))?;
    Ok((count, data))
}

fn offsets(interp: &Interp, view: u64, size: usize) -> Result<Vec<usize>, Trap> {
    let (count, data) = pair(interp, view)?;
    let len = count
        .checked_mul(8)
        .ok_or_else(|| Trap::new("invalid option schema"))?;
    fetch(interp, data, len)?
        .as_chunks::<8>()
        .0
        .iter()
        .map(|bytes| {
            let offset = usize::try_from(u64::from_le_bytes(*bytes))
                .map_err(|_| Trap::new("invalid option offset"))?;
            if offset.checked_add(16).is_none_or(|end| end > size) {
                return Err(Trap::new("option view outside snapshot"));
            }
            Ok(offset)
        })
        .collect()
}

impl Snapshot {
    pub(super) fn capture(
        interp: &Interp,
        address: u64,
        size: usize,
        strings: u64,
        arrays: u64,
    ) -> Result<Self, Trap> {
        let bytes = fetch(interp, address, size)?;
        let mut result = Self {
            bytes,
            strings: Vec::new(),
            arrays: Vec::new(),
        };
        for offset in offsets(interp, strings, size)? {
            let (count, data) = pair(interp, address + offset as u64)?;
            result.strings.push((offset, fetch(interp, data, count)?));
        }
        for offset in offsets(interp, arrays, size)? {
            let (count, data) = pair(interp, address + offset as u64)?;
            count
                .checked_mul(16)
                .ok_or_else(|| Trap::new("invalid option array count"))?;
            let mut items = Vec::new();
            for i in 0..count {
                let (len, ptr) = pair(interp, data + (i * 16) as u64)?;
                items.push(fetch(interp, ptr, len)?);
            }
            result.arrays.push((offset, items));
        }
        Ok(result)
    }

    pub(super) fn restore(
        &self,
        interp: &mut Interp,
        address: u64,
        size: usize,
    ) -> Result<(), Trap> {
        if self.bytes.len() != size {
            return Err(Trap::new("build option schema size mismatch"));
        }
        let mut bytes = self.bytes.clone();
        for (offset, string) in &self.strings {
            let ptr = interp.register_data_global(string, 1, false, 1)?;
            put_view(&mut bytes[*offset..*offset + 16], string.len(), ptr);
        }
        for (offset, strings) in &self.arrays {
            let mut items = vec![0; strings.len() * 16];
            for (item, string) in items.as_chunks_mut::<16>().0.iter_mut().zip(strings) {
                let ptr = interp.register_data_global(string, 1, false, 1)?;
                put_view(item, string.len(), ptr);
            }
            let ptr = interp.register_data_global(&items, 8, false, 1)?;
            put_view(&mut bytes[*offset..*offset + 16], strings.len(), ptr);
        }
        if !interp.write(address, &bytes) {
            return Err(Trap::new("unwritable build options"));
        }
        Ok(())
    }
}

fn put_view(out: &mut [u8], count: usize, data: u64) {
    out[..8].copy_from_slice(&(count as u64).to_le_bytes());
    out[8..16].copy_from_slice(&data.to_le_bytes());
}
