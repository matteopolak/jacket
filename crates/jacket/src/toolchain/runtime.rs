//! `jacket toolchain add <freestanding target>`: compiler-rt's builtins (64-bit division, soft
//! float, outline atomics), compiled for the target with the installed clang straight from the
//! pinned source list. No cmake, ninja, git or shell: it runs the same on every host.
use jaic::target::{Target, TargetAbi, TargetCpu};
use jaic_toolchain::{Lookup, Missing, cache, pins};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// How one freestanding CPU is compiled.
struct Flags {
    /// clang's `--target`: compiler-rt calls Cortex-M cores `armv7em` and `armv8m.main`, and
    /// builds them soft float. The program's hard-float ABI is a calling convention for its own
    /// functions; LLVM calls the helpers (`__muldf3`) with the base one.
    triple: String,
    args: Vec<String>,
}

fn flags(target: Target) -> Option<Flags> {
    let (triple, args): (String, Vec<&str>) = match (target.cpu, target.abi) {
        (TargetCpu::Arm64, _) => (
            "aarch64-none-elf".into(),
            vec![
                "-DENABLE_BAREMETAL_AARCH64_FMV",
                "-DHAS_ASM_LSE",
                "-DCOMPILER_RT_HAS_FLOAT16",
            ],
        ),
        (TargetCpu::Arm, TargetAbi::CortexM(core)) => {
            let triple = format!("{}-none-eabi", core.compiler_rt_arch());
            let cpu = format!("-mcpu={}", core.llvm_cpu());

            return Some(Flags {
                triple,
                args: [
                    cpu.as_str(),
                    "-mfloat-abi=soft",
                    "-DCOMPILER_RT_HAS_FLOAT16",
                ]
                .map(String::from)
                .to_vec(),
            });
        }
        // The Arduino Uno's core: objects for another AVR ISA level are refused at link time.
        (TargetCpu::Avr, _) => ("avr-none-elf".into(), vec!["-mmcu=atmega328p"]),
        _ => return None,
    };

    Some(Flags {
        triple,
        args: args.into_iter().map(String::from).collect(),
    })
}

/// A build of the builtins: how long it took and what it was built from.
pub struct Built {
    pub source: String,
    pub took: Duration,
}

/// The tools a build needs, or what is missing.
fn tools(lookup: &Lookup) -> Result<(PathBuf, PathBuf), Missing> {
    use jaic_toolchain::package::{Need, PackageManager};
    let note = |need: Need| need.note(lookup.host.os(), PackageManager::detect(lookup));
    let clang = lookup
        .llvm_tool("clang")
        .ok_or_else(|| Missing::new("no clang to compile the builtins", vec![note(Need::Clang)]))?;
    let ar = lookup.llvm_tool("llvm-ar").ok_or_else(|| {
        Missing::new(
            "no llvm-ar to archive the builtins",
            vec![note(Need::Clang)],
        )
    })?;

    Ok((clang, ar))
}

/// Builds the builtins for the freestanding `target` into the cache and returns them.
/// `progress` is told how many files are done of how many.
///
/// ```text
/// jacket toolchain add thumbv7em-none-eabihf
/// ```
pub fn add(
    target: Target,
    lookup: &Lookup,
    progress: &(dyn Fn(usize, usize) + Sync),
) -> Result<Built, String> {
    let family = pins::compiler_rt_family(target)
        .ok_or_else(|| format!("jacket cannot build builtins for {target}"))?;
    let flags =
        flags(target).ok_or_else(|| format!("jacket cannot build builtins for {target}"))?;
    let (clang, ar) = tools(lookup).map_err(|m| m.to_string())?;
    let pin = pins::compiler_rt(family);
    let root = cache::root(lookup.host).ok_or("no cache folder: HOME is not set")?;
    let tarball = root
        .join("downloads")
        .join(format!("compiler-rt-{}.src.tar.xz", pin.version));

    super::fetch::pinned(pin.url, pin.sha256, &tarball)?;
    let sources = root.join("compiler-rt").join(pin.version);
    let builtins = sources.join(format!("compiler-rt-{}.src/lib/builtins", pin.version));

    if !builtins.join("CMakeLists.txt").is_file() {
        let _ = std::fs::remove_dir_all(&sources);

        std::fs::create_dir_all(&sources).map_err(|e| format!("{}: {e}", sources.display()))?;
        super::fetch::untar(&tarball, &sources)?;
    }

    let out = cache::builtins_dir(lookup.host, target).ok_or("no cache folder: HOME is not set")?;
    let work = out.with_file_name("builtins.partial");
    let _ = std::fs::remove_dir_all(&work);

    std::fs::create_dir_all(work.join("obj")).map_err(|e| format!("{}: {e}", work.display()))?;
    let started = Instant::now();
    let objects = compile(
        &clang,
        &flags,
        &builtins,
        &pin.sources,
        &work.join("obj"),
        progress,
    );
    let objects = match objects {
        Ok(objects) => objects,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&work);
            return Err(error);
        }
    };
    let archive = work.join(cache::BUILTINS_FILE);
    let result = archive_objects(&ar, &archive, &objects);
    let _ = std::fs::remove_dir_all(work.join("obj"));

    if let Err(error) = result {
        let _ = std::fs::remove_dir_all(&work);
        return Err(error);
    }

    let source = format!("compiler-rt {}", pin.version);
    let mark = format!(
        "{}{}-{}",
        cache::SOURCE_MARK,
        source.replace(' ', "-"),
        pin.stamp()
    );

    std::fs::write(work.join(&mark), format!("{} {}\n", pin.url, pin.sha256))
        .map_err(|e| format!("{}: {e}", work.display()))?;
    let _ = std::fs::remove_dir_all(&out);

    std::fs::rename(&work, &out).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(Built {
        source,
        took: started.elapsed(),
    })
}

