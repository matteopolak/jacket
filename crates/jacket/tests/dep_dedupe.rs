//! `[resolver] dedupe`: one copy of each shared module, checked on local repositories and by
//! compiling the result.
use crate::dep_support;
use dep_support::{Origin, Project, Run, git};
use std::collections::BTreeMap;
use std::path::Path;

/// What a version requires: the package, the range it asks for, and the tag its author checked
/// the nested copy out at.
#[derive(Clone)]
pub struct Need {
    pub dep: &'static str,
    pub range: &'static str,
    pub pinned: &'static str,
    pub scheme: Option<&'static str>,
    /// The author keeps a plain copy of the module instead of a submodule.
    pub vendored: bool,
}

pub fn need(dep: &'static str, range: &'static str, pinned: &'static str) -> Need {
    Need {
        dep,
        range,
        pinned,
        scheme: None,
        vendored: false,
    }
}

/// One version of a generated package.
#[derive(Clone)]
pub struct Version {
    pub tag: &'static str,
    pub needs: Vec<Need>,
    /// Text added to `module.jai` after the generated functions.
    pub body: &'static str,
    /// The line that opens `module.jai`, such as `#module_parameters(N := 1);`.
    pub head: &'static str,
    /// How a requirement is imported, such as `(N = 2)`.
    pub arguments: &'static str,
    pub files: Vec<(&'static str, &'static str)>,
}

pub fn version(tag: &'static str) -> Version {
    Version {
        tag,
        needs: Vec::new(),
        body: "",
        head: "",
        arguments: "",
        files: Vec::new(),
    }
}

impl Version {
    #[must_use]
    pub fn needs(mut self, needs: &[Need]) -> Self {
        self.needs = needs.to_vec();
        self
    }

    #[must_use]
    pub fn body(mut self, body: &'static str) -> Self {
        self.body = body;
        self
    }

    #[must_use]
    pub fn head(mut self, head: &'static str) -> Self {
        self.head = head;
        self
    }

    #[must_use]
    pub fn arguments(mut self, arguments: &'static str) -> Self {
        self.arguments = arguments;
        self
    }

    #[must_use]
    pub fn file(mut self, path: &'static str, text: &'static str) -> Self {
        self.files.push((path, text));
        self
    }
}

pub struct World {
    pub project: Project,
    pub libs: BTreeMap<String, Origin>,
}

/// `v1.4.0` as 140: a number each version's functions return, so a program shows which one it
/// was built with.
fn number(tag: &str) -> u32 {
    tag.trim_start_matches('v')
        .split('.')
        .fold(0, |n, part| n * 10 + part.parse::<u32>().unwrap_or(0))
}

fn module_text(name: &str, version: &Version) -> String {
    let mut text = String::new();
    if !version.head.is_empty() {
        text.push_str(version.head);
        text.push('\n');
    }
    for need in &version.needs {
        text.push_str(&format!("#import \"{}\"{};\n", need.dep, version.arguments));
    }
    let n = number(version.tag);
    text.push_str(&format!("{name}_id :: () -> s64 {{ return {n}; }}\n"));
    if name == "qlib" {
        text.push_str("Handle :: struct { n: s64; }\n");
        text.push_str(&format!(
            "make_handle :: () -> Handle {{ h: Handle; h.n = {n}; return h; }}\n"
        ));
    }
    for need in &version.needs {
        let dep = need.dep;
        text.push_str(&format!(
            "{name}_via_{dep} :: () -> s64 {{ return {dep}_id(); }}\n"
        ));
        if dep == "qlib" {
            text.push_str(&format!(
                "{name}_handle :: () -> Handle {{ return make_handle(); }}\n{name}_take :: (h: Handle) -> s64 {{ return h.n; }}\n"
            ));
        }
    }
    text.push_str(version.body);
    text
}

impl World {
    pub fn new(dedupe: bool) -> Self {
        let resolver = if dedupe {
            "[resolver]\ndedupe = true\n"
        } else {
            ""
        };
        Self {
            project: Project::new(&format!("[project]\nentry = [\"main.jai\"]\n{resolver}")),
            libs: BTreeMap::new(),
        }
    }

