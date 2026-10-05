//! Fixtures shared by the `jaic` integration tests.
use std::path::{Path, PathBuf};

/// A plugin module for `-plug`: `init` takes the options jaic does not know, `add_source`
/// adds code to the program, `message` sees typechecked procedures and `finish` reports.
const ECHO_PLUGIN: &str = r#"#import "Basic";
#import "Compiler";
Echo :: struct { #as using base: Metaprogram_Plugin; prefix: string; typechecked: int; }
get_plugin :: () -> *Metaprogram_Plugin {
    p := New(Echo);
    p.init = (p: *Metaprogram_Plugin, options: [] string) -> bool {
        echo := cast(*Echo) p;
        for options  if it == "-shout"  echo.prefix = "LOUD ";
        return !array_find(options, "-refuse");
    };
    p.add_source = (p: *Metaprogram_Plugin) {
        add_build_string("from_plugin :: () -> int { return 42; }", p.workspace);
    };
    p.message = (p: *Metaprogram_Plugin, message: *Message) {
        echo := cast(*Echo) p;
        if message.kind == .TYPECHECKED  echo.typechecked += 1;
    };
    p.finish = (p: *Metaprogram_Plugin) {
        echo := cast(*Echo) p;
        print("%finished, typechecked: %\n", echo.prefix, echo.typechecked > 0);
    };
    return p;
}
"#;

/// Write `dir/uses_plugin.jai`, which calls the procedure the plugin adds, and the plugin
/// as `dir/modules/Echo_Plugin`.
pub fn write_plugin_program(dir: &Path) -> PathBuf {
    std::fs::create_dir_all(dir.join("modules/Echo_Plugin")).unwrap();
    std::fs::write(dir.join("modules/Echo_Plugin/module.jai"), ECHO_PLUGIN).unwrap();
    let source = dir.join("uses_plugin.jai");
    std::fs::write(
        &source,
        "#import \"Basic\";\nmain :: () { print(\"%\\n\", from_plugin()); }\n",
    )
    .unwrap();
    source
}
