//! The release-age policy against a fake forge on localhost. Git remotes are local bare
//! repositories that `url.<path>.insteadOf` presents as GitHub, GitLab and Codeberg.
use crate::dep_support;
use dep_support::forge::Forge;
use dep_support::{Origin, Project, Run};

const HOUR: i64 = 3600;
const DAY: i64 = 24 * HOUR;

/// A project whose `[resolver]` asks for three days, and an origin standing in for
/// `github.com/someone/lib` with `tags` (name, age in seconds).
struct World {
    project: Project,
    origin: Origin,
    forge: Forge,
}

impl World {
    fn new(resolver: &str, tags: &[(&str, i64)]) -> Self {
        let project = Project::new(&format!("[resolver]\n{resolver}"));
        let origin = Origin::new(&project.root, "lib", &[("module.jai", "x :: 0;\n")]);
        let forge = Forge::start();
        for (tag, age) in tags {
            origin.release(tag, &[("module.jai", &format!("x :: {};\n", tag.len()))]);
            forge.release("someone/lib", tag, *age);
        }
        project.rewrite("https://github.com/someone/lib", &origin.url());
        Self {
            project,
            origin,
            forge,
        }
    }

    fn run(&self, args: &[&str]) -> Run {
        self.project
            .jacket_env(args, &[("JACKET_FORGE_API", &self.forge.base)])
    }
}

#[test]
fn a_named_version_that_is_too_new_is_refused_with_the_override_in_the_note() {
    let world = World::new(
        "min_release_age = \"3d\"\n",
        &[("v1.2.0", 10 * DAY), ("v1.3.0", 20 * HOUR)],
    );
    let run = world.run(&["dep", "add", "someone/lib@v1.3.0"]);
    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err
            .contains("error: lib v1.3.0 was published 20 hours ago; min_release_age is 3d"),
        "{}",
        run.err
    );
    assert!(
        run.err
            .contains("note: to accept it, add `min_release_age = \"0\"` to its entry:")
            && run.err.contains(
                "lib = { git = \"someone/lib\", tag = \"v1.3.0\", min_release_age = \"0\" }"
            ),
        "{}",
        run.err
    );
    assert!(
        !world.project.dir.join("modules/lib").exists(),
        "nothing was fetched"
    );
    assert!(!world.project.toml().contains("[dependencies]"));
}