    /// Publishes `versions` of `name` in order; each one's requirements are nested submodules
    /// at the tags its author chose.
    pub fn package(&mut self, name: &str, versions: &[Version]) {
        for version in versions {
            let mut files = vec![
                ("module.jai".to_string(), module_text(name, version)),
                ("jacket.toml".to_string(), self.manifest(name, version)),
            ];
            files.extend(
                version
                    .files
                    .iter()
                    .map(|(p, t)| (p.to_string(), t.to_string())),
            );
            if !self.libs.contains_key(name) {
                let origin = Origin::new(
                    &self.project.root,
                    name,
                    &files
                        .iter()
                        .map(|(p, t)| (p.as_str(), t.as_str()))
                        .collect::<Vec<_>>(),
                );
                self.libs.insert(name.to_string(), origin);
            }
            let origin = &self.libs[name];
            for need in version.needs.iter().filter(|n| !n.vendored) {
                origin.nest(&self.libs[need.dep], need.dep, need.pinned);
            }
            let files: Vec<(&str, &str)> = files
                .iter()
                .map(|(p, t)| (p.as_str(), t.as_str()))
                .collect();
            origin.release(version.tag, &files);
        }
    }

    fn manifest(&self, name: &str, version: &Version) -> String {
        let mut text = format!("[package]\nname = \"{name}\"\n");
        if !version.needs.is_empty() {
            text.push_str("[dependencies]\n");
        }
        for need in &version.needs {
            let scheme = need
                .scheme
                .map_or(String::new(), |s| format!(", scheme = \"{s}\""));
            text.push_str(&format!(
                "{} = {{ git = \"{}\", version = \"{}\"{scheme} }}\n",
                need.dep,
                self.libs[need.dep].url(),
                need.range
            ));
        }
        text
    }

    pub fn add(&self, name: &str, range: &str) -> Run {
        self.project
            .jacket(&["dep", "add", &format!("{}@{range}", self.libs[name].url())])
    }

    pub fn main(&self, text: &str) {
        std::fs::write(self.project.dir.join("main.jai"), text).unwrap();
    }

    /// `jacket check`, with the compiler of this build.
    pub fn check(&self) -> Run {
        let jacket = Path::new(env!("CARGO_BIN_EXE_jacket"));
        let bin = jacket.parent().unwrap();
        assert!(
            bin.join("jaic").exists(),
            "build the compiler first: cargo build -p jaic-cli"
        );
        let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        self.project.jacket_env(
            &["check", "main.jai"],
            &[("JAIC_STDLIB", stdlib.to_str().unwrap())],
        )
    }

