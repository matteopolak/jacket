//! `[resolver] dedupe` (experimental): one copy of each module for the whole program.
//!
//! What each checked-out dependency asks for in its own `[dependencies]` is solved together with
//! the project's entries. A name no entry of the project covers becomes an entry with
//! `shared = true`, pinned to the newest version every requirer admits, as a submodule under
//! the project's `modules/`. The nested copies it replaces are never checked out, so a
//! library's `#import` reaches the shared one. The compiler is unchanged.
use super::commands::{Gate, Pin, accept_newest, path_of, resolved_tag, say, short};
use super::git::RemoteTag;
use super::package;
use super::resolve::Reading;
use super::version::Tagged;
use super::{Ctx, DepError, Project, Result, manifest, spec};
use jacket_config::Dependency;
use std::collections::BTreeMap;

/// One package's request for a name.
#[derive(Clone, Debug)]
pub struct Need {
    pub by: String,
    pub entry: Dependency,
}

impl Need {
    /// What the request says, for messages.
    fn says(&self) -> String {
        let asked = match (&self.entry.version, &self.entry.tag, &self.entry.rev) {
            (Some(range), ..) => range.clone(),
            (_, Some(tag), _) => format!("tag {tag}"),
            (_, _, Some(rev)) => format!("rev {rev}"),
            _ => "any version".into(),
        };
        format!("{} needs {asked}", self.by)
    }
}

/// Whether the version at `tag`/`commit` is one `need` admits.
fn admits(reading: &Reading, need: &Need, tag: Option<&str>, commit: &str) -> bool {
    let entry = &need.entry;
    if let Some(range) = &entry.version {
        let version = tag.and_then(|t| reading.parse(t));
        return reading
            .range(range)
            .ok()
            .zip(version)
            .is_some_and(|(range, v)| range.matches(&v));
    }
    if let Some(wanted) = &entry.tag {
        return tag == Some(wanted.as_str());
    }
    if let Some(rev) = &entry.rev {
        return commit.starts_with(rev.as_str()) || rev.starts_with(short(commit));
    }
    true
}

/// How every requirer reads tags: they must agree on the repository and the scheme.
fn common(name: &str, needs: &[Need]) -> Result<Dependency> {
    let first = &needs[0];
    for other in &needs[1..] {
        if !spec::same_repo(&first.entry.git, &other.entry.git) {
            return Err(DepError::new(format!(
                "{name}: {} and {} name different repositories",
                first.by, other.by
            ))
            .note(format!(
                "{}: {}\n{}: {}",
                first.by, first.entry.git, other.by, other.entry.git
            )));
        }
        if first.entry.scheme != other.entry.scheme
            || first.entry.tag_prefix != other.entry.tag_prefix
        {
            return Err(DepError::new(format!(
                "{name}: {} and {} read its tags differently",
                first.by, other.by
            ))
            .note("`scheme` and `tag_prefix` must agree"));
        }
    }
    Ok(Dependency {
        git: first.entry.git.clone(),
        scheme: first.entry.scheme.clone(),
        tag_prefix: first.entry.tag_prefix.clone(),
        ..Dependency::default()
    })
}

fn conflict(name: &str, url: &str, needs: &[Need], tags: &[Tagged]) -> DepError {
    let mut lines: Vec<String> = needs.iter().map(Need::says).collect();
    let all: Vec<&str> = tags.iter().take(8).map(|t| t.tag.as_str()).collect();
    if !all.is_empty() {
        lines.push(format!("{url} has {}", all.join(", ")));
    }
    DepError::new(format!(
        "{name}: no version of {url} satisfies every package that needs it"
    ))
    .note(lines.join("\n"))
}

