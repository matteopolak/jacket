//! The generated metaprogram behind `jaic +Plugin file.jai`.

use std::path::{Path, PathBuf};

/// One `+Plugin` the metaprogram imports.
pub struct PluginImport {
    /// The module name, without its module parameters.
    pub module: String,
    /// The module parameter list as it follows `#import "Name"`, parentheses included
    /// (`(A=1)`), or empty.
    pub import_parameters: String,
    /// The options passed to the plugin's `init` hook.
    pub options: Vec<String>,
}

/// What the generated metaprogram compiles and how it names the result.
pub struct PluginProgram<'a> {
    pub plugins: &'a [PluginImport],
    /// The program the plugins watch.
    pub main_file: &'a Path,
    /// `-exe`: the executable's name; it may carry a directory.
    pub exe: Option<&'a str>,
    /// `-output_path`.
    pub output_path: Option<&'a Path>,
    /// Source text added to the program's workspace after `main_file`, in order (`-add`,
    /// `-run`).
    pub added_sources: &'a [String],
    /// Whether the WASI runtime is added to the program's sources.
    pub wasi: bool,
}

/// A Jai string literal with `text` as its value.
fn quote(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

/// The metaprogram `+Plugin` stands for: import each plugin module, then compile the main file
/// in a workspace using the public Compiler lifecycle procedures.
pub fn plugin_metaprogram(program: &PluginProgram) -> String {
    let lossy = |p: Option<&std::ffi::OsStr>| p.map(|n| n.to_string_lossy().into_owned());
    // `-exe` and `-output_path` name the executable; otherwise it is named after the file, next
    // to it.
    let target = match (program.exe, program.output_path) {
        (Some(exe), Some(dir)) => Some(dir.join(exe)),
        (Some(exe), None) => Some(PathBuf::from(exe)),
        (None, Some(dir)) => Some(dir.join(program.main_file.file_stem().unwrap_or_default())),
        (None, None) => None,
    };
    let (name, dir) = match &target {
        Some(out) => (
            lossy(out.file_name()),
            lossy(out.parent().map(|d| d.as_os_str())),
        ),
        None => (lossy(program.main_file.file_stem()), None),
    };
    let plugins = program.plugins;
    let mut text = String::from("Compiler :: #import \"Compiler\";\n");
    for (i, plugin) in plugins.iter().enumerate() {
        text += &format!(
            "__plugin_{i} :: #import {}{};\n",
            quote(&plugin.module),
            plugin.import_parameters
        );
    }
    text += "#scope_file\n__jaic_plugin_failure :: (plugins: [] *Compiler.Metaprogram_Plugin, initialized: s64, w: Workspace, message: string) {\n    for p: plugins {\n        if it_index >= initialized break;\n        if p.shutdown p.shutdown(p);\n    }\n    Compiler.compiler_set_workspace_status(.FAILED, w);\n    Compiler.compiler_report(\"\", 0, 0, message);\n}\n";
    text += &format!(
        "#run,stallable {{\n    w := Compiler.compiler_create_workspace(\"Target Program\");\n    plugins: [{}] *Compiler.Metaprogram_Plugin;\n",
        plugins.len()
    );
    text += &format!(
        "    build := Compiler.get_build_options();\n    metaprogram := build;\n    metaprogram.output_type = .NO_OUTPUT;\n    Compiler.set_build_options(metaprogram);\n    build.output_executable_name = {};\n    build.output_path = {};\n    Compiler.set_build_options(build, w);\n",
        quote(&name.unwrap_or_default()),
        quote(&dir.unwrap_or_default()),
    );
    for (i, plugin) in plugins.iter().enumerate() {
        text += &format!(
            "    plugins[{i}] = __plugin_{i}.get_plugin();\n    if plugins[{i}] == null __jaic_plugin_failure(plugins, {i}, w, \"plugin get_plugin returned null\");\n    plugins[{i}].workspace = w;\n"
        );
        let options = plugin
            .options
            .iter()
            .map(|s| quote(s))
            .collect::<Vec<_>>()
            .join(", ");
        text += &format!(
            "    options_{i}: [{}] string = .[{options}];\n",
            plugin.options.len()
        );
        if !plugin.options.is_empty() {
            text += &format!(
                "    if !plugins[{i}].init __jaic_plugin_failure(plugins, {}, w, \"plugin has options but no init hook\");\n",
                i + 1
            );
        }
        text += &format!(
            "    if plugins[{i}].init && !plugins[{i}].init(plugins[{i}], options_{i}) __jaic_plugin_failure(plugins, {}, w, \"plugin initialization failed\");\n",
            i + 1
        );
    }
    text += "    flags: Compiler.Intercept_Flags;\n    for p: plugins if p.before_intercept p.before_intercept(p, *flags);\n    Compiler.compiler_begin_intercept(w, flags);\n    for p: plugins if p.add_source p.add_source(p);\n";
    text += &format!(
        "    Compiler.add_build_file({}, w);\n",
        quote(&program.main_file.to_string_lossy())
    );
    for added in program.added_sources {
        text += &format!("    Compiler.add_build_string({}, w);\n", quote(added));
    }
    if program.wasi {
        text += &format!(
            "    Compiler.add_build_string({}, w);\n",
            quote(super::WASI_RUNTIME_IMPORT)
        );
    }
    text += "    failed := false;\n    while true {\n        message := Compiler.compiler_wait_for_message();\n        if message == null Compiler.compiler_report(\"\", 0, 0, \"plugin workspace ended without COMPLETE\");\n        for p: plugins if p.message p.message(p, message);\n        if message.workspace == w && message.kind == .COMPLETE {\n            complete := cast(*Compiler.Message_Complete) message;\n            failed = complete.error_code != .NONE;\n            break;\n        }\n    }\n    Compiler.compiler_end_intercept(w);\n    for p: plugins if p.finish p.finish(p);\n    for p: plugins if p.shutdown p.shutdown(p);\n    if failed Compiler.compiler_set_workspace_status(.FAILED, w);\n}\n";
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plugin(module: &str, parameters: &str, options: &[&str]) -> PluginImport {
        PluginImport {
            module: module.into(),
            import_parameters: parameters.into(),
            options: options.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn generate(
        plugins: &[PluginImport],
        exe: Option<&str>,
        output_path: Option<&Path>,
        added: &[String],
        wasi: bool,
    ) -> String {
        plugin_metaprogram(&PluginProgram {
            plugins,
            main_file: Path::new("/src/game.jai"),
            exe,
            output_path,
            added_sources: added,
            wasi,
        })
    }

    #[test]
    fn imports_each_plugin_with_its_module_parameters() {
        let plugins = [plugin("Check", "", &[]), plugin("Lint", "(LEVEL=2)", &[])];
        let text = generate(&plugins, None, None, &[], false);
        assert!(text.starts_with("Compiler :: #import \"Compiler\";\n"));
        assert!(text.contains("__plugin_0 :: #import \"Check\";\n"));
        assert!(text.contains("__plugin_1 :: #import \"Lint\"(LEVEL=2);\n"));
        assert!(text.contains("plugins: [2] *Compiler.Metaprogram_Plugin;"));
    }

    #[test]
    fn options_need_an_init_hook() {
        let plugins = [plugin("Check", "", &["-strict", "a\"b"])];
        let text = generate(&plugins, None, None, &[], false);
        assert!(text.contains("options_0: [2] string = .[\"-strict\", \"a\\\"b\"];"));
        assert!(text.contains("if !plugins[0].init __jaic_plugin_failure(plugins, 1, w,"));
        let plain = generate(&[plugin("Check", "", &[])], None, None, &[], false);
        assert!(plain.contains("options_0: [0] string = .[];"));
        assert!(!plain.contains("has options but no init hook"));
    }

    #[test]
    fn the_executable_is_named_after_the_file_unless_told_otherwise() {
        let plugins = [plugin("Check", "", &[])];
        let default = generate(&plugins, None, None, &[], false);
        assert!(default.contains("build.output_executable_name = \"game\";"));
        assert!(default.contains("build.output_path = \"\";"));
        let exe = generate(&plugins, Some("app"), None, &[], false);
        assert!(exe.contains("build.output_executable_name = \"app\";"));
        let dir = generate(&plugins, None, Some(Path::new("/out")), &[], false);
        assert!(dir.contains("build.output_executable_name = \"game\";"));
        assert!(dir.contains("build.output_path = \"/out\";"));
        let both = generate(&plugins, Some("app"), Some(Path::new("/out")), &[], false);
        assert!(both.contains("build.output_executable_name = \"app\";"));
        assert!(both.contains("build.output_path = \"/out\";"));
    }

    #[test]
    fn adds_the_program_then_added_code_then_the_wasi_runtime() {
        let plugins = [plugin("Check", "", &[])];
        let added = ["#run print(\"x\\n\");\n".to_string()];
        let text = generate(&plugins, None, None, &added, true);
        let file = text
            .find("Compiler.add_build_file(\"/src/game.jai\", w);")
            .unwrap();
        let extra = text
            .find("Compiler.add_build_string(\"#run print(\\\"x\\\\n\\\");\\n\", w);")
            .unwrap();
        let wasi = text
            .find(&format!(
                "Compiler.add_build_string({}, w);",
                quote(super::super::WASI_RUNTIME_IMPORT)
            ))
            .unwrap();
        assert!(file < extra && extra < wasi);
        let without = generate(&plugins, None, None, &[], false);
        assert_eq!(without.matches("add_build_string").count(), 0);
    }

    #[test]
    fn quoting_escapes_what_a_string_literal_cannot_hold() {
        assert_eq!(quote("a\\b\"c\nd\re"), "\"a\\\\b\\\"c\\nd\\re\"");
    }
}
