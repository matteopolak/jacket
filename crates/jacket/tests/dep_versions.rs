//! Version schemes, prefixes and ranges against local bare repositories with many tags.
mod dep_support;
use dep_support::{Origin, Project, git};

fn tagged(names: &[&str]) -> (Project, Origin) {
    let project = Project::new("");
    let origin = Origin::new(&project.root, "lib", &[("module.jai", "x :: 0;\n")]);
    for (i, name) in names.iter().enumerate() {
        origin.release(name, &[("module.jai", &format!("x :: {};\n", i + 1))]);
    }
    (project, origin)
}

fn head(project: &Project, name: &str) -> String {
    git(
        &project.dir.join("modules").join(name),
        &["describe", "--tags", "--exact-match"],
    )
}

#[test]
fn no_version_pins_the_newest_stable_tag_as_a_caret_range() {
    let (project, origin) = tagged(&["v1.0.0", "v1.2.0", "v1.10.0", "v2.0.0-rc.1"]);
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(
        head(&project, "lib"),
        "v1.10.0",
        "the prerelease is not stable"
    );
    assert!(
        project.toml().contains("version = \"^1.10.0\""),
        "{}",
        project.toml()
    );
}

#[test]
fn a_range_picks_the_newest_tag_it_admits_and_update_follows_it() {
    let (project, origin) = tagged(&["v1.0.0", "v1.2.0", "v2.0.0"]);
    let run = project.jacket(&["dep", "add", &format!("{}@^1.0", origin.url())]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(head(&project, "lib"), "v1.2.0");
    assert!(
        project.toml().contains("version = \"^1.0\""),
        "the range is kept as written"
    );
    origin.release("v1.3.0", &[("module.jai", "x :: 9;\n")]);
    origin.release("v3.0.0", &[("module.jai", "x :: 10;\n")]);
    let list = project.jacket(&["dep", "list"]);
    assert!(
        list.out.contains("v1.3.0"),
        "list shows the newer allowed one: {}",
        list.out
    );
    assert!(!list.out.contains("v3.0.0"), "{}", list.out);
    let run = project.jacket(&["dep", "update"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(head(&project, "lib"), "v1.3.0");
    assert!(
        project
            .jacket(&["dep", "update", "lib"])
            .out
            .contains("up to date")
    );
    assert_eq!(project.jacket(&["dep", "check"]).code, 0);
}

#[test]
fn a_version_in_full_names_its_tag_and_two_part_tags_read_as_three() {
    let (project, origin) = tagged(&["v0.8", "v1.0.0"]);
    let run = project.jacket(&["dep", "add", &format!("{}@0.8.0", origin.url())]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(head(&project, "lib"), "v0.8");
    assert!(
        project.toml().contains("tag = \"v0.8\""),
        "{}",
        project.toml()
    );
}

#[test]
fn prefixes_and_other_schemes_read_their_own_tags() {
    let (project, origin) = tagged(&["alpha-0.0.1", "alpha-0.0.3", "beta-9.9.9"]);
    let run = project.jacket(&["dep", "add", &origin.url(), "--tag-prefix", "alpha-"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(head(&project, "lib"), "alpha-0.0.3");
    assert!(
        project.toml().contains("tag_prefix = \"alpha-\""),
        "{}",
        project.toml()
    );

    let (project, origin) = tagged(&["r9", "r10", "r42", "notes"]);
    let run = project.jacket(&["dep", "add", &origin.url(), "--scheme", "integer"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(head(&project, "lib"), "r42");
    assert!(
        project.toml().contains("scheme = \"integer\"") && project.toml().contains("tag = \"r42\"")
    );

    let (project, origin) = tagged(&["2024.05.01", "2024.5", "2023.12"]);
    let run = project.jacket(&[
        "dep",
        "add",
        &format!("{}@>=2023.1, <2024.1", origin.url()),
        "--scheme",
        "calendar",
    ]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert_eq!(head(&project, "lib"), "2023.12");
    let run = project.jacket(&[
        "dep",
        "add",
        &format!("{}@^2023", origin.url()),
        "--scheme",
        "calendar",
        "--name",
        "x",
    ]);
    assert_eq!(run.code, 2);
    assert!(run.err.contains("not a comparison"), "{}", run.err);
}

#[test]
fn a_range_nothing_satisfies_is_an_error_naming_what_exists() {
    let (project, origin) = tagged(&["v1.0.0", "v1.1.0"]);
    let run = project.jacket(&["dep", "add", &format!("{}@^3", origin.url())]);
    assert_eq!(run.code, 2);
    assert!(
        run.err.contains("no tag") && run.err.contains("^3") && run.err.contains("v1.1.0"),
        "{}",
        run.err
    );
    assert!(!project.dir.join("modules").exists());
}

#[test]
fn tags_that_do_not_parse_are_hidden_until_all_tags_asks() {
    let (project, origin) = tagged(&["v1.0.0", "nightly", "release-candidate"]);
    project.jacket(&["dep", "add", &origin.url()]);
    let plain = project.jacket(&["dep", "list"]);
    assert!(!plain.out.contains("nightly"), "{}", plain.out);
    let all = project.jacket(&["dep", "list", "--all-tags"]);
    assert!(
        all.out.contains("nightly") && all.out.contains("release-candidate"),
        "{}",
        all.out
    );
}

#[test]
fn a_repository_with_no_usable_tags_is_pinned_by_commit_and_says_why() {
    let (project, origin) = tagged(&["nightly"]);
    let run = project.jacket(&["dep", "add", &origin.url()]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(project.toml().contains("rev = \""), "{}", project.toml());
    assert!(
        run.err.contains("1 tag") && run.err.contains("--tag-prefix"),
        "{}",
        run.err
    );
}

#[test]
fn init_adopts_the_highest_version_tag_at_the_commit() {
    let (project, origin) = tagged(&["v1.0.0"]);
    origin.tag("v1.0.1");
    origin.tag("stable");
    git(
        &project.dir,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &origin.url(),
            "modules/lib",
        ],
    );
    git(
        &project.dir.join("modules/lib"),
        &["checkout", "-q", "v1.0.1"],
    );
    let run = project.jacket(&["dep", "init"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        project.toml().contains("tag = \"v1.0.1\""),
        "{}",
        project.toml()
    );
}
