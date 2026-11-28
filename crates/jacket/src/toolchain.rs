//! `jacket toolchain`: what each target needs from this machine, found by the same lookup a build
//! runs, so the listing and a build always agree. See `docs/native/toolchain-discovery.md`.
pub mod add;
mod fetch;
mod runtime;
mod sysroot;

use jaic::target::{Target as Triple, TargetOs};
use jaic_toolchain::builtins::find_builtins;
use jaic_toolchain::linker::{SysrootUse, find_linker, find_sysroot, host_target};
use jaic_toolchain::package::{Need, PackageManager};
use jaic_toolchain::{Found, Lookup, Missing, Overrides, System, tilde};

use crate::target::Target;

static SYSTEM: System = System;

/// The lookup over this machine, with the LLVM installs searched only when an LLVM program is.
pub fn lookup() -> Lookup<'static> {
    Lookup::new(&SYSTEM)
        .with_llvm_bins(|| jacket_config::llvm::bin_dirs(None))
        .with_llvm_prefixes(|| jacket_config::llvm::prefixes(None))
}

/// `runner` with `{sysroot}` replaced by the sysroot `target`'s programs run against: the
/// system's cross packages, else the one `jacket toolchain add` fetched. A runner without the
/// placeholder is returned as it is. The path is one word of the command, so it must have no
/// spaces.
///
/// ```text
/// runner = "qemu-arm -L {sysroot} {exe}"
/// ```
pub fn fill_sysroot(runner: &str, target: &Target) -> Result<String, String> {
    if !runner.contains("{sysroot}") {
        return Ok(runner.to_string());
    }

    let triple = target.effective_triple().ok_or_else(|| {
        format!(
            "`{}` has a `{{sysroot}}` in its runner but no triple",
            target.name
        )
    })?;
    let triple = Triple::parse(triple)?;
    let lookup = lookup();
    let found = find_sysroot(&lookup, triple, &Overrides::from_env(), SysrootUse::Run)
        .map_err(|missing| missing.to_string())?;
    let Some(found) = found else {
        let note = Need::CrossLinux {
            target: triple,
        }
        .note(lookup.host.os(), PackageManager::detect(&lookup));

        return Err(format!(
            "the runner of `{}` needs a sysroot for {triple}\nnote: {note}",
            target.name
        ));
    };

    Ok(runner.replace("{sysroot}", &found.program.to_string_lossy()))
}

/// One piece a target needs: found, missing with the notes that say how to get it, or not needed.
#[derive(Debug)]
pub enum Piece {
    Found(Found),
    Missing(Missing),
    NotNeeded,
}

/// A target and what this machine has for it.
#[derive(Debug)]
pub struct Row {
    pub name: String,
    pub from: &'static str,
    pub triple: String,
    pub linker: Piece,
    /// The sysroot of a Linux target, or the builtins of a bare-metal one.
    pub libraries: Piece,
    pub runner: Piece,
}