/// Compiles each source, in parallel, to its own object in `obj`, in the order listed.
fn compile(
    clang: &Path,
    flags: &Flags,
    builtins: &Path,
    sources: &[(&str, Vec<&str>)],
    obj: &Path,
    progress: &(dyn Fn(usize, usize) + Sync),
) -> Result<Vec<PathBuf>, String> {
    let done = AtomicUsize::new(0);

    crate::parallel::try_map(
        sources,
        crate::parallel::threads(),
        |index, (source, defines)| {
            let object = obj.join(format!("{index:04}.o"));

            compile_one(clang, flags, builtins, source, defines, &object)?;
            progress(done.fetch_add(1, Ordering::Relaxed) + 1, sources.len());
            Ok(object)
        },
    )
}

fn compile_one(
    clang: &Path,
    flags: &Flags,
    builtins: &Path,
    source: &str,
    defines: &[&str],
    object: &Path,
) -> Result<(), String> {
    let mut command = Command::new(clang);

    command
        .arg(format!("--target={}", flags.triple))
        .args(&flags.args)
        .args([
            "-O3",
            "-ffreestanding",
            "-fno-builtin",
            "-fomit-frame-pointer",
        ])
        .args(["-fvisibility=hidden", "-DVISIBILITY_HIDDEN"])
        .arg(format!("-I{}", builtins.display()));

    if source.ends_with(".c") {
        command.arg("-std=c11");
    } else if source.ends_with(".cpp") {
        command.args(["-std=c++17", "-fno-exceptions", "-fno-rtti"]);
    }

    for define in defines {
        command.arg(format!("-D{define}"));
    }

    let output = command
        .arg("-c")
        .arg(builtins.join(source))
        .arg("-o")
        .arg(object)
        .output()
        .map_err(|e| format!("could not run `{}`: {e}", clang.display()))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "compiling {source} failed:\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn archive_objects(ar: &Path, archive: &Path, objects: &[PathBuf]) -> Result<(), String> {
    let output = Command::new(ar)
        .arg("rcs")
        .arg(archive)
        .args(objects)
        .output()
        .map_err(|e| format!("could not run `{}`: {e}", ar.display()))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "archiving the builtins failed:\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

/// The text `jacket toolchain add` prints for a finished build.
pub fn finished(built: &Built) -> String {
    format!(
        "{} (built in {:.1}s)",
        built.source,
        built.took.as_secs_f64()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cortex_m_is_built_soft_float_as_compiler_rts_own_architecture() {
        let m4 = flags(Target::parse("thumbv7em-none-eabihf").unwrap()).unwrap();

        assert_eq!(m4.triple, "armv7em-none-eabi");
        assert!(m4.args.contains(&"-mcpu=cortex-m4".to_string()));
        assert!(m4.args.contains(&"-mfloat-abi=soft".to_string()));
        let m33 = flags(Target::parse("thumbv8m.main-none-eabihf").unwrap()).unwrap();

        assert_eq!(m33.triple, "armv8m.main-none-eabi");
        assert!(m33.args.contains(&"-mcpu=cortex-m33".to_string()));
    }

    #[test]
    fn avr_is_built_for_the_uno_core_and_other_targets_have_no_builtins() {
        let avr = flags(Target::parse("avr-none-elf").unwrap()).unwrap();

        assert_eq!(avr.args, ["-mmcu=atmega328p"]);
        assert!(flags(Target::parse("x86_64-unknown-linux-gnu").unwrap()).is_none());
    }

    #[test]
    fn finished_says_how_long_it_took() {
        let built = Built {
            source: "compiler-rt 21.1.8".into(),
            took: Duration::from_millis(14_240),
        };

        assert_eq!(finished(&built), "compiler-rt 21.1.8 (built in 14.2s)");
    }
}