/// The versions of the remote `tags` every need admits, newest first.
pub fn candidates(reading: &Reading, needs: &[Need], tags: &[RemoteTag]) -> Vec<Tagged> {
    let ranged = needs.iter().any(|n| n.entry.version.is_some());
    reading
        .versions(tags)
        .into_iter()
        .filter(|t| {
            let named = needs
                .iter()
                .any(|n| n.entry.tag.as_deref() == Some(t.tag.as_str()));
            (!t.version.is_prerelease() || ranged || named)
                && needs
                    .iter()
                    .all(|n| admits(reading, n, Some(&t.tag), &t.commit))
        })
        .collect()
}

/// What each checked-out entry of the project asks for, by name.
fn gather(ctx: &Ctx, project: &Project) -> BTreeMap<String, Vec<Need>> {
    let mut needs: BTreeMap<String, Vec<Need>> = BTreeMap::new();
    for (by, entry) in &project.config.dependencies {
        let path = path_of(by, entry);
        if !ctx.git.initialised(&project.dir, &path) {
            continue;
        }
        let Some(manifest) = package::read(&project.dir.join(&path)) else {
            continue;
        };
        for (name, entry) in manifest.dependencies {
            if &name != by {
                needs.entry(name).or_default().push(Need {
                    by: by.clone(),
                    entry,
                });
            }
        }
    }
    needs
}

/// The project's commit for each submodule path.
fn commits(ctx: &Ctx, project: &Project) -> Result<BTreeMap<String, String>> {
    Ok(ctx
        .git
        .submodules(&project.dir)?
        .into_iter()
        .map(|s| (s.path, s.commit))
        .collect())
}

/// Solves and applies the shared set. With `fresh`, shared entries move to the newest version
/// their requirers admit; otherwise one that still satisfies everyone stays.
pub(super) fn run(ctx: &mut Ctx, project: &mut Project, fresh: bool) -> Result<()> {
    if project.config.resolver.dedupe != Some(true) {
        return Ok(());
    }
    // A choice can bring in a package with requirements of its own, and a cycle of names settles
    // because a name already chosen is only revisited when its constraints grow.
    for _ in 0..64 {
        if !round(ctx, project, fresh)? {
            report_vendored(ctx, project);
            return Ok(());
        }
    }
    Err(DepError::new(
        "the shared dependencies did not settle after 64 rounds",
    ))
}

/// One pass; whether anything changed.
fn round(ctx: &mut Ctx, project: &mut Project, fresh: bool) -> Result<bool> {
    let needs = gather(ctx, project);
    let mut changed = false;
    for (name, asked) in &needs {
        let known = commits(ctx, project)?;
        let current = project.config.dependencies.get(name).cloned();
        let common = common(name, asked)?;
        let reading = Reading::of(&common).map_err(|e| DepError::new(format!("{name}: {e}")))?;
        let url = spec::url(&common.git);
        match current {
            Some(entry) if !entry.shared => {
                // The project's own pin is not moved; every requirer must accept it.
                check_pinned(ctx, project, name, &entry, &reading, asked, &known)?;
            }
            Some(entry)
                if !fresh && still_fits(ctx, project, name, &entry, &reading, asked, &known) => {}
            existing => {
                let tags = ctx.git.remote_tags(&url)?;
                let pin = choose(ctx, project, name, &common, &reading, asked, &tags)?;
                if let Some(entry) = &existing
                    && known.get(&path_of(name, entry)) == Some(&pin.commit)
                {
                    continue;
                }
                install(ctx, project, name, &common, asked, &pin, existing.as_ref())?;
                changed = true;
            }
        }
    }
    // A shared copy no package asks for any more goes with the last one that did.
    let orphans: Vec<String> = project
        .config
        .dependencies
        .iter()
        .filter(|(name, e)| e.shared && !needs.contains_key(*name))
        .map(|(name, _)| name.clone())
        .collect();
    for name in orphans {
        let entry = project.config.dependencies[&name].clone();
        let path = path_of(&name, &entry);
        ctx.git.remove(&project.dir, &path)?;
        let (text, _) = manifest::remove_dependency(&project.text, &name);
        project.save(text)?;
        say(ctx, format!("removed shared {name} (no package needs it)"));
        changed = true;
    }
    Ok(changed)
}

