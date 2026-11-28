//! `[package]` in a dependency's own `jacket.toml`: module discovery, names and requirements.
use crate::dep_support;
use dep_support::{Origin, Project};

const MODULE: &[(&str, &str)] = &[("module.jai", "answer :: 42;\n")];

fn project() -> Project {
    Project::new("[project]\nentry = [\"main.jai\"]\n")
}

#[test]
fn the_package_name_names_the_folder_when_no_name_is_given() {
    let project = project();
    let origin = Origin::new(
        &project.root,
        "repo-with-a-long-name",
        &[
            ("module.jai", "answer :: 42;\n"),
            ("jacket.toml", "[package]\nname = \"Short\"\n"),
        ],
    );
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(project.dir.join("modules/Short/module.jai").is_file());
    assert!(!project.dir.join("modules/repo-with-a-long-name").exists());
    assert!(project.toml().contains("Short = {"), "{}", project.toml());
    assert!(run.out.contains("added Short"), "{}", run.out);
    let modules = std::fs::read_to_string(project.dir.join(".gitmodules")).unwrap();
    assert!(modules.contains("path = modules/Short"), "{modules}");
}

#[test]
fn an_explicit_name_is_kept() {
    let project = project();
    let origin = Origin::new(
        &project.root,
        "lib",
        &[
            ("module.jai", "answer :: 42;\n"),
            ("jacket.toml", "[package]\nname = \"Other\"\n"),
        ],
    );
    let run = project.jacket(&["dep", "add", &origin.url(), "--name", "Mine"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(project.dir.join("modules/Mine/module.jai").is_file());
}

#[test]
fn a_module_in_a_subfolder_is_found_and_recorded() {
    let project = project();
    let origin = Origin::new(
        &project.root,
        "wrapper",
        &[("modules/Inner/module.jai", "answer :: 1;\n")],
    );
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        project.toml().contains("module = \"modules/Inner\""),
        "{}",
        project.toml()
    );
}

#[test]
fn several_modules_need_a_choice_and_leave_nothing_behind() {
    let project = project();
    let origin = Origin::new(
        &project.root,
        "many",
        &[
            ("modules/A/module.jai", "a :: 1;\n"),
            ("modules/B/module.jai", "b :: 2;\n"),
        ],
    );
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_ne!(run.code, 0);
    assert!(
        run.err.contains("modules/A") && run.err.contains("--module"),
        "{}",
        run.err
    );
    assert!(!project.dir.join("modules/many").exists());
    assert!(!project.toml().contains("many"));
    let chosen = project.jacket(&["dep", "add", &origin.url(), "--module", "modules/B"]);
    assert_eq!(chosen.code, 0, "{}{}", chosen.out, chosen.err);
    assert!(
        project.toml().contains("module = \"modules/B\""),
        "{}",
        project.toml()
    );
}

#[test]
fn a_package_that_needs_a_newer_jacket_is_refused_and_removed() {
    let project = project();
    let origin = Origin::new(
        &project.root,
        "future",
        &[
            ("module.jai", "answer :: 42;\n"),
            ("jacket.toml", "[package]\nrequires_jacket = \">=99.0.0\"\n"),
        ],
    );
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_ne!(run.code, 0);
    assert!(run.err.contains("requires jacket >=99.0.0"), "{}", run.err);
    assert!(!project.dir.join("modules/future").exists());
    assert!(!project.toml().contains("future"));
}

#[test]
fn update_keeps_the_old_commit_when_the_new_one_needs_a_newer_jacket() {
    let project = project();
    let origin = Origin::new(&project.root, "lib", MODULE);
    let first = origin.release("v1.0.0", &[("module.jai", "answer :: 1;\n")]);
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    origin.release(
        "v2.0.0",
        &[("jacket.toml", "[package]\nrequires_jacket = \">=99.0.0\"\n")],
    );
    let run = project.jacket(&["dep", "update", "lib", "--to", "v2.0.0"]);
    assert_ne!(run.code, 0);
    assert!(run.err.contains("requires jacket"), "{}", run.err);
    assert_eq!(
        dep_support::git(&project.dir.join("modules/lib"), &["rev-parse", "HEAD"]),
        first
    );
    assert!(!project.toml().contains("2.0.0"), "{}", project.toml());
}

#[test]
fn a_package_name_that_differs_from_its_module_folder_is_refused() {
    let project = project();
    let origin = Origin::new(
        &project.root,
        "odd",
        &[
            ("modules/Real/module.jai", "a :: 1;\n"),
            ("jacket.toml", "[package]\nname = \"Fake\"\n"),
        ],
    );
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_ne!(run.code, 0);
    assert!(
        run.err.contains("Fake") && run.err.contains("Real"),
        "{}",
        run.err
    );
    assert!(!project.dir.join("modules/odd").exists());
}

#[test]
fn platforms_and_version_mismatches_warn_without_failing() {
    let project = project();
    let origin = Origin::new(&project.root, "warn", MODULE);
    origin.release(
        "v1.0.0",
        &[(
            "jacket.toml",
            "[package]\nversion = \"1.2.0\"\nplatforms = [\"plan9\"]\n",
        )],
    );
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.err.contains("plan9"), "{}", run.err);
    assert!(
        run.err.contains("[package] version is 1.2.0"),
        "{}",
        run.err
    );
}

#[test]
fn an_existing_folder_is_not_replaced_without_a_name() {
    let project = project();
    std::fs::create_dir_all(project.dir.join("modules/lib")).unwrap();
    std::fs::write(project.dir.join("modules/lib/module.jai"), "mine :: 1;\n").unwrap();
    let origin = Origin::new(&project.root, "lib", MODULE);
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_ne!(run.code, 0);
    assert!(run.err.contains("--name lib"), "{}", run.err);
    assert_eq!(
        std::fs::read_to_string(project.dir.join("modules/lib/module.jai")).unwrap(),
        "mine :: 1;\n"
    );
}

#[test]
fn a_new_library_names_itself_in_its_package_table() {
    let project = project();
    let run = dep_support::run_in(
        &project.dir,
        &project.root,
        &["new", "--lib", "Shapes"],
        &[],
    );
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let toml = std::fs::read_to_string(project.dir.join("Shapes/jacket.toml")).unwrap();
    assert!(
        toml.contains("[package]\nname = \"Shapes\"\nversion = \"0.1.0\""),
        "{toml}"
    );
}

#[test]
fn list_reports_what_a_dependency_needs_against_its_nested_pin() {
    let project = project();
    let inner = Origin::new(&project.root, "inner", MODULE);
    inner.release("v1.0.0", &[]);
    inner.release("v2.0.0", &[]);
    let outer = Origin::new(&project.root, "outer", MODULE);
    dep_support::git(
        &outer.work,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &inner.url(),
            "modules/inner",
        ],
    );
    dep_support::git(
        &outer.work.join("modules/inner"),
        &["checkout", "-q", "v1.0.0"],
    );
    outer.release(
        "v1.0.0",
        &[(
            "jacket.toml",
            &format!(
                "[dependencies]\ninner = {{ git = \"{}\", version = \"^2.0\" }}\n",
                inner.url()
            ),
        )],
    );
    let run = project.jacket(&["dep", "add", &outer.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let list = project.jacket(&["dep", "list", "--offline"]);
    assert_eq!(list.code, 0, "{}{}", list.out, list.err);
    assert!(
        list.out
            .contains("outer needs inner ^2.0: pinned at v1.0.0, which does not satisfy it"),
        "{}",
        list.out
    );
}