    /// The checked-out copies of `name` anywhere under the project.
    pub fn copies(&self, name: &str) -> Vec<String> {
        fn walk(dir: &Path, root: &Path, name: &str, out: &mut Vec<String>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if entry.file_name() == ".git" || !path.is_dir() {
                    continue;
                }
                if path.file_name().is_some_and(|n| n == name) && path.join("module.jai").is_file()
                {
                    out.push(path.strip_prefix(root).unwrap().display().to_string());
                }
                walk(&path, root, name, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.project.dir, &self.project.dir, name, &mut out);
        out.sort();
        out
    }
}

fn qlib() -> Vec<Version> {
    ["v1.0.0", "v1.2.0", "v1.4.0", "v2.0.0"]
        .map(version)
        .to_vec()
}

fn first_error(run: &Run) -> String {
    run.err
        .lines()
        .find(|l| l.contains("error"))
        .unwrap_or("")
        .trim()
        .to_string()
}

/// What a world came to: the first failing step, or the compile's verdict, and where the
/// copies of `qlib` ended up.
#[derive(Debug, PartialEq, Eq)]
pub struct Observed {
    pub result: String,
    pub copies: Vec<String>,
}

/// Builds the world once with dedupe off and once with it on, adds `roots`, compiles `main`, and
/// prints both observations.
fn observe(
    shape: &str,
    build: impl Fn(&mut World),
    roots: &[(&str, &str)],
    main: &str,
) -> (Observed, Observed) {
    let mut seen = Vec::new();
    for dedupe in [false, true] {
        let mut world = World::new(dedupe);
        build(&mut world);
        world.main(main);
        let mut result = String::from("ok");
        for (name, range) in roots {
            let run = world.add(name, range);
            if run.code != 0 {
                result = format!("add {name}: {}", first_error(&run));
                break;
            }
        }
        if result == "ok" {
            let run = world.check();
            if run.code != 0 {
                result = format!("check: {}", first_error(&run));
            }
        }
        seen.push(Observed {
            result,
            copies: world.copies("qlib"),
        });
    }
    let on = seen.pop().unwrap();
    let off = seen.pop().unwrap();
    eprintln!("STRESS {shape}\n  off: {off:?}\n  on:  {on:?}");
    (off, on)
}

const HANDLES: &str = "#import \"Basic\";\n#import \"alpha\";\n#import \"beta\";\nmain :: () {\n    h := alpha_handle();\n    print(\"%\\n\", beta_take(h));\n}\n";

const PLAIN: &str = "#import \"Basic\";\n#import \"alpha\";\nmain :: () {\n    print(\"%\\n\", alpha_via_qlib());\n}\n";

fn diamond(world: &mut World) {
    world.package("qlib", &qlib());
    world.package(
        "alpha",
        &[version("v1.0.0").needs(&[need("qlib", "^1.0", "v1.2.0")])],
    );
    world.package(
        "beta",
        &[version("v1.0.0").needs(&[need("qlib", "^1.1", "v1.4.0")])],
    );
}

#[test]
fn a_diamond_with_a_type_in_a_public_api_compiles_only_when_shared() {
    let (off, on) = observe(
        "diamond",
        diamond,
        &[("alpha", "^1"), ("beta", "^1")],
        HANDLES,
    );
    assert!(off.result.starts_with("check:"), "{off:?}");
    assert_eq!(off.copies.len(), 2, "{off:?}");
    assert_eq!(on.result, "ok", "{on:?}");
    assert_eq!(on.copies, ["modules/qlib"]);
}

#[test]
fn the_shared_copy_is_the_newest_version_every_requirer_admits() {
    let mut world = World::new(true);
    diamond(&mut world);
    world.main(HANDLES);
    for name in ["alpha", "beta"] {
        let run = world.add(name, "^1");
        assert_eq!(run.code, 0, "{name}: {}{}", run.out, run.err);
    }
    let toml = world.project.toml();
    assert!(
        toml.contains("qlib = {")
            && toml.contains("tag = \"v1.4.0\"")
            && toml.contains("shared = true"),
        "{toml}"
    );
    let list = world.project.jacket(&["dep", "list", "--offline"]);
    assert!(list.out.contains("(shared)"), "{}", list.out);
    let check = world.project.jacket(&["dep", "check", "--offline"]);
    assert_eq!(check.code, 0, "{}{}", check.out, check.err);
    assert_eq!(
        git(
            &world.project.dir.join("modules/qlib"),
            &["describe", "--tags"]
        ),
        "v1.4.0"
    );
    // The nested copies the shared one replaces are empty folders, not checkouts.
    assert!(
        !world
            .project
            .dir
            .join("modules/alpha/modules/qlib/module.jai")
            .exists()
    );
}

#[test]
fn removing_the_last_package_that_needs_a_shared_copy_removes_it() {
    let mut world = World::new(true);
    diamond(&mut world);
    world.main(HANDLES);
    for name in ["alpha", "beta"] {
        assert_eq!(world.add(name, "^1").code, 0);
    }
    let run = world.project.jacket(&["dep", "remove", "alpha"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        world.copies("qlib"),
        ["modules/qlib"],
        "beta still needs it"
    );
    let run = world.project.jacket(&["dep", "remove", "beta"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        world.copies("qlib").is_empty(),
        "{:?}",
        world.copies("qlib")
    );
    assert!(
        !world.project.toml().contains("qlib"),
        "{}",
        world.project.toml()
    );
}

#[test]
fn a_shared_entry_cannot_be_removed_or_updated_by_name() {
    let mut world = World::new(true);
    diamond(&mut world);
    assert_eq!(world.add("alpha", "^1").code, 0);
    for command in [["dep", "remove", "qlib"], ["dep", "update", "qlib"]] {
        let run = world.project.jacket(&command);
        assert_ne!(run.code, 0);
        assert!(run.err.contains("is shared"), "{}", run.err);
    }
}

#[test]
fn ranges_nothing_satisfies_name_every_requirer_and_leave_nothing_added() {
    let mut world = World::new(true);
    world.package("qlib", &qlib());
    world.package(
        "alpha",
        &[version("v1.0.0").needs(&[need("qlib", "^1.0", "v1.2.0")])],
    );
    world.package(
        "beta",
        &[version("v1.0.0").needs(&[need("qlib", "^2.0", "v2.0.0")])],
    );
    assert_eq!(world.add("alpha", "^1").code, 0);
    let run = world.add("beta", "^1");
    assert_ne!(run.code, 0);
    assert!(
        run.err.contains("alpha needs ^1.0") && run.err.contains("beta needs ^2.0"),
        "{}",
        run.err
    );
    let toml = world.project.toml();
    assert!(toml.contains("alpha") && !toml.contains("beta"), "{toml}");
    assert!(!world.project.dir.join("modules/beta").exists());
    assert!(
        world.project.toml().contains("qlib"),
        "alpha still needs qlib"
    );
}

#[test]
fn a_chain_shares_every_level() {
    let build = |world: &mut World| {
        world.package("qlib", &qlib());
        world.package(
            "beta",
            &[version("v1.0.0").needs(&[need("qlib", "^1.0", "v1.2.0")])],
        );
        world.package(
            "alpha",
            &[version("v1.0.0")
                .needs(&[need("beta", "^1.0", "v1.0.0")])
                .body("alpha_via_qlib :: () -> s64 { return beta_via_qlib(); }\n")],
        );
    };
    let (off, on) = observe("chain", build, &[("alpha", "^1")], PLAIN);
    assert_eq!(off.result, "ok", "{off:?}");
    assert_eq!(off.copies, ["modules/alpha/modules/beta/modules/qlib"]);
    assert_eq!(on.result, "ok", "{on:?}");
    assert_eq!(on.copies, ["modules/qlib"]);
}

#[test]
fn a_cycle_of_packages_settles() {
    let build = |world: &mut World| {
        world.package("qlib", &qlib());
        world.package("alpha", &[version("v1.0.0")]);
        world.package(
            "beta",
            &[version("v1.0.0").needs(&[need("alpha", "^1.0", "v1.0.0")])],
        );
        // alpha 1.1 needs beta, which needs alpha: the project's alpha satisfies it.
        world.package(
            "alpha",
            &[version("v1.1.0").needs(&[
                need("beta", "^1.0", "v1.0.0"),
                need("qlib", "^1.0", "v1.2.0"),
            ])],
        );
    };
    let (off, on) = observe("cycle", build, &[("alpha", "^1.1")], PLAIN);
    assert_eq!(on.result, "ok", "{on:?}");
    assert_eq!(on.copies, ["modules/qlib"]);
    assert_eq!(off.result, "ok", "{off:?}");
}

#[test]
fn files_in_a_subfolder_skip_the_packages_own_modules_folder() {
    let build = |world: &mut World| {
        world.package("qlib", &qlib());
        world.package(
            "alpha",
            &[version("v1.0.0")
                .needs(&[need("qlib", "^1.0", "v1.2.0")])
                .body("#load \"src/deep.jai\";\nalpha_deep :: () -> s64 { return deep_id(); }\n")
                .file(
                    "src/deep.jai",
                    "#import \"qlib\";\ndeep_id :: () -> s64 { return qlib_id(); }\n",
                )],
        );
    };
    let main = "#import \"Basic\";\n#import \"alpha\";\nmain :: () {\n    print(\"%\\n\", alpha_deep());\n}\n";
    let (off, on) = observe("subfolder", build, &[("alpha", "^1")], main);
    assert!(off.result.contains("module `qlib` not found"), "{off:?}");
    assert_eq!(on.result, "ok", "{on:?}");
}

#[test]
fn each_requirer_keeps_its_own_module_parameters() {
    let build = |world: &mut World| {
        let mut shared = qlib();
        shared[2] = version("v1.4.0").head("#module_parameters(N := 1);");
        world.package("qlib", &shared);
        world.package(
            "alpha",
            &[version("v1.0.0")
                .needs(&[need("qlib", "^1.1", "v1.4.0")])
                .arguments("(N = 2)")],
        );
        world.package(
            "beta",
            &[version("v1.0.0")
                .needs(&[need("qlib", "^1.1", "v1.4.0")])
                .arguments("(N = 3)")],
        );
    };
    let (off, on) = observe(
        "module parameters",
        build,
        &[("alpha", "^1"), ("beta", "^1")],
        HANDLES,
    );
    // Different parameters make different instances, and so different types, shared or not.
    assert!(off.result.starts_with("check:"), "{off:?}");
    assert_eq!(off.result, on.result, "{off:?} {on:?}");
    assert_eq!(on.copies, ["modules/qlib"]);
}

#[test]
fn requirers_that_read_tags_differently_conflict() {
    let mut world = World::new(true);
    world.package("qlib", &qlib());
    world.package(
        "alpha",
        &[version("v1.0.0").needs(&[need("qlib", "^1.0", "v1.2.0")])],
    );
    let mut calendar = need("qlib", ">=1", "v1.2.0");
    calendar.scheme = Some("calendar");
    world.package("beta", &[version("v1.0.0").needs(&[calendar])]);
    assert_eq!(world.add("alpha", "^1").code, 0);
    let run = world.add("beta", "^1");
    assert_ne!(run.code, 0);
    assert!(run.err.contains("read its tags differently"), "{}", run.err);
}

#[test]
fn a_vendored_copy_cannot_be_shared_and_is_reported() {
    let mut world = World::new(true);
    world.package("qlib", &qlib());
    let mut vendored = need("qlib", "^1.0", "v1.2.0");
    vendored.vendored = true;
    world.package(
        "alpha",
        &[version("v1.0.0").needs(&[vendored]).file(
            "modules/qlib/module.jai",
            "qlib_id :: () -> s64 { return 1; }\n",
        )],
    );
    let run = world.add("alpha", "^1");
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        world.copies("qlib"),
        ["modules/alpha/modules/qlib", "modules/qlib"]
    );
    assert!(
        run.err.contains("vendors its own copy of qlib"),
        "{}",
        run.err
    );
}

#[test]
fn a_moved_shared_tag_fails_check() {
    let mut world = World::new(true);
    diamond(&mut world);
    for name in ["alpha", "beta"] {
        assert_eq!(world.add(name, "^1").code, 0);
    }
    let qlib = &world.libs["qlib"];
    qlib.write(&[("module.jai", "qlib_id :: () -> s64 { return 0; }\n")]);
    qlib.commit("rewrite");
    qlib.tag("v1.4.0");
    let check = world.project.jacket(&["dep", "check"]);
    assert_ne!(check.code, 0);
    assert!(
        check.err.contains("v1.4.0") && check.err.contains("moved"),
        "{}",
        check.err
    );
}

#[test]
fn a_deep_chain_with_a_diamond_in_the_middle_shares_each_name_once() {
    let build = |world: &mut World| {
        world.package("qlib", &qlib());
        world.package(
            "gamma",
            &[version("v1.0.0").needs(&[need("qlib", "^1.0", "v1.0.0")])],
        );
        world.package(
            "beta",
            &[version("v1.0.0").needs(&[
                need("gamma", "^1.0", "v1.0.0"),
                need("qlib", "^1.1", "v1.2.0"),
            ])],
        );
        world.package(
            "alpha",
            &[version("v1.0.0")
                .needs(&[
                    need("beta", "^1.0", "v1.0.0"),
                    need("gamma", "^1.0", "v1.0.0"),
                ])
                .body(
                    "alpha_via_qlib :: () -> s64 { return beta_via_qlib() + gamma_via_qlib(); }\n",
                )],
        );
    };
    let (_, on) = observe("deep diamond", build, &[("alpha", "^1")], PLAIN);
    assert_eq!(on.result, "ok", "{on:?}");
    assert_eq!(on.copies, ["modules/qlib"]);
}

#[test]
fn update_moves_every_shared_copy_to_the_newest_version_its_requirers_admit() {
    let mut world = World::new(true);
    diamond(&mut world);
    for name in ["alpha", "beta"] {
        assert_eq!(world.add(name, "^1").code, 0);
    }
    let qlib = &world.libs["qlib"];
    qlib.release("v1.5.0", &[]);
    qlib.release("v2.1.0", &[]);
    let run = world.project.jacket(&["dep", "update"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        world.project.toml().contains("tag = \"v1.5.0\""),
        "{}",
        world.project.toml()
    );
    let check = world.project.jacket(&["dep", "check", "--offline"]);
    assert_eq!(check.code, 0, "{}{}", check.out, check.err);
}

#[test]
fn a_fresh_clone_syncs_the_shared_copy_and_leaves_the_nested_ones_empty() {
    let mut world = World::new(true);
    diamond(&mut world);
    world.main(HANDLES);
    for name in ["alpha", "beta"] {
        assert_eq!(world.add(name, "^1").code, 0);
    }
    let project = &world.project;
    git(&project.dir, &["add", "-A"]);
    git(&project.dir, &["commit", "-q", "-m", "deps"]);
    let clone = project.root.join("clone");
    git(
        &project.root,
        &[
            "clone",
            "-q",
            project.dir.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let run = dep_support::run_in(&clone, &project.root, &["dep", "sync"], &[]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let copies = |dir: &Path| {
        ["alpha", "beta"]
            .iter()
            .filter(|n| {
                dir.join(format!("modules/{n}/modules/qlib/module.jai"))
                    .exists()
            })
            .count()
            + usize::from(dir.join("modules/qlib/module.jai").exists())
    };
    assert_eq!(copies(&clone), 1);
    let check = dep_support::run_in(&clone, &project.root, &["dep", "check", "--offline"], &[]);
    assert_eq!(check.code, 0, "{}{}", check.out, check.err);
}

#[test]
fn a_version_the_project_pinned_itself_must_satisfy_every_requirer() {
    let mut world = World::new(true);
    diamond(&mut world);
    let url = world.libs["qlib"].url();
    let run = world
        .project
        .jacket(&["dep", "add", &format!("{url}@v1.0.0")]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let run = world.add("beta", "^1");
    assert_ne!(run.code, 0);
    assert!(
        run.err.contains("qlib is pinned at v1.0.0") && run.err.contains("beta needs ^1.1"),
        "{}",
        run.err
    );
    assert!(!world.project.toml().contains("beta"));
}

/// What git has recorded for the project: the gitlinks and `.gitmodules`.
fn recorded(world: &World) -> (String, String) {
    (
        git(&world.project.dir, &["ls-files", "-s"]),
        std::fs::read_to_string(world.project.dir.join(".gitmodules")).unwrap_or_default(),
    )
}

#[test]
fn an_update_that_fails_while_sharing_puts_back_the_shared_copies_too() {
    let mut world = World::new(true);
    diamond(&mut world);
    world.package("zlib", &[version("v2.0.0")]);
    for name in ["alpha", "beta"] {
        assert_eq!(world.add(name, "^1").code, 0);
    }
    // qlib moves to its newest version first, then alpha's new release asks for a zlib no
    // version of which fits.
    world.libs["qlib"].release("v1.5.0", &[]);
    world.package(
        "alpha",
        &[version("v1.1.0").needs(&[
            need("qlib", "^1.0", "v1.2.0"),
            need("zlib", "^1.0", "v2.0.0"),
        ])],
    );
    let (toml, state) = (world.project.toml(), recorded(&world));
    let run = world.project.jacket(&["dep", "update"]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(!run.out.contains("updated"), "{}", run.out);
    assert_eq!(world.project.toml(), toml);
    assert_eq!(recorded(&world), state, "{}", run.err);
    let check = world.project.jacket(&["dep", "check", "--offline"]);
    assert_eq!(check.code, 0, "{}{}", check.out, check.err);
}

#[cfg(unix)]
#[test]
fn a_shared_copy_that_cannot_be_cloned_leaves_the_add_undone() {
    let mut world = World::new(true);
    diamond(&mut world);
    let path = dep_support::git_shim(
        world.project.root.path(),
        "case \"$*\" in *submodule*add*qlib*) echo 'fatal: injected' >&2; exit 128;; esac",
    );
    let before = (world.project.toml(), recorded(&world));
    let spec = format!("{}@^1", world.libs["alpha"].url());
    let run = world
        .project
        .jacket_env(&["dep", "add", &spec], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(!run.out.contains("added"), "{}", run.out);
    assert_eq!(
        (world.project.toml(), recorded(&world)),
        before,
        "{}",
        run.err
    );
    assert!(!world.project.dir.join("modules/alpha").exists());
}

#[cfg(unix)]
#[test]
fn a_shared_copy_that_cannot_be_recorded_is_not_left_behind() {
    let mut world = World::new(true);
    diamond(&mut world);
    // Once qlib is cloned, jacket.toml is read-only until a submodule is removed, so the save
    // that records the copy fails and the undo can still restore the file.
    let path = dep_support::git_shim(
        world.project.root.path(),
        "FILE=\"$SHIM_DIR/../project/jacket.toml\"\n\
         case \"$*\" in *deinit*) chmod u+w \"$FILE\";; esac\n\
         case \"$*\" in *submodule*add*qlib*) \"$REAL\" \"$@\"; rc=$?; chmod a-w \"$FILE\"; exit $rc;; esac",
    );
    let before = (world.project.toml(), recorded(&world));
    let spec = format!("{}@^1", world.libs["alpha"].url());
    let run = world
        .project
        .jacket_env(&["dep", "add", &spec], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        (world.project.toml(), recorded(&world)),
        before,
        "{}",
        run.err
    );
    assert!(!world.project.dir.join("modules/qlib").exists());
}

#[cfg(unix)]
#[test]
fn a_nested_checkout_that_fails_leaves_an_add_and_an_update_undone() {
    let mut world = World::new(false);
    diamond(&mut world);
    let path = dep_support::git_shim(
        world.project.root.path(),
        "case \"$(pwd -P)\" in */project/modules/*) case \"$*\" in *submodule*update*) echo 'fatal: injected' >&2; exit 128;; esac;; esac",
    );
    let before = (world.project.toml(), recorded(&world));
    let spec = format!("{}@^1", world.libs["alpha"].url());
    let run = world
        .project
        .jacket_env(&["dep", "add", &spec], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("injected"), "{}", run.err);
    assert!(!run.out.contains("added"), "{}", run.out);
    assert_eq!(
        (world.project.toml(), recorded(&world)),
        before,
        "{}",
        run.err
    );
    assert!(!world.project.dir.join("modules/alpha").exists());
    // The same for an update that has to check out what its new release nests.
    assert_eq!(world.add("alpha", "^1").code, 0);
    world.package(
        "alpha",
        &[version("v1.1.0").needs(&[need("qlib", "^1.0", "v1.2.0")])],
    );
    let before = (world.project.toml(), recorded(&world));
    let run = world
        .project
        .jacket_env(&["dep", "update"], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("injected"), "{}", run.err);
    assert!(!run.out.contains("updated"), "{}", run.out);
    assert_eq!(
        (world.project.toml(), recorded(&world)),
        before,
        "{}",
        run.err
    );
}

/// A world where `alpha` v1.0.0 shares `qlib` from one repository and v1.1.0 asks for the same
/// name from another.
fn moved_repository() -> World {
    let mut world = World::new(true);
    world.package("qlib", &qlib());
    world.package("qlib2", &qlib());
    world.package(
        "alpha",
        &[version("v1.0.0").needs(&[need("qlib", "^1.0", "v1.2.0")])],
    );
    assert_eq!(world.add("alpha", "^1").code, 0);
    let manifest = format!(
        "[package]\nname = \"alpha\"\n[dependencies]\nqlib = {{ git = \"{}\", version = \"^1.0\" }}\n",
        world.libs["qlib2"].url()
    );
    world.libs["alpha"].release("v1.1.0", &[("jacket.toml", manifest.as_str())]);
    world
}

#[cfg(unix)]
#[test]
fn a_shared_copy_replaced_by_another_repository_comes_back_when_the_new_one_fails() {
    let world = moved_repository();
    let path = dep_support::git_shim(
        world.project.root.path(),
        "case \"$*\" in *submodule*add*qlib2*) echo 'fatal: injected' >&2; exit 128;; esac",
    );
    let before = (world.project.toml(), recorded(&world));
    let run = world
        .project
        .jacket_env(&["dep", "update"], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("injected"), "{}", run.err);
    assert_eq!(
        (world.project.toml(), recorded(&world)),
        before,
        "{}",
        run.err
    );
    let check = world.project.jacket(&["dep", "check", "--offline"]);
    assert_eq!(check.code, 0, "{}{}", check.out, check.err);
}

#[cfg(unix)]
#[test]
fn a_shared_copy_from_another_repository_is_replaced_by_the_old_one_on_a_later_failure() {
    let world = moved_repository();
    // The new copy is cloned, then jacket.toml turns read-only until a submodule is removed.
    let path = dep_support::git_shim(
        world.project.root.path(),
        "FILE=\"$SHIM_DIR/../project/jacket.toml\"\n\
         case \"$*\" in *deinit*) chmod u+w \"$FILE\";; esac\n\
         case \"$*\" in *submodule*add*qlib2*) \"$REAL\" \"$@\"; rc=$?; chmod a-w \"$FILE\"; exit $rc;; esac",
    );
    let before = (world.project.toml(), recorded(&world));
    let run = world
        .project
        .jacket_env(&["dep", "update"], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        (world.project.toml(), recorded(&world)),
        before,
        "{}",
        run.err
    );
}

#[test]
fn a_shared_copy_needed_by_an_unchecked_out_package_is_not_removed() {
    let mut world = World::new(true);
    diamond(&mut world);
    for name in ["alpha", "beta"] {
        assert_eq!(world.add(name, "^1").code, 0);
    }
    git(&world.project.dir, &["add", "-A"]);
    git(&world.project.dir, &["commit", "-q", "-m", "deps"]);
    // A fresh clone has no submodule checked out, so nothing can be read about what they need.
    let clone = world.project.root.join("clone");
    git(
        &world.project.root,
        &[
            "clone",
            "-q",
            world.project.dir.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    let before = std::fs::read_to_string(clone.join("jacket.toml")).unwrap();
    let run = dep_support::run_in(&clone, world.project.root.path(), &["dep", "update"], &[]);
    let after = std::fs::read_to_string(clone.join("jacket.toml")).unwrap();
    assert_eq!(after, before, "{}{}", run.out, run.err);
    assert!(!run.out.contains("removed shared"), "{}", run.out);
}

#[cfg(unix)]
#[test]
fn an_undo_that_cannot_read_the_submodules_says_so_and_changes_nothing_more() {
    let mut world = World::new(false);
    diamond(&mut world);
    // The nested checkout fails, and afterwards git can no longer list the index.
    let path = dep_support::git_shim(
        world.project.root.path(),
        "case \"$(pwd -P)\" in */project/modules/*) case \"$*\" in *submodule*update*) touch \"$SHIM_DIR/flag\"; echo 'fatal: injected nested' >&2; exit 128;; esac;; esac\n\
         case \"$*\" in *ls-files*) if [ -f \"$SHIM_DIR/flag\" ]; then echo 'fatal: injected listing' >&2; exit 128; fi;; esac",
    );
    let spec = format!("{}@^1", world.libs["alpha"].url());
    let run = world
        .project
        .jacket_env(&["dep", "add", &spec], &[("PATH", &path)]);
    assert_ne!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("injected nested"), "{}", run.err);
    assert!(
        run.err.contains("could not read the submodules to undo"),
        "{}",
        run.err
    );
    // Nothing was removed on a guess.
    assert!(world.project.dir.join("modules/alpha").exists());
}