/// The tag a checked-out entry stands on.
fn tag_of(
    ctx: &Ctx,
    project: &Project,
    entry: &Dependency,
    path: &str,
    commit: &str,
) -> Option<String> {
    if let Some(tag) = &entry.tag {
        return Some(tag.clone());
    }
    resolved_tag(ctx.git, project, entry, path, commit)
}

fn check_pinned(
    ctx: &Ctx,
    project: &Project,
    name: &str,
    entry: &Dependency,
    reading: &Reading,
    needs: &[Need],
    known: &BTreeMap<String, String>,
) -> Result<()> {
    let path = path_of(name, entry);
    let commit = known.get(&path).cloned().unwrap_or_default();
    let tag = tag_of(ctx, project, entry, &path, &commit);
    let refused: Vec<String> = needs
        .iter()
        .filter(|n| !admits(reading, n, tag.as_deref(), &commit))
        .map(Need::says)
        .collect();
    if refused.is_empty() {
        return Ok(());
    }
    let pinned = tag.unwrap_or_else(|| format!("rev {}", short(&commit)));
    Err(DepError::new(format!(
        "{name} is pinned at {pinned} in jacket.toml, which not every package accepts"
    ))
    .note(refused.join("\n"))
    .note(format!(
        "change its entry in jacket.toml, or `jacket dep update {name} --to <tag>`"
    )))
}

fn still_fits(
    ctx: &Ctx,
    project: &Project,
    name: &str,
    entry: &Dependency,
    reading: &Reading,
    needs: &[Need],
    known: &BTreeMap<String, String>,
) -> bool {
    let path = path_of(name, entry);
    let Some(commit) = known.get(&path) else {
        return false;
    };
    ctx.git.initialised(&project.dir, &path)
        && spec::same_repo(&entry.git, &needs[0].entry.git)
        && {
            let tag = tag_of(ctx, project, entry, &path, commit);
            needs
                .iter()
                .all(|n| admits(reading, n, tag.as_deref(), commit))
        }
}

/// The commit to share: the newest version every requirer admits and the policy accepts.
fn choose(
    ctx: &mut Ctx,
    project: &Project,
    name: &str,
    common: &Dependency,
    reading: &Reading,
    needs: &[Need],
    tags: &[RemoteTag],
) -> Result<Pin> {
    let url = spec::url(&common.git);
    let gate = Gate::new(project, name, common, &url)?;
    let revs: Vec<&String> = needs.iter().filter_map(|n| n.entry.rev.as_ref()).collect();
    if let Some(rev) = revs.iter().max_by_key(|r| r.len()) {
        let others = needs
            .iter()
            .any(|n| n.entry.version.is_some() || n.entry.tag.is_some());
        let agree = revs.iter().all(|r| rev.starts_with(r.as_str()));
        if others || !agree {
            return Err(conflict(name, &url, needs, &reading.versions(tags)));
        }
        if gate.active() {
            return Err(DepError::new(format!(
                "{name} rev {rev}'s age can't be checked: a commit has no release"
            ))
            .note("a shared dependency pinned by commit needs `min_release_age = \"0\"` in its requirers' policy"));
        }
        return Ok(Pin {
            commit: (*rev).clone(),
            tag: None,
            rev: Some((*rev).clone()),
            version: None,
        });
    }
    let found = candidates(reading, needs, tags);
    if found.is_empty() {
        return Err(conflict(name, &url, needs, &reading.versions(tags)));
    }
    accept_newest(ctx, &gate, &found, common, |t: &Tagged| Pin {
        commit: t.commit.clone(),
        tag: Some(t.tag.clone()),
        rev: None,
        version: None,
    })
}

