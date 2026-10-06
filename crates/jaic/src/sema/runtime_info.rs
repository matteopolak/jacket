//! `__runtime_info`: the compiler-provided `Runtime_Info` behind the Compiler
//! module's `get_runtime_info()` (type table and global data segments, used
//! e.g. by the memory debugger to scan globals for live pointers).
//!
//! The global is created on first reference and filled by `finish_program`
//! once every global and type descriptor of the program exists. Layouts follow
//! `stdlib/Compiler/workspace.jai`: `Runtime_Info { type_table: [] *Type_Info;
//! global_data_info: *Global_Data_Info; }`, `Global_Data_Info { version_stamp:
//! u64; segment_info: [] Global_Data_Segment_Info; }`,
//! `Global_Data_Segment_Info { segment_tag: u16; data: [] u8; }`.
use super::*;

const SEGMENT_DATA: u16 = 1;
const SEGMENT_RDATA: u16 = 2;
const SEGMENT_INFO_SIZE: u64 = 24;

impl Compiler {
    pub(super) fn runtime_info_global(&mut self, _span: Span) -> Result<ir::GlobalId> {
        if let Some(g) = self.runtime_info {
            return Ok(g);
        }
        let g = self.data_global("__runtime_info", 24, Vec::new(), Vec::new());
        self.runtime_info = Some(g);
        Ok(g)
    }

    fn data_global(
        &mut self,
        name: &str,
        size: u64,
        init: Vec<u8>,
        relocs: Vec<ir::Reloc>,
    ) -> ir::GlobalId {
        self.program.add_global(ir::Global {
            name: name.into(),
            size,
            align: 8,
            init,
            relocs,
            read_only: false,
            export: None,
        })
    }

    /// `__jaic_build_directory: string #elsewhere;` in Runtime_Support: the directory the
    /// build started in (`crate::display_base`) with a trailing separator, or empty. Native
    /// failure reports strip it from source paths, so they read like the interpreter's.
    pub(super) fn build_directory_global(&mut self) -> ir::GlobalId {
        let mut text = crate::display_base()
            .map(|dir| dir.display().to_string())
            .unwrap_or_default();
        if !text.is_empty() && !text.ends_with(std::path::MAIN_SEPARATOR) {
            text.push(std::path::MAIN_SEPARATOR);
        }
        let bytes: Rc<[u8]> = Rc::from(text.as_bytes());
        let data = self.string_global(&bytes);
        let mut init = (text.len() as u64).to_le_bytes().to_vec();
        init.extend_from_slice(&[0; 8]);
        let reloc = ir::Reloc {
            offset: 8,
            target: ir::RelocTarget::Global(data),
            addend: 0,
        };
        self.data_global("__jaic_build_directory", 16, init, vec![reloc])
    }

    /// Fill `__runtime_info` (if the program uses it) from the final program.
    pub(super) fn fill_runtime_info(&mut self) {
        let Some(info) = self.runtime_info else {
            return;
        };
        let reloc = |offset: u64, target: ir::GlobalId| ir::Reloc {
            offset,
            target: ir::RelocTarget::Global(target),
            addend: 0,
        };
        let data_globals: Vec<(ir::GlobalId, u64, bool)> = self
            .program
            .globals
            .iter()
            .enumerate()
            .filter(|(_, g)| g.size > 0)
            .map(|(i, g)| (ir::GlobalId(i as u32), g.size, g.read_only))
            .collect();

        let mut types: Vec<(TypeId, ir::GlobalId)> =
            self.type_infos.iter().map(|(&t, &g)| (t, g)).collect();
        types.sort_by_key(|&(t, _)| t);
        let table_relocs = (0..types.len())
            .map(|i| reloc(i as u64 * 8, types[i].1))
            .collect();
        let table = self.data_global(
            "__runtime_info.type_table",
            types.len() as u64 * 8,
            vec![0; types.len() * 8],
            table_relocs,
        );

        let mut segments = vec![0u8; data_globals.len() * SEGMENT_INFO_SIZE as usize];
        let mut segment_relocs = Vec::new();
        for (i, &(global, size, read_only)) in data_globals.iter().enumerate() {
            let at = i * SEGMENT_INFO_SIZE as usize;
            let tag = if read_only {
                SEGMENT_RDATA
            } else {
                SEGMENT_DATA
            };
            segments[at..at + 2].copy_from_slice(&tag.to_le_bytes());
            segments[at + 8..at + 16].copy_from_slice(&size.to_le_bytes());
            segment_relocs.push(reloc(at as u64 + 16, global));
        }
        let segments = self.data_global(
            "__runtime_info.segments",
            segments.len() as u64,
            segments,
            segment_relocs,
        );

        let mut gdi = vec![0u8; 24];
        gdi[8..16].copy_from_slice(&(data_globals.len() as u64).to_le_bytes());
        let gdi = self.data_global(
            "__runtime_info.global_data_info",
            24,
            gdi,
            vec![reloc(16, segments)],
        );

        let mut bytes = vec![0u8; 24];
        bytes[0..8].copy_from_slice(&(types.len() as u64).to_le_bytes());
        let g = &mut self.program.globals[info.0 as usize];
        g.init = bytes;
        g.relocs = vec![reloc(8, table), reloc(16, gdi)];
    }
}
