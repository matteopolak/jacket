//! Private `Build_Options` transport between compilers sharing a workspace registry.
//! The caller supplies the string and view offsets from the declared layout, so no target
//! layout is hardcoded here.
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
        .read_view(address)
        .ok_or_else(|| Trap::new("unreadable build option view"))?;
    let count = usize::try_from(count).map_err(|_| Trap::new("invalid build option count"))?;
    Ok((count, data))
}

/// Check that each offset leaves room for a `view`-byte view inside a snapshot of `size` bytes.
fn checked(
    offsets: &[u64],
    size: usize,
    view: u64,
) -> Result<impl Iterator<Item = usize> + '_, Trap> {
    if offsets
        .iter()
        .any(|&offset| offset.checked_add(view).is_none_or(|end| end > size as u64))
    {
        return Err(Trap::new("option view outside snapshot"));
    }
    Ok(offsets.iter().map(|&offset| offset as usize))
}

impl Snapshot {
    pub(super) fn capture(
        interp: &Interp,
        address: u64,
        size: usize,
        strings: &[u64],
        arrays: &[u64],
    ) -> Result<Self, Trap> {
        let view = interp.data_layout().builtins().view.size;
        let bytes = fetch(interp, address, size)?;
        let mut result = Self {
            bytes,
            strings: Vec::new(),
            arrays: Vec::new(),
        };
        for offset in checked(strings, size, view)? {
            let (count, data) = pair(interp, address + offset as u64)?;
            result.strings.push((offset, fetch(interp, data, count)?));
        }
        for offset in checked(arrays, size, view)? {
            let (count, data) = pair(interp, address + offset as u64)?;
            count
                .checked_mul(view as usize)
                .ok_or_else(|| Trap::new("invalid option array count"))?;
            let mut items = Vec::new();
            for i in 0..count {
                let (len, ptr) = pair(interp, data + i as u64 * view)?;
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
        let layout = *interp.data_layout();
        let view = layout.builtins().view;
        let mut bytes = self.bytes.clone();
        for (offset, string) in &self.strings {
            let ptr = interp.register_data_global(string, 1, false, 1)?;
            layout.write_view(&mut bytes[*offset..], string.len() as u64, ptr);
        }
        for (offset, strings) in &self.arrays {
            let mut items = vec![0; strings.len() * view.size as usize];
            for (item, string) in items.chunks_exact_mut(view.size as usize).zip(strings) {
                let ptr = interp.register_data_global(string, 1, false, 1)?;
                layout.write_view(item, string.len() as u64, ptr);
            }
            let ptr = interp.register_data_global(&items, view.align, false, 1)?;
            layout.write_view(&mut bytes[*offset..], strings.len() as u64, ptr);
        }
        if !interp.write(address, &bytes) {
            return Err(Trap::new("unwritable build options"));
        }
        Ok(())
    }
}