/// Checks out `pin` as the shared copy of `name` and records the entry.
fn install(
    ctx: &mut Ctx,
    project: &mut Project,
    name: &str,
    common: &Dependency,
    needs: &[Need],
    pin: &Pin,
    existing: Option<&Dependency>,
) -> Result<()> {
    let path = format!("modules/{name}");
    let url = spec::url(&common.git);
    if let Some(old) = existing {
        ctx.git.remove(&project.dir, &path_of(name, old))?;
    } else if project
        .dir
        .join(&path)
        .read_dir()
        .is_ok_and(|mut d| d.next().is_some())
    {
        return Err(DepError::new(format!(
            "{path} exists and is not a dependency, so {name} cannot be shared there"
        ))
        .note(format!(
            "{} need it",
            needs
                .iter()
                .map(|n| n.by.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .note(format!(
            "`jacket dep add <spec> --name {name}` replaces it with a pinned copy"
        )));
    }
    ctx.git.add(&project.dir, &url, &path, &pin.commit)?;
    let entry = Dependency {
        tag: pin.tag.clone(),
        rev: pin.rev.clone(),
        shared: true,
        ..common.clone()
    };
    let text = manifest::set_dependency(&project.text, name, &entry).map_err(DepError::new)?;
    project.save(text)?;
    let shown = pin
        .tag
        .clone()
        .unwrap_or_else(|| format!("rev {}", short(&pin.commit)));
    let by: Vec<&str> = needs.iter().map(|n| n.by.as_str()).collect();
    say(
        ctx,
        format!(
            "shared {name} {shown} for {} ({})",
            by.join(", "),
            short(&pin.commit)
        ),
    );
    Ok(())
}

/// The paths, inside the package at `path`, of nested submodules the project covers itself.
pub(super) fn covered(project: &Project, path: &str) -> Vec<String> {
    if project.config.resolver.dedupe != Some(true) {
        return Vec::new();
    }
    let Some(manifest) = package::read(&project.dir.join(path)) else {
        return Vec::new();
    };
    manifest
        .dependencies
        .iter()
        .filter(|(name, _)| project.config.dependencies.contains_key(*name))
        .map(|(name, wanted)| path_of(name, wanted))
        .collect()
}

/// Checks out what is nested inside every dependency, leaving out the copies the project
/// shares.
pub(super) fn checkout_nested(ctx: &mut Ctx, project: &Project) -> Result<()> {
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry);
        let skip = covered(project, &path);
        ctx.git.nested(&project.dir, &path, &skip)?;
    }
    Ok(())
}

/// Nested copies that cannot be shared because they are files in the package, not submodules.
fn report_vendored(ctx: &mut Ctx, project: &Project) {
    for (by, entry) in &project.config.dependencies {
        let path = path_of(by, entry);
        let Some(manifest) = package::read(&project.dir.join(&path)) else {
            continue;
        };
        let nested = ctx
            .git
            .submodules(&project.dir.join(&path))
            .unwrap_or_default();
        for (name, wanted) in &manifest.dependencies {
            let inner = path_of(name, wanted);
            let vendored = project
                .dir
                .join(&path)
                .join(&inner)
                .join("module.jai")
                .is_file()
                && !nested.iter().any(|s| s.path == inner);
            if vendored && project.config.dependencies.contains_key(name) {
                let _ = writeln!(
                    ctx.err,
                    "warning: {by} vendors its own copy of {name} at {inner}, which can't be shared"
                );
            }
        }
    }
}

/// What `dep check` adds under dedupe: every requirement is covered and satisfied.
pub(super) fn problems(ctx: &Ctx, project: &Project) -> Vec<String> {
    let mut out = Vec::new();
    if project.config.resolver.dedupe != Some(true) {
        return out;
    }
    let Ok(known) = commits(ctx, project) else {
        return out;
    };
    for (name, asked) in gather(ctx, project) {
        let Some(entry) = project.config.dependencies.get(&name) else {
            let by: Vec<&str> = asked.iter().map(|n| n.by.as_str()).collect();
            out.push(format!(
                "{name} is needed by {} but is not a dependency: run `jacket dep update`",
                by.join(", ")
            ));
            continue;
        };
        let Ok(reading) = Reading::of(entry) else {
            continue;
        };
        let path = path_of(&name, entry);
        let commit = known.get(&path).cloned().unwrap_or_default();
        let tag = tag_of(ctx, project, entry, &path, &commit);
        for need in &asked {
            if !admits(&reading, need, tag.as_deref(), &commit) {
                out.push(format!(
                    "{name} is pinned at {}, but {}",
                    tag.clone()
                        .unwrap_or_else(|| format!("rev {}", short(&commit))),
                    need.says()
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn need(by: &str, version: Option<&str>, tag: Option<&str>) -> Need {
        Need {
            by: by.into(),
            entry: Dependency {
                git: "x/q".into(),
                version: version.map(String::from),
                tag: tag.map(String::from),
                ..Dependency::default()
            },
        }
    }

    fn tags(names: &[&str]) -> Vec<RemoteTag> {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| RemoteTag {
                name: (*name).into(),
                commit: format!("{i:040}"),
            })
            .collect()
    }

    fn names(found: Vec<Tagged>) -> Vec<String> {
        found.into_iter().map(|t| t.tag).collect()
    }

    #[test]
    fn the_candidates_are_what_every_requirer_admits() {
        let reading = Reading::default();
        let all = tags(&["v1.0.0", "v1.2.0", "v1.4.0", "v2.0.0", "v2.1.0-rc.1"]);
        let diamond = [need("a", Some("^1.2"), None), need("b", Some("<1.4"), None)];
        assert_eq!(names(candidates(&reading, &diamond, &all)), ["v1.2.0"]);
        let open = [need("a", Some(">=1.0"), None)];
        assert_eq!(
            names(candidates(&reading, &open, &all)),
            ["v2.0.0", "v1.4.0", "v1.2.0", "v1.0.0"]
        );
        let clash = [need("a", Some("^1"), None), need("b", Some("^2"), None)];
        assert!(candidates(&reading, &clash, &all).is_empty());
    }

    #[test]
    fn an_exact_tag_is_admitted_only_by_ranges_that_contain_it() {
        let reading = Reading::default();
        let all = tags(&["v1.0.0", "v1.2.0", "v2.0.0"]);
        let fits = [need("a", None, Some("v1.2.0")), need("b", Some("^1"), None)];
        assert_eq!(names(candidates(&reading, &fits, &all)), ["v1.2.0"]);
        let misses = [need("a", None, Some("v2.0.0")), need("b", Some("^1"), None)];
        assert!(candidates(&reading, &misses, &all).is_empty());
    }

    #[test]
    fn a_prerelease_needs_a_requirer_that_names_it() {
        let reading = Reading::default();
        let all = tags(&["v1.0.0", "v2.0.0-rc.1"]);
        let any = [need("a", None, None)];
        assert_eq!(names(candidates(&reading, &any, &all)), ["v1.0.0"]);
        let named = [need("a", None, Some("v2.0.0-rc.1"))];
        assert_eq!(names(candidates(&reading, &named, &all)), ["v2.0.0-rc.1"]);
    }

    #[test]
    fn requirers_must_agree_on_the_repository_and_how_tags_read() {
        let mut other = need("b", None, None);
        other.entry.git = "y/q".into();
        let error = common("q", &[need("a", None, None), other]).unwrap_err();
        assert!(
            error.message.contains("different repositories"),
            "{error:?}"
        );
        let mut calendar = need("b", None, None);
        calendar.entry.scheme = Some("calendar".into());
        let error = common("q", &[need("a", None, None), calendar]).unwrap_err();
        assert!(
            error.message.contains("read its tags differently"),
            "{error:?}"
        );
    }
}
