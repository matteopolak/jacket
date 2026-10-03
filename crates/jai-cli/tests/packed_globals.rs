//! Exercise the real CLI linker path with an unaligned pointer relocation.
#![cfg(target_os = "macos")]
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn packed_global_string_keeps_its_relocatable_pointer() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-cli-packed-global-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&fixture.0).unwrap();
    let input = fixture.0.join("main.jai");
    let output = fixture.0.join("program");
    fs::write(&input, "Packed :: struct #no_padding { tag:u8 = 1; label:string = \"42\"; } value:Packed; main :: () -> int { if value.tag == 1 && value.label[0] == 52 && value.label[1] == 50 return 0; return 1; }").unwrap();
    let build = Command::new(env!("CARGO_BIN_EXE_jai-rs"))
        .arg("build")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(Command::new(&output).status().unwrap().success());
}