#[test]
fn a_range_skips_versions_that_are_too_new_and_a_waiver_accepts_one() {
    let world = World::new(
        "min_release_age = \"3d\"\n",
        &[("v1.2.0", 10 * DAY), ("v1.3.0", 20 * HOUR)],
    );
    let run = world.run(&["dep", "add", "someone/lib@^1.0"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.out.contains("v1.2.0"), "{}", run.out);
    assert!(
        run.err.contains("note: lib v1.3.0 is newer but not used: it was published 20 hours ago; min_release_age is 3d"),
        "{}",
        run.err
    );
    // Waiving the policy for this entry lets update take the newer one.
    let toml = world.project.toml().replace(
        "version = \"^1.0\"",
        "version = \"^1.0\", min_release_age = \"0\"",
    );
    std::fs::write(world.project.dir.join("jacket.toml"), toml).unwrap();
    let run = world.run(&["dep", "update", "lib"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(run.out.contains("v1.3.0"), "{}", run.out);
}

#[test]
fn nothing_old_enough_is_an_error_naming_the_newest() {
    let world = World::new("min_release_age = \"3d\"\n", &[("v1.0.0", 5 * HOUR)]);
    let run = world.run(&["dep", "add", "someone/lib"]);
    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("lib v1.0.0 was published 5 hours ago"),
        "{}",
        run.err
    );
    assert!(run.err.contains("min_release_age = \"0\""), "{}", run.err);
}

#[test]
fn an_age_that_cannot_be_checked_is_an_error_with_the_same_note() {
    // A tag without a release, a host that keeps no times, and a commit pin all have no age.
    let world = World::new("min_release_age = \"3d\"\n", &[]);
    world.origin.release("v1.0.0", &[]);
    let run = world.run(&["dep", "add", "someone/lib@v1.0.0"]);
    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("lib v1.0.0's age can't be checked")
            && run.err.contains("no release for v1.0.0"),
        "{}",
        run.err
    );
    assert!(run.err.contains("min_release_age = \"0\""), "{}", run.err);

    let plain = Project::new("[resolver]\nmin_release_age = \"3d\"\n");
    let local = Origin::new(&plain.root, "local", &[("module.jai", "")]);
    local.release("v1.0.0", &[]);
    let run = plain.jacket(&["dep", "add", &format!("{}@v1.0.0", local.url())]);
    assert_eq!(run.code, 2);
    assert!(run.err.contains("age can't be checked"), "{}", run.err);

    let rev = world.origin.commit("tip");
    let run = world.run(&["dep", "add", &format!("someone/lib@{}", &rev[..7])]);
    assert_eq!(run.code, 2);
    assert!(
        run.err.contains("age can't be checked") && run.err.contains("rev = "),
        "{}",
        run.err
    );
}

#[test]
fn with_no_policy_nothing_asks_a_forge() {
    let world = World::new("", &[("v1.0.0", HOUR)]);
    let run = world.run(&["dep", "add", "someone/lib@v1.0.0"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(world.forge.log().is_empty());
}

#[test]
fn times_are_cached_and_lists_are_asked_for_with_their_etag() {
    let world = World::new("min_release_age = \"1d\"\n", &[("v1.0.0", 5 * DAY)]);
    assert_eq!(world.run(&["dep", "add", "someone/lib"]).code, 0);
    assert_eq!(world.forge.log().len(), 1);
    // The same tag at the same commit: answered from the cache.
    world.forge.clear();
    assert_eq!(world.run(&["dep", "update", "lib"]).code, 0);
    assert!(world.forge.log().is_empty(), "{:?}", world.forge.log());
    // A new release: the list is fetched again, conditionally, and now differs.
    world
        .origin
        .release("v1.1.0", &[("module.jai", "x :: 11;\n")]);
    world.forge.release("someone/lib", "v1.1.0", 4 * DAY);
    world.forge.clear();
    assert_eq!(world.run(&["dep", "update", "lib"]).code, 0);
    let log = world.forge.log();
    assert_eq!(log.len(), 1);
    assert!(log[0].conditional && log[0].status == 200, "{log:?}");
    // The tag moves to a new commit: the time cached for the old one does not apply, so the
    // list is asked for again, conditionally, and has not changed.
    world.origin.commit("moved");
    world.origin.tag("v1.1.0");
    world.forge.clear();
    let run = world.run(&["dep", "add", "someone/lib@v1.1.0", "--name", "again"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let log = world.forge.log();
    assert!(
        log.len() == 1 && log[0].conditional && log[0].status == 304,
        "{log:?}"
    );
}

#[test]
fn a_rate_limit_is_reported_with_the_token_to_set() {
    let world = World::new("min_release_age = \"1d\"\n", &[("v1.0.0", 5 * DAY)]);
    world.forge.limit(true);
    let run = world.run(&["dep", "add", "someone/lib"]);
    assert_eq!(run.code, 2, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("rate limit") && run.err.contains("GITHUB_TOKEN"),
        "{}",
        run.err
    );
}

#[test]
fn tokens_go_only_to_their_own_forge() {
    let world = World::new("min_release_age = \"1d\"\n", &[("v1.0.0", 5 * DAY)]);
    let envs = [
        ("JACKET_FORGE_API", world.forge.base.as_str()),
        ("GITHUB_TOKEN", "gh-secret"),
        ("GITLAB_TOKEN", "gl-secret"),
    ];
    assert_eq!(
        world
            .project
            .jacket_env(&["dep", "add", "someone/lib"], &envs)
            .code,
        0
    );
    let log = world.forge.log();
    assert_eq!(
        log[0].authorization.as_deref(),
        Some("Bearer gh-secret"),
        "{log:?}"
    );
    assert_eq!(log[0].private_token, None);
    // GitLab and Codeberg repositories.
    world.forge.release("grp/sub/lib2", "v1.0.0", 5 * DAY);
    let second = Origin::new(&world.project.root, "lib2", &[("module.jai", "")]);
    second.release("v1.0.0", &[]);
    world
        .project
        .rewrite("https://gitlab.com/grp/sub/lib2", &second.url());
    world.forge.clear();
    assert_eq!(
        world
            .project
            .jacket_env(&["dep", "add", "gitlab.com/grp/sub/lib2"], &envs)
            .code,
        0
    );
    let log = world.forge.log();
    assert_eq!(
        log[0].private_token.as_deref(),
        Some("gl-secret"),
        "{log:?}"
    );
    assert_eq!(log[0].authorization, None);
    world.forge.release("me/lib3", "v1.0.0", 5 * DAY);
    let third = Origin::new(&world.project.root, "lib3", &[("module.jai", "")]);
    third.release("v1.0.0", &[]);
    world
        .project
        .rewrite("https://codeberg.org/me/lib3", &third.url());
    world.forge.clear();
    assert_eq!(
        world
            .project
            .jacket_env(&["dep", "add", "codeberg.org/me/lib3"], &envs)
            .code,
        0
    );
    let log = world.forge.log();
    assert!(
        log[0].authorization.is_none() && log[0].private_token.is_none(),
        "{log:?}"
    );
}

#[test]
fn update_asks_github_once_for_every_dependency() {
    let world = World::new("min_release_age = \"1d\"\n", &[("v1.0.0", 5 * DAY)]);
    let other = Origin::new(&world.project.root, "other", &[("module.jai", "")]);
    other.release("v1.0.0", &[]);
    world
        .project
        .rewrite("https://github.com/someone/other", &other.url());
    world.forge.release("someone/other", "v1.0.0", 5 * DAY);
    let envs = [
        ("JACKET_FORGE_API", world.forge.base.as_str()),
        ("GITHUB_TOKEN", "t"),
    ];
    assert_eq!(
        world
            .project
            .jacket_env(&["dep", "add", "someone/lib"], &envs)
            .code,
        0
    );
    assert_eq!(
        world
            .project
            .jacket_env(&["dep", "add", "someone/other"], &envs)
            .code,
        0
    );
    other.release("v1.1.0", &[("module.jai", "y :: 1;\n")]);
    world.forge.release("someone/other", "v1.1.0", 3 * DAY);
    world
        .origin
        .release("v1.1.0", &[("module.jai", "y :: 2;\n")]);
    world.forge.release("someone/lib", "v1.1.0", 3 * DAY);
    world.forge.clear();
    // With no cached lists, one GraphQL query answers for both repositories.
    std::fs::remove_dir_all(world.project.root.join("cache")).unwrap();
    let run = world.project.jacket_env(&["dep", "update"], &envs);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    let posts: Vec<_> = world
        .forge
        .log()
        .into_iter()
        .filter(|r| r.method == "POST")
        .collect();
    assert_eq!(posts.len(), 1, "{:?}\n{}", world.forge.log(), run.err);
    assert!(
        world
            .forge
            .log()
            .iter()
            .all(|r| r.path.starts_with("/graphql")),
        "{:?}",
        world.forge.log()
    );
    assert!(run.out.matches("v1.1.0").count() >= 2, "{}", run.out);
}

#[test]
fn jitter_needs_a_seed_and_the_first_add_writes_one() {
    let world = World::new(
        "min_release_age = \"1d\"\nrelease_age_jitter = \"2d\"\n",
        &[("v1.0.0", 30 * DAY)],
    );
    let run = world.run(&["dep", "add", "someone/lib"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("release_age_jitter")
            && run.err.contains("seed")
            && run.err.contains("jacket dep init"),
        "{}",
        run.err
    );
    assert!(
        world.project.toml().contains("seed = \""),
        "{}",
        world.project.toml()
    );
}

#[test]
fn the_same_seed_gives_the_same_wait_and_another_seed_another() {
    // A release just past the floor is accepted or not depending on the seed alone.
    let age = DAY + 12 * HOUR;
    let mut verdicts = Vec::new();
    for seed in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        let verdict = || {
            let world = World::new(
                &format!(
                    "min_release_age = \"1d\"\nrelease_age_jitter = \"2d\"\nseed = \"{seed}\"\n"
                ),
                &[("v1.0.0", age)],
            );
            world.run(&["dep", "add", "someone/lib@v1.0.0"])
        };
        let (first, again) = (verdict(), verdict());
        assert_eq!(first.code, again.code, "seed {seed} is stable");
        verdicts.push(first.code);
    }
    assert!(
        verdicts.contains(&0) && verdicts.contains(&2),
        "seeds differ: {verdicts:?}"
    );
}

#[test]
fn init_lists_adopted_versions_that_break_the_policy() {
    let world = World::new("min_release_age = \"3d\"\n", &[("v1.0.0", 10 * HOUR)]);
    dep_support::git(
        &world.project.dir,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &world.origin.url(),
            "modules/lib",
        ],
    );
    dep_support::git(
        &world.project.dir.join("modules/lib"),
        &["checkout", "-q", "v1.0.0"],
    );
    // The remote is GitHub as far as `.gitmodules` says.
    dep_support::git(
        &world.project.dir,
        &[
            "config",
            "-f",
            ".gitmodules",
            "submodule.modules/lib.url",
            "https://github.com/someone/lib",
        ],
    );
    let run = world.run(&["dep", "init"]);
    assert_eq!(run.code, 0, "{}{}", run.out, run.err);
    assert!(
        run.err.contains("lib v1.0.0 was published 10 hours ago")
            && run.err.contains("min_release_age = \"0\""),
        "{}{}",
        run.out,
        run.err
    );
    assert!(world.project.toml().contains("lib = {"));
}