impl Row {
    fn pieces(&self) -> [(&'static str, &Piece); 3] {
        [
            ("linker", &self.linker),
            ("libraries", &self.libraries),
            ("runner", &self.runner),
        ]
    }

    /// Whether everything the target needs is here.
    pub fn complete(&self) -> bool {
        self.pieces()
            .iter()
            .all(|(_, piece)| !matches!(piece, Piece::Missing(_)))
    }
}

/// The row for the machine's own target.
pub fn host_row(lookup: &Lookup, overrides: &Overrides) -> Row {
    let triple = host_target(lookup.host);
    let mut row = triple_row(&triple.to_string(), triple, lookup, overrides);

    row.name = "host".to_string();
    row.from = "-";
    row
}

/// The row for a `--target` name.
pub fn target_row(target: &Target, lookup: &Lookup, overrides: &Overrides) -> Row {
    let mut row = match target.effective_triple().map(Triple::parse) {
        Some(Ok(triple)) => triple_row(&target.name, triple, lookup, overrides),
        // A triple jaic cannot build for is the target's problem, not the command's.
        Some(Err(error)) => Row {
            name: target.name.clone(),
            from: "-",
            triple: target.effective_triple().unwrap_or_default().to_string(),
            linker: Piece::Missing(Missing::new(error, Vec::new())),
            libraries: Piece::NotNeeded,
            runner: Piece::NotNeeded,
        },
        None => triple_row(&target.name, host_target(lookup.host), lookup, overrides),
    };

    row.from = if target.project {
        "project"
    } else {
        "built in"
    };
    row.runner = target
        .runner
        .as_deref()
        .and_then(|runner| runner.split_whitespace().next())
        .map_or(Piece::NotNeeded, |program| runner_piece(program, lookup));
    row
}

fn triple_row(name: &str, triple: Triple, lookup: &Lookup, overrides: &Overrides) -> Row {
    let linker = match find_linker(lookup, Some(triple), overrides) {
        Ok(found) => Piece::Found(found),
        Err(missing) => Piece::Missing(missing),
    };
    let host = host_target(lookup.host);
    let libraries = if triple.is_freestanding() && !triple.is_wasm() {
        match find_builtins(lookup, triple, overrides) {
            Ok(found) => Piece::Found(found),
            Err(missing) => Piece::Missing(missing),
        }
    } else if triple.os == TargetOs::Linux && triple != host {
        // Reported once, by the linker: a Linux target without a sysroot has no linker.
        match find_sysroot(lookup, triple, overrides, SysrootUse::Link) {
            Ok(found) => found.map_or(Piece::NotNeeded, Piece::Found),
            Err(missing) => Piece::Missing(missing),
        }
    } else {
        Piece::NotNeeded
    };

    Row {
        name: name.to_string(),
        from: "-",
        triple: triple.to_string(),
        linker,
        libraries,
        runner: Piece::NotNeeded,
    }
}

/// A runner's program: found on this machine, or missing with the note for its package.
pub fn runner_piece(program: &str, lookup: &Lookup) -> Piece {
    program_piece(program, lookup)
}

/// The program `name` as the lookup finds it, or the note that says how to get it.
pub fn program_piece(name: &str, lookup: &Lookup) -> Piece {
    if let Some(path) = lookup.which(name) {
        return Piece::Found(Found::new(path, jaic_toolchain::Origin::System));
    }

    let note = Need::for_program(name).note(lookup.host.os(), PackageManager::detect(lookup));
    Piece::Missing(Missing::new(format!("{name} is not installed"), vec![note]))
}

fn cell(piece: &Piece, lookup: &Lookup) -> String {
    match piece {
        Piece::Found(found) => found.describe(lookup),
        Piece::Missing(_) => "missing".to_string(),
        Piece::NotNeeded => "-".to_string(),
    }
}

/// The table `jacket toolchain` prints, then what each missing piece needs.
pub fn listing(rows: &[Row], lookup: &Lookup) -> String {
    let header = ["TARGET", "FROM", "TRIPLE", "LINKER", "LIBRARIES", "RUNNER"].map(String::from);
    let cells: Vec<[String; 6]> = rows
        .iter()
        .map(|row| {
            [
                row.name.clone(),
                row.from.to_string(),
                row.triple.clone(),
                cell(&row.linker, lookup),
                cell(&row.libraries, lookup),
                cell(&row.runner, lookup),
            ]
        })
        .collect();
    let widths: Vec<usize> = (0..6)
        .map(|c| {
            std::iter::once(&header)
                .chain(&cells)
                .map(|row| row[c].chars().count())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut text = String::new();

    for row in std::iter::once(&header).chain(&cells) {
        let line: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:<width$}"))
            .collect();

        text.push_str(line.join("  ").trim_end());
        text.push('\n');
    }

    text.push_str(&problems(rows));
    text
}

/// What `jacket toolchain` printed, and whether everything it looked for was found.
pub struct Report {
    pub text: String,
    pub complete: bool,
}

/// The machine's own target, then every target `--target` accepts.
pub fn list(
    config: Option<&jacket_config::Config>,
    lookup: &Lookup,
    overrides: &Overrides,
) -> Report {
    let mut rows = vec![host_row(lookup, overrides)];

    for target in crate::target::all(config) {
        rows.push(target_row(&target, lookup, overrides));
    }

    // Listing every board is not a failure on a machine that has none of their emulators.
    Report {
        text: listing(&rows, lookup),
        complete: true,
    }
}

/// The rows for `names` (targets of the project, built-in boards, or triples; the machine's own
/// when none), and `tools` as programs that must be installed. Incomplete when anything is
/// missing; a name that is none of those is an error ([`crate::target::named`]).
///
/// ```text
/// jacket toolchain check Mps2_An386 --tool qemu-system-arm
/// ```
pub fn check(
    names: &[String],
    tools: &[String],
    config: Option<&jacket_config::Config>,
    lookup: &Lookup,
    overrides: &Overrides,
) -> Result<Report, String> {
    let mut rows = Vec::new();

    for name in names {
        let target = crate::target::named(name, config)?;

        rows.push(target_row(&target, lookup, overrides));
    }

    if names.is_empty() && tools.is_empty() {
        rows.push(host_row(lookup, overrides));
    }

    let mut text = if rows.is_empty() {
        String::new()
    } else {
        listing(&rows, lookup)
    };
    let mut complete = rows.iter().all(Row::complete);

    for tool in tools {
        match program_piece(tool, lookup) {
            Piece::Found(found) => text.push_str(&format!("{tool}: {}\n", tilde(&found.program))),
            Piece::Missing(missing) => {
                complete = false;

                if !text.is_empty() {
                    text.push('\n');
                }

                text.push_str(&format!("{tool}: {missing}\n"));
            }
            Piece::NotNeeded => {}
        }
    }

    Ok(Report {
        text,
        complete,
    })
}

/// What each missing piece of `rows` needs, one block per target.
pub fn problems(rows: &[Row]) -> String {
    let mut text = String::new();

    for row in rows {
        for (_, piece) in row.pieces() {
            if let Piece::Missing(missing) = piece {
                text.push_str(&format!("\n{}: {missing}\n", row.name));
            }
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use jaic_toolchain::testing::FakeHost;

    use super::*;

    fn row_for(host: &FakeHost, name: &str, board: Option<&str>, runner: Option<&str>) -> Row {
        let lookup = Lookup::new(host);
        let target = Target {
            name: name.to_string(),
            board: board.map(str::to_string),
            runner: runner.map(str::to_string),
            ..Target::default()
        };

        target_row(&target, &lookup, &Overrides::default())
    }

    #[test]
    fn a_built_in_board_is_checked_for_its_triple() {
        let host = FakeHost::mac();
        let row = row_for(
            &host,
            "Mps2_An386",
            Some("Mps2_An386"),
            Some("qemu-system-arm -M x"),
        );

        assert_eq!(row.triple, "thumbv7em-none-eabihf");
        assert!(matches!(row.linker, Piece::Missing(_)));
        assert!(matches!(row.runner, Piece::Missing(_)));
        assert!(!row.complete());
    }

    #[test]
    fn a_missing_piece_prints_its_note_under_the_table() {
        let mut host = FakeHost::mac();

        host.file("/opt/homebrew/bin/brew");
        let lookup = Lookup::new(&host);
        let row = row_for(
            &host,
            "Arduino_Uno",
            Some("Arduino_Uno"),
            Some("qemu-system-avr -M uno"),
        );
        let text = listing(&[row], &lookup);

        assert!(text.starts_with("TARGET"), "{text}");
        assert!(
            text.contains("Arduino_Uno: no linker for avr-none-elf"),
            "{text}"
        );
        assert!(text.contains("note: "), "{text}");
        assert!(text.contains("brew install"), "{text}");
    }

    #[test]
    fn a_triple_jaic_cannot_build_for_is_a_missing_linker() {
        let host = FakeHost::mac();
        let target = Target {
            name: "pico".to_string(),
            project: true,
            triple: Some("thumbv6m-none-eabi".to_string()),
            ..Target::default()
        };
        let row = target_row(&target, &Lookup::new(&host), &Overrides::default());

        assert_eq!(row.triple, "thumbv6m-none-eabi");
        assert!(
            matches!(&row.linker, Piece::Missing(m) if m.what.contains("unsupported")),
            "{row:?}"
        );
    }

    #[test]
    fn a_found_runner_is_listed_by_path() {
        let mut host = FakeHost::mac();

        host.file("/opt/homebrew/bin/qemu-system-arm");
        let lookup = Lookup::new(&host);
        let piece = runner_piece("qemu-system-arm", &lookup);

        assert!(matches!(piece, Piece::Found(_)), "{piece:?}");
    }
}
