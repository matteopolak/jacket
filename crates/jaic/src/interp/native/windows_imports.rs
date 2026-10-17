//! COFF import-provider metadata only. No DLL execution or arbitrary string scanning.
use std::path::{Path, PathBuf};

fn bytes(data: &[u8], at: usize, len: usize) -> Result<&[u8], String> {
    data.get(at..at.checked_add(len).ok_or("COFF offset overflow")?)
        .ok_or_else(|| "truncated COFF import metadata".into())
}

fn u16_at(data: &[u8], at: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(bytes(data, at, 2)?.try_into().unwrap()))
}

fn u32_at(data: &[u8], at: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(bytes(data, at, 4)?.try_into().unwrap()))
}

fn terminated(data: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let end = data
        .iter()
        .position(|&c| c == 0)
        .ok_or("unterminated COFF import name")?;
    Ok((&data[..end], &data[end + 1..]))
}

fn provider(data: &[u8]) -> Result<String, String> {
    let (name, _) = terminated(data)?;
    if name.is_empty()
        || name
            .iter()
            .any(|&c| !c.is_ascii() || c < 32 || b"<>:\"/\\|?*".contains(&c))
    {
        return Err("import provider must be a nonempty module basename".into());
    }
    // Import providers can be .drv/.ocx as well as .dll. Preserve their actual name.
    String::from_utf8(name.to_vec()).map_err(|_| "invalid import provider name".into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ImportTarget {
    Name(String),
    Ordinal(u16),
}

#[derive(Clone, Debug)]
pub(crate) struct Import {
    pub symbol: String,
    pub provider: String,
    pub target: ImportTarget,
}

#[derive(Clone, Debug)]
pub(crate) struct Archive {
    pub providers: Vec<String>,
    pub imports: Vec<Import>,
    expected: u16,
    incompatible_symbols: Vec<(u16, String)>,
}

impl Archive {
    /// Linker symbols are case-sensitive; DLL identities are case-insensitive.
    pub fn select(&self, symbol: &str) -> Result<Option<&Import>, String> {
        let mut selected: Option<&Import> = None;
        for import in self.imports.iter().filter(|import| import.symbol == symbol) {
            if let Some(previous) = selected {
                if !previous.provider.eq_ignore_ascii_case(&import.provider)
                    || previous.target != import.target
                {
                    return Err(format!(
                        "import symbol `{symbol}` has ambiguous providers/exports: {} {:?}, {} {:?}",
                        previous.provider, previous.target, import.provider, import.target
                    ));
                }
            } else {
                selected = Some(import);
            }
        }
        if selected.is_none()
            && let Some((actual, _)) = self
                .incompatible_symbols
                .iter()
                .find(|(_, name)| name == symbol)
        {
            return Err(format!(
                "import symbol `{symbol}` CPU 0x{actual:04x} does not match interpreter CPU 0x{:04x}",
                self.expected
            ));
        }
        Ok(selected)
    }
}

struct Record {
    machine: u16,
    provider: String,
    import: Option<(String, ImportTarget)>,
}

impl Record {
    fn descriptor(machine: u16, provider: String) -> Self {
        Self {
            machine,
            provider,
            import: None,
        }
    }
}

fn symbol(data: &[u8]) -> Result<String, String> {
    if data.is_empty() || data.iter().any(|&byte| !(32..127).contains(&byte)) {
        return Err("invalid or empty short import symbol/export name".into());
    }
    String::from_utf8(data.to_vec()).map_err(|_| "invalid short import symbol/export name".into())
}

// Parse and validate before selecting an ABI. An archive may contain separate
// ARM64EC and ARM64 imports; their machine tags must never be conflated.
fn object(data: &[u8]) -> Result<Vec<Record>, String> {
    if data.len() < 20 {
        if data.starts_with(&[0, 0, 0xff, 0xff])
            || data.get(..2).is_some_and(|bytes| {
                matches!(
                    u16::from_le_bytes([bytes[0], bytes[1]]),
                    0x14c | 0x8664 | 0xaa64 | 0xa641
                )
            })
        {
            return Err("truncated COFF import object header".into());
        }
        return Ok(Vec::new());
    }
    if u16_at(data, 0)? == 0 && u16_at(data, 2)? == 0xffff {
        // BigObj has a different header; it is not a short import object.
        if u16_at(data, 4)? != 0 {
            return Ok(Vec::new());
        }
        let actual = u16_at(data, 6)?;
        let flags = u16_at(data, 18)?;
        if flags & !0x1f != 0 || flags & 3 > 2 || (flags >> 2) & 7 > 4 {
            return Err("invalid short import type/name flags".into());
        }
        let payload = bytes(data, 20, u32_at(data, 12)? as usize)?;
        let (public, dll) = terminated(payload)?;
        let public = symbol(public)?;
        let (_, rest) = terminated(dll)?;
        let name_type = (flags >> 2) & 7;
        let (target, rest) = match name_type {
            0 => (ImportTarget::Ordinal(u16_at(data, 16)?), rest),
            1 => (ImportTarget::Name(public.clone()), rest),
            2 | 3 => {
                let export = public.strip_prefix(['?', '@', '_']).unwrap_or(&public);
                let export = if name_type == 3 {
                    export.split('@').next().unwrap()
                } else {
                    export
                };
                (ImportTarget::Name(symbol(export.as_bytes())?), rest)
            }
            4 => {
                let (export, rest) = terminated(rest)?;
                (ImportTarget::Name(symbol(export)?), rest)
            }
            _ => unreachable!("validated import name type"),
        };
        if !rest.is_empty() {
            return Err("unexpected short import payload after names".into());
        }
        return Ok(vec![Record {
            machine: actual,
            provider: provider(dll)?,
            import: Some((public, target)),
        }]);
    }
    let actual = u16_at(data, 0)?;
    if !matches!(actual, 0x14c | 0x8664 | 0xaa64 | 0xa641) {
        return Ok(Vec::new());
    }
    let count = u16_at(data, 2)? as usize;
    let start = 20 + u16_at(data, 16)? as usize;
    let table = bytes(data, start, count * 40)?;
    let mut found = Vec::new();
    for header in table.as_chunks::<40>().0 {
        let name = &header[..8];
        let raw = || {
            bytes(
                data,
                u32_at(header, 20)? as usize,
                u32_at(header, 16)? as usize,
            )
        };
        // GNU's DLL-name member is a relocation-free .idata$7 section. Per-symbol
        // .idata$7 references have relocations and are not provider names.
        if name == b".idata$7" && u16_at(header, 32)? == 0 {
            let payload = raw()?;
            if payload.first().is_some_and(|&b| b != 0) {
                found.push(Record::descriptor(actual, provider(payload)?));
            }
        }
        // Long MS COFF descriptors point at the DLL-name section through the
        // Name RVA relocation at offset12. Never interpret a hint/name as a DLL.
        if name == b".idata$2" {
            let descriptor = raw()?;
            let relocs = bytes(
                data,
                u32_at(header, 24)? as usize,
                u16_at(header, 32)? as usize * 10,
            )?;
            for reloc in relocs.as_chunks::<10>().0 {
                if u32_at(reloc, 0)? != 12 {
                    continue;
                }
                let index = u32_at(reloc, 4)? as usize;
                if index >= u32_at(data, 12)? as usize {
                    return Err("COFF import relocation symbol out of bounds".into());
                }
                let symbols = u32_at(data, 8)? as usize;
                let symbol = bytes(
                    data,
                    symbols
                        .checked_add(index.checked_mul(18).ok_or("COFF symbol overflow")?)
                        .ok_or("COFF symbol overflow")?,
                    18,
                )?;
                let section = u16_at(symbol, 12)? as i16;
                if section == 0 {
                    continue;
                } // GNU's name is in another member.
                if section < 0 || section as usize > count {
                    return Err("invalid COFF import name section".into());
                }
                let target = &table[(section as usize - 1) * 40..section as usize * 40];
                if &target[..8] != b".idata$6" && &target[..8] != b".idata$7" {
                    return Err("COFF import name relocation has no name section".into());
                }
                let contents = bytes(
                    data,
                    u32_at(target, 20)? as usize,
                    u32_at(target, 16)? as usize,
                )?;
                let offset = (u32_at(symbol, 8)? as usize)
                    .checked_add(u32_at(descriptor, 12)? as usize)
                    .ok_or("COFF import name overflow")?;
                found.push(Record::descriptor(
                    actual,
                    provider(
                        contents
                            .get(offset..)
                            .ok_or("COFF import name out of bounds")?,
                    )?,
                ));
            }
        }
    }
    Ok(found)
}

/// Read one real archive's provider records. Empty means no supported imports;
/// multiple providers are returned explicitly, never collapsed to a fake handle.
pub(crate) fn archive(data: &[u8], expected: u16) -> Result<Archive, String> {
    if !data.starts_with(b"!<arch>\n") {
        return Err("not a COFF import archive".into());
    }
    let mut cursor = 8;
    let mut result: Vec<String> = Vec::new();
    let mut incompatible = None;
    let mut imports = Vec::new();
    let mut incompatible_symbols = Vec::new();
    while cursor < data.len() {
        let header = bytes(data, cursor, 60)?;
        if &header[58..] != b"`\n" {
            return Err("invalid COFF archive member trailer".into());
        }
        let length = std::str::from_utf8(&header[48..58])
            .map_err(|_| "invalid archive member length")?
            .trim();
        if length.is_empty() || !length.bytes().all(|c| c.is_ascii_digit()) {
            return Err("invalid archive member length".into());
        }
        let size = length
            .parse::<usize>()
            .map_err(|_| "archive member length overflow")?;
        cursor += 60;
        let mut payload = bytes(data, cursor, size)?;
        let name = std::str::from_utf8(&header[..16])
            .map_err(|_| "invalid archive member name")?
            .trim_end();
        if let Some(len) = name.strip_prefix("#1/") {
            let len = len
                .parse::<usize>()
                .map_err(|_| "invalid extended member name")?;
            payload = payload.get(len..).ok_or("truncated extended member name")?;
        }
        if !matches!(name, "/" | "//" | "/SYM64/") {
            for record in object(payload)? {
                let actual = record.machine;
                let name = record.provider;
                if actual != expected {
                    incompatible.get_or_insert(actual);
                    if let Some((symbol, _)) = record.import {
                        incompatible_symbols.push((actual, symbol));
                    }
                    continue;
                }
                if let Some((symbol, target)) = record.import {
                    imports.push(Import {
                        symbol,
                        provider: name.clone(),
                        target,
                    });
                }
                if !result.iter().any(|known| known.eq_ignore_ascii_case(&name)) {
                    result.push(name);
                }
            }
        }
        cursor = cursor.checked_add(size).ok_or("archive member overflow")?;
        if size % 2 != 0 {
            // PE/COFF documents IMAGE_ARCHIVE_PAD as newline; Microsoft SDK
            // archives also use NUL. Both occupy the same required alignment
            // byte. Never consume payload or a following header as padding.
            let pad = bytes(data, cursor, 1)?[0];
            if !matches!(pad, 0 | b'\n') {
                return Err(format!(
                    "invalid archive member padding 0x{pad:02x} at offset {cursor}"
                ));
            }
            cursor += 1;
        }
    }
    if result.is_empty()
        && let Some(actual) = incompatible
    {
        return Err(format!(
            "import provider CPU 0x{actual:04x} does not match interpreter CPU 0x{expected:04x}"
        ));
    }
    Ok(Archive {
        providers: result,
        imports,
        expected,
        incompatible_symbols,
    })
}

#[cfg(test)]
pub(crate) fn providers(data: &[u8], expected: u16) -> Result<Vec<String>, String> {
    archive(data, expected).map(|archive| archive.providers)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Candidate {
    Archive(PathBuf),
    Dll(PathBuf),
}

/// Match source/configured/SDK location precedence; within each location an
/// import archive determines provider identity before a guessed DLL basename.
pub(crate) fn candidates(
    name: &str,
    system: bool,
    base: &str,
    dirs: &[PathBuf],
    gnu: bool,
) -> Result<Vec<Candidate>, String> {
    if name.contains('\0') {
        return Err("native library name contains NUL".into());
    }
    let path = Path::new(name);
    let file = path
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or("empty native library name")?;
    let lower = file.to_ascii_lowercase();
    let explicit = lower.ends_with(".dll") || lower.ends_with(".drv") || lower.ends_with(".ocx");
    let archive = lower.ends_with(".lib") || lower.ends_with(".a");
    if path.is_absolute() {
        return Ok(vec![if archive {
            Candidate::Archive(path.into())
        } else {
            Candidate::Dll(path.into())
        }]);
    }
    let mut locations = Vec::new();
    if !system {
        locations.push((
            Path::new(base).join(path.parent().unwrap_or(Path::new(""))),
            false,
        ));
    }
    locations.extend(dirs.iter().cloned().map(|dir| (dir, true)));
    let mut out = Vec::new();
    for (dir, strip_lib) in locations {
        if explicit || archive {
            out.push(if archive {
                Candidate::Archive(dir.join(file))
            } else {
                Candidate::Dll(dir.join(file))
            });
            continue;
        }
        let stem = if strip_lib {
            file.strip_prefix("lib").unwrap_or(file)
        } else {
            file
        };
        let archives = if gnu {
            vec![
                format!("lib{stem}.dll.a"),
                format!("lib{stem}.a"),
                format!("{stem}.lib"),
                format!("lib{stem}.lib"),
            ]
        } else {
            vec![
                format!("{stem}.lib"),
                format!("lib{stem}.lib"),
                format!("lib{stem}.dll.a"),
                format!("lib{stem}.a"),
            ]
        };
        out.extend(
            archives
                .into_iter()
                .map(|name| Candidate::Archive(dir.join(name))),
        );
        out.push(Candidate::Dll(dir.join(format!("{stem}.dll"))));
        out.push(Candidate::Dll(dir.join(format!("lib{stem}.dll"))));
    }
    if archive {
        out.push(Candidate::Archive(path.into()));
    } else if explicit {
        out.push(Candidate::Dll(path.into()));
    } else {
        out.push(Candidate::Dll(PathBuf::from(format!("{name}.dll"))));
        let bare = name.strip_prefix("lib").unwrap_or(name);
        if bare != name {
            out.push(Candidate::Dll(PathBuf::from(format!("{bare}.dll"))));
        }
    }
    Ok(out)
}
