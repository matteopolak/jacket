//! `[resolver] dedupe` (experimental): one copy of each module for the whole program.
//!
//! What each checked-out dependency asks for in its own `[dependencies]` is solved together with
//! the project's entries. A name no entry of the project covers becomes an entry with
//! `shared = true`, pinned to the newest version every requirer admits, as a submodule under
//! the project's `modules/`. The nested copies it replaces are never checked out, so a
//! library's `#import` reaches the shared one. The compiler is unchanged.
use super::commands::{
    Gate, Pin, accept_newest, path_of, resolved_tag, say, short, source_of, url_of,
};
use super::git::RemoteTag;
use super::package;
use super::resolve::{Asked, Reading};
use super::snapshot::Snapshot;
use super::spec::RepoUrl;
use super::types::{CommitId, Rev, SubPath, Target};
use super::version::Tagged;
use super::{Ctx, DepError, Project, Result, manifest};
use jacket_config::Dependency;
use jacket_config::packages::import_name;
use std::collections::BTreeMap;

/// One package's request for a name.
#[derive(Clone, Debug)]
pub struct Need {
    pub by: String,
    pub entry: Dependency,
    pub asked: Asked,
}

impl Need {
    /// A request read once; fails when the entry is not one a package may write.
    pub fn new(by: String, entry: Dependency) -> std::result::Result<Self, String> {
        let asked = Asked::of(&entry)?;

        Ok(Self {
            by,
            entry,
            asked,
        })
    }

    /// What the request says, for messages.
    fn says(&self) -> String {
        let asked = match &self.asked {
            Asked::Range(range) => range.to_string(),
            Asked::Tag(tag) => format!("tag {tag}"),
            Asked::Rev(rev) => format!("rev {rev}"),
            Asked::Unpinned => "any version".into(),
        };
        format!("{} needs {asked}", self.by)
    }
}

/// Whether the version at `tag`/`commit` is one `need` admits.
fn admits(reading: &Reading, need: &Need, tag: Option<&str>, commit: Option<&CommitId>) -> bool {
    match &need.asked {
        Asked::Range(range) => tag
            .and_then(|t| reading.parse(t))
            .is_some_and(|v| range.range().matches(&v)),
        Asked::Tag(wanted) => tag == Some(wanted.as_str()),
        Asked::Rev(rev) => commit.is_some_and(|commit| commit.agrees_with(rev)),
        Asked::Unpinned => true,
    }
}

/// How every requirer reads tags: they must agree on the repository and the scheme.
fn common(name: &str, needs: &[Need]) -> Result<Dependency> {
    let first = &needs[0];

    for other in &needs[1..] {
        if !source_of(&first.entry)?.same_repo(&source_of(&other.entry)?) {
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
            .note("`scheme` and `tag-prefix` must agree"));
        }
    }

    Ok(Dependency {
        git: first.entry.git.clone(),
        scheme: first.entry.scheme.clone(),
        tag_prefix: first.entry.tag_prefix.clone(),
        ..Dependency::default()
    })
}

fn conflict(name: &str, url: &RepoUrl, needs: &[Need], tags: &[Tagged]) -> DepError {
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
    let ranged = needs.iter().any(|n| matches!(n.asked, Asked::Range(_)));

    reading
        .versions(tags)
        .into_iter()
        .filter(|t| {
            let named = needs
                .iter()
                .any(|n| matches!(&n.asked, Asked::Tag(tag) if tag.as_str() == t.tag));

            (!t.version.is_prerelease() || ranged || named)
                && needs
                    .iter()
                    .all(|n| admits(reading, n, Some(&t.tag), Some(&t.commit)))
        })
        .collect()
}

/// What each checked-out entry of the project asks for, by name, and whether some entry is not
/// checked out, so what it asks for is unknown.
fn gather(ctx: &Ctx, project: &Project) -> Result<(BTreeMap<String, Vec<Need>>, bool)> {
    let mut needs: BTreeMap<String, Vec<Need>> = BTreeMap::new();
    let mut unread = false;

    for (by, entry) in &project.config.dependencies {
        let path = path_of(by, entry)?;

        if !ctx.git.initialised(&project.dir, &path)? {
            unread = true;
            continue;
        }

        let Some(manifest) = package::read(&path.under(&project.dir)) else {
            continue;
        };

        for (name, entry) in manifest.dependencies {
            if name != import_name(by)
                && let Ok(need) = Need::new(by.clone(), entry)
            {
                needs.entry(name).or_default().push(need);
            }
        }
    }

    Ok((needs, unread))
}

/// The project's commit for each submodule path.
fn commits(ctx: &Ctx, project: &Project) -> Result<BTreeMap<SubPath, CommitId>> {
    Ok(ctx
        .git
        .submodules(&project.dir)?
        .into_iter()
        .map(|s| (s.path, s.commit))
        .collect())
}

/// The entries of `project` that are copies of `name`: the entry `name` and each `name@<group>`.
fn copies_of<'a>(project: &'a Project, name: &str) -> Vec<(&'a String, &'a Dependency)> {
    project
        .config
        .dependencies
        .iter()
        .filter(|(key, _)| import_name(key) == name)
        .collect()
}

/// Where the copy of `name` for compatibility group `key` is, and the name of its entry: the
/// primary copy is `modules/<name>`, any other `modules/.versions/<name>@<group>/<name>`.
fn place(name: &str, group: &str, primary: bool) -> Result<(String, SubPath)> {
    if primary {
        let path = SubPath::parse(&format!("modules/{name}")).map_err(DepError::new)?;
        return Ok((name.to_string(), path));
    }

    let key = format!("{name}@{group}");
    let path = SubPath::parse(&format!("modules/.versions/{key}/{name}")).map_err(DepError::new)?;

    Ok((key, path))
}

/// The requirers that share one copy of a name.
#[derive(Debug)]
pub struct Group {
    /// The compatibility group: the major version, or `0.<minor>` below 1.0.
    pub key: String,
    /// Indices of the requirers in the needs the group was made from.
    pub needs: Vec<usize>,
    /// The versions every member admits, newest first. Empty when the copy is fixed by the
    /// project's own pin, or chosen by commit.
    pub versions: Vec<Tagged>,
    /// The project pinned this copy itself, so it does not move.
    pub fixed: bool,
}

/// The project's own entry for a name: the one copy it fixes.
pub struct Own {
    pub key: String,
    pub tag: Option<String>,
    pub commit: Option<CommitId>,
}

/// The compatibility group a tag belongs to, or `rev` for a commit.
fn key_of(reading: &Reading, tag: Option<&str>) -> String {
    tag.and_then(|t| reading.parse(t))
        .map_or_else(|| "rev".to_string(), |v| v.compat_key())
}

/// Splits the requirers of `name` into as few groups of compatible versions as admit them all.
/// A group is one copy; a requirer joins the group of the project's pin when that admits it,
/// else the group with the most requirers that admits it. Fails when requirers whose ranges
/// share a group admit no common version, or one admits nothing.
pub fn partition(
    name: &str,
    url: &RepoUrl,
    reading: &Reading,
    needs: &[Need],
    tags: &[RemoteTag],
    own: Option<&Own>,
    prefer: Option<&str>,
) -> Result<Vec<Group>> {
    let everything = || reading.versions(tags);
    let mut groups = Vec::new();
    let mut left: Vec<usize> = (0..needs.len()).collect();

    if let Some(own) = own {
        let (join, rest): (Vec<usize>, Vec<usize>) = left.iter().partition(|i| {
            admits(
                reading,
                &needs[**i],
                own.tag.as_deref(),
                own.commit.as_ref(),
            )
        });

        groups.push(Group {
            key: own.key.clone(),
            needs: join,
            versions: Vec::new(),
            fixed: true,
        });
        left = rest;
    } else if needs.iter().any(|n| matches!(n.asked, Asked::Rev(_))) {
        let revs: Vec<&Rev> = needs
            .iter()
            .filter_map(|n| match &n.asked {
                Asked::Rev(rev) => Some(rev),
                _ => None,
            })
            .collect();
        let longest = revs.iter().max_by_key(|r| r.as_str().len()).expect("some");
        let others = needs.iter().any(|n| !matches!(n.asked, Asked::Rev(_)));

        if others
            || !revs
                .iter()
                .all(|r| longest.as_str().starts_with(r.as_str()))
        {
            return Err(conflict(name, url, needs, &everything()));
        }

        return Ok(vec![Group {
            key: "rev".into(),
            needs: left,
            versions: Vec::new(),
            fixed: false,
        }]);
    }

    let taken = own.map(|o| o.key.as_str());
    // What each requirer left admits, by group.
    let mut keys: BTreeMap<usize, Vec<String>> = BTreeMap::new();

    for &i in &left {
        let admitted = candidates(reading, std::slice::from_ref(&needs[i]), tags);
        let mut found: Vec<String> = Vec::new();

        for t in &admitted {
            let key = t.version.compat_key();

            if Some(key.as_str()) != taken && !found.contains(&key) {
                found.push(key);
            }
        }

        if found.is_empty() {
            return Err(match (own, admitted.is_empty()) {
                (Some(own), false) => pinned(name, own, &needs[i]),
                _ => conflict(name, url, std::slice::from_ref(&needs[i]), &everything()),
            });
        }

        keys.insert(i, found);
    }

    while !left.is_empty() {
        let mut tally: BTreeMap<&String, usize> = BTreeMap::new();

        for i in &left {
            for key in &keys[i] {
                *tally.entry(key).or_default() += 1;
            }
        }

        // Most requirers first, then the group already in use, then the one listed first
        // (every list is newest first, so that is the newest).
        let order: Vec<&String> = keys.values().flatten().collect();
        let best = tally
            .iter()
            .max_by_key(|(key, count)| {
                (
                    **count,
                    Some(key.as_str()) == prefer,
                    std::cmp::Reverse(order.iter().position(|k| k == *key)),
                )
            })
            .map(|(key, _)| (*key).clone())
            .expect("a requirer is left");
        let (join, rest): (Vec<usize>, Vec<usize>) =
            left.iter().partition(|i| keys[*i].contains(&best));
        let members: Vec<Need> = join.iter().map(|i| needs[*i].clone()).collect();
        let versions: Vec<Tagged> = candidates(reading, &members, tags)
            .into_iter()
            .filter(|t| t.version.compat_key() == best)
            .collect();

        if versions.is_empty() {
            return Err(conflict(name, url, &members, &everything()));
        }

        groups.push(Group {
            key: best,
            needs: join,
            versions,
            fixed: false,
        });
        left = rest;
    }

    Ok(groups)
}

/// Solves and applies the shared set. With `fresh`, shared entries move to the newest version
/// their requirers admit; otherwise one that still satisfies everyone stays.
pub(super) fn run(ctx: &mut Ctx, project: &mut Project, fresh: bool) -> Result<()> {
    if !enabled(project) {
        return Ok(());
    }

    // Whatever the rounds changed is put back when one fails.
    let before = Snapshot::take(ctx, project)?;
    settle(ctx, project, fresh).map_err(|error| before.restore(ctx, project, error))
}

/// Whether `[resolver] dedupe` is on: it is unless the project turns it off.
pub(super) fn enabled(project: &Project) -> bool {
    project.config.resolver.dedupe != Some(false)
}

fn settle(ctx: &mut Ctx, project: &mut Project, fresh: bool) -> Result<()> {
    // A choice can bring in a package with requirements of its own, and a cycle of names settles
    // because a name already chosen is only revisited when its constraints grow.
    for _ in 0..64 {
        if !round(ctx, project, fresh)? {
            return report_vendored(ctx, project);
        }
    }

    Err(DepError::new(
        "the shared dependencies did not settle after 64 rounds",
    ))
}

/// A copy to put in place: the entry it is recorded as, and what it stands on.
struct Install {
    key: String,
    path: SubPath,
    pin: Pin,
    existing: Option<Dependency>,
    by: Vec<String>,
}

/// One pass; whether anything changed.
fn round(ctx: &mut Ctx, project: &mut Project, fresh: bool) -> Result<bool> {
    let (needs, unread) = gather(ctx, project)?;
    // Every remote is asked and every version chosen before the first copy changes.
    let mut installs: Vec<(String, Dependency, Install)> = Vec::new();
    let mut stale: Vec<String> = Vec::new();

    for (name, asked) in &needs {
        let known = commits(ctx, project)?;
        let common = common(name, asked)?;
        let reading = Reading::of(&common).map_err(|e| DepError::new(format!("{name}: {e}")))?;
        let url = url_of(project, &common)?;
        let copies = copies_of(project, name);
        let own = match copies
            .iter()
            .find(|(key, e)| key.as_str() == name && !e.shared)
        {
            Some((_, entry)) => {
                let path = path_of(name, entry)?;
                let commit = known.get(&path).cloned();
                let tag = commit
                    .as_ref()
                    .and_then(|c| tag_of(ctx, project, entry, &path, c));

                Some(Own {
                    key: key_of(&reading, tag.as_deref()),
                    tag,
                    commit,
                })
            }
            None => None,
        };
        let shared: Vec<(String, Dependency)> = copies
            .iter()
            .filter(|(_, e)| e.shared)
            .map(|(k, e)| ((*k).clone(), (*e).clone()))
            .collect();

        if !fresh && own.is_none() && settled(ctx, project, &reading, asked, &shared, &known)? {
            continue;
        }

        let tags = ctx.git.remote_tags(&url)?;
        let prefer = shared
            .iter()
            .find(|(key, _)| key == name)
            .and_then(|(_, e)| e.tag.as_deref())
            .map(|tag| key_of(&reading, Some(tag)));
        let groups = partition(
            name,
            &url,
            &reading,
            asked,
            &tags,
            own.as_ref(),
            prefer.as_deref(),
        )?;
        // The project's pin is the primary copy, else the group with the most requirers.
        let primary = match groups.iter().position(|g| g.fixed) {
            Some(at) => at,
            None => (0..groups.len())
                .max_by_key(|&i| {
                    (
                        groups[i].needs.len(),
                        Some(&groups[i].key) == prefer.as_ref(),
                        std::cmp::Reverse(i),
                    )
                })
                .expect("a name with requirers has a group"),
        };
        let mut wanted: Vec<String> = Vec::new();

        for (at, group) in groups.iter().enumerate() {
            let (key, path) = place(name, &group.key, at == primary)?;

            wanted.push(key.clone());

            if group.fixed {
                continue;
            }

            let existing = shared
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, e)| e.clone());
            let members: Vec<Need> = group.needs.iter().map(|i| asked[*i].clone()).collect();

            if !fresh
                && let Some(entry) = &existing
                && entry
                    .tag
                    .as_ref()
                    .is_some_and(|t| group.versions.iter().any(|v| &v.tag == t))
                && known.contains_key(&path)
            {
                continue;
            }

            let pin = choose(ctx, project, name, &common, &members, group)?;

            if let Some(entry) = &existing
                && known.get(&path_of(&key, entry)?).map(CommitId::as_str)
                    == Some(pin.commit.as_str())
                && path_of(&key, entry)? == path
            {
                continue;
            }

            installs.push((
                name.clone(),
                common.clone(),
                Install {
                    key,
                    path,
                    pin,
                    existing,
                    by: members.into_iter().map(|n| n.by).collect(),
                },
            ));
        }

        stale.extend(
            shared
                .into_iter()
                .map(|(k, _)| k)
                .filter(|k| !wanted.contains(k)),
        );
    }

    let mut changed = !installs.is_empty();

    for (name, common, install) in installs {
        put(ctx, project, &name, &common, install)?;
    }

    // A copy no package asks for any more goes with the last one that did; with a package not
    // checked out, who asks is not known, so none goes.
    if !unread {
        stale.extend(
            project
                .config
                .dependencies
                .iter()
                .filter(|(key, e)| e.shared && !needs.contains_key(import_name(key)))
                .map(|(key, _)| key.clone()),
        );
    }

    stale.sort();
    stale.dedup();

    for key in stale {
        let Some(entry) = project.config.dependencies.get(&key).cloned() else {
            continue;
        };
        let path = path_of(&key, &entry)?;

        ctx.git.remove(&project.dir, &path)?;
        prune_empty_parents(project, &path);
        let (text, _) = manifest::remove_dependency(&project.text, &key);

        project.save(text)?;
        say(ctx, format!("removed shared {key} (no package needs it)"));
        changed = true;
    }

    Ok(changed)
}

/// Removes the folders `modules/.versions/<copy>` left empty by taking a copy out.
fn prune_empty_parents(project: &Project, path: &SubPath) {
    let mut dir = path.under(&project.dir);

    while dir.pop() && dir != project.dir.join("modules") {
        if std::fs::remove_dir(&dir).is_err() {
            break;
        }
    }
}

/// Whether the copies in place already serve every requirer and each serves one, so the remote
/// need not be asked.
fn settled(
    ctx: &Ctx,
    project: &Project,
    reading: &Reading,
    needs: &[Need],
    shared: &[(String, Dependency)],
    known: &BTreeMap<SubPath, CommitId>,
) -> Result<bool> {
    let mut seen: Vec<(String, Option<String>, Option<CommitId>)> = Vec::new();

    for (key, entry) in shared {
        let path = path_of(key, entry)?;
        let Some(commit) = known.get(&path) else {
            return Ok(false);
        };

        if !ctx.git.initialised(&project.dir, &path)?
            || !source_of(entry)?.same_repo(&source_of(&needs[0].entry)?)
        {
            return Ok(false);
        }

        let tag = tag_of(ctx, project, entry, &path, commit);

        seen.push((key.clone(), tag, Some(commit.clone())));
    }

    let serves = |n: &Need| {
        seen.iter()
            .any(|(_, tag, commit)| admits(reading, n, tag.as_deref(), commit.as_ref()))
    };
    let used = |tag: &Option<String>, commit: &Option<CommitId>| {
        needs
            .iter()
            .any(|n| admits(reading, n, tag.as_deref(), commit.as_ref()))
    };

    Ok(needs.iter().all(serves) && seen.iter().all(|(_, tag, commit)| used(tag, commit)))
}

/// The tag a checked-out entry stands on.
fn tag_of(
    ctx: &Ctx,
    project: &Project,
    entry: &Dependency,
    path: &SubPath,
    commit: &CommitId,
) -> Option<String> {
    if let Some(tag) = &entry.tag {
        return Some(tag.clone());
    }

    resolved_tag(ctx.git, project, entry, path, commit)
}

/// A requirer the project's own pin does not admit, in a group the pin occupies.
fn pinned(name: &str, own: &Own, need: &Need) -> DepError {
    let at = own
        .tag
        .clone()
        .unwrap_or_else(|| format!("rev {}", own.commit.as_ref().map_or("", CommitId::short)));

    DepError::new(format!(
        "{name} is pinned at {at} in jacket.toml, which not every package accepts"
    ))
    .note(need.says())
    .note(format!(
        "change its entry in jacket.toml, or `jacket dep update {name} --to <tag>`"
    ))
}

/// The commit to put in `group`'s copy: the newest version its requirers admit and the policy
/// accepts, or the commit they name.
fn choose(
    ctx: &mut Ctx,
    project: &Project,
    name: &str,
    common: &Dependency,
    members: &[Need],
    group: &Group,
) -> Result<Pin> {
    let url = url_of(project, common)?;
    let gate = Gate::new(project, name, common, &url)?;

    if group.key == "rev" {
        let rev = members
            .iter()
            .filter_map(|n| match &n.asked {
                Asked::Rev(rev) => Some(rev),
                _ => None,
            })
            .max_by_key(|r| r.as_str().len())
            .expect("a rev group has a rev");

        if gate.active() {
            return Err(DepError::new(format!(
                "{name} rev {rev}'s age can't be checked: a commit has no release"
            ))
            .note("a shared dependency pinned by commit needs `min-release-age = \"0\"` in its requirers' policy"));
        }

        return Ok(Pin {
            commit: Target::Rev(rev.clone()),
            tag: None,
            rev: Some(rev.as_str().to_string()),
            version: None,
        });
    }

    accept_newest(ctx, &gate, &group.versions, common, |t: &Tagged| Pin {
        commit: t.commit.clone().into(),
        tag: Some(t.tag.clone()),
        rev: None,
        version: None,
    })
}

/// Checks out the copy `install` describes and records its entry.
fn put(
    ctx: &mut Ctx,
    project: &mut Project,
    name: &str,
    common: &Dependency,
    install: Install,
) -> Result<()> {
    let Install {
        key,
        path,
        pin,
        existing,
        by,
    } = install;
    let url = url_of(project, common)?;
    // The same repository at the same place only moves; a clone is not thrown away and fetched
    // again, and a failure cannot leave the copy missing.
    let in_place = match &existing {
        Some(old) => {
            path_of(&key, old)? == path
                && matches!((source_of(old), source_of(common)), (Ok(a), Ok(b)) if a.same_repo(&b))
        }
        None => false,
    };

    if in_place {
        ctx.git.move_to(&project.dir, &path, &pin.commit)?;
    } else if let Some(old) = &existing {
        ctx.git.remove(&project.dir, &path_of(&key, old)?)?;
    } else if path
        .under(&project.dir)
        .read_dir()
        .is_ok_and(|mut d| d.next().is_some())
    {
        return Err(DepError::new(format!(
            "{path} exists and is not a dependency, so {name} cannot be shared there"
        ))
        .note(format!("{} need it", by.join(", ")))
        .note(format!(
            "`jacket dep add <spec> --name {name}` replaces it with a pinned copy"
        )));
    }

    if !in_place {
        ctx.git.add(&project.dir, &url, &path, &pin.commit)?;
    }

    let entry = Dependency {
        tag: pin.tag.clone(),
        rev: pin.rev.clone(),
        shared: true,
        path: (key != name).then(|| path.as_str().to_string()),
        ..common.clone()
    };
    let text = manifest::set_dependency(&project.text, &key, &entry).map_err(DepError::new)?;

    project.save(text)?;
    let shown = pin
        .tag
        .clone()
        .unwrap_or_else(|| format!("rev {}", short(&pin.commit)));

    say(
        ctx,
        format!(
            "shared {key} {shown} for {} ({})",
            by.join(", "),
            short(&pin.commit)
        ),
    );
    Ok(())
}

/// The paths, inside the package at `path`, of nested submodules the project covers itself.
pub(super) fn covered(project: &Project, path: &SubPath) -> Result<Vec<SubPath>> {
    if !enabled(project) {
        return Ok(Vec::new());
    }

    let Some(manifest) = package::read(&path.under(&project.dir)) else {
        return Ok(Vec::new());
    };

    manifest
        .dependencies
        .iter()
        .filter(|(name, _)| !copies_of(project, name).is_empty())
        .map(|(name, wanted)| path_of(name, wanted))
        .collect()
}

/// Checks out what is nested inside every dependency, leaving out the copies the project
/// shares.
pub(super) fn checkout_nested(ctx: &mut Ctx, project: &Project) -> Result<()> {
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry)?;
        let skip = covered(project, &path)?;

        ctx.git.nested(&project.dir, &path, &skip)?;
    }

    Ok(())
}

/// Nested copies that cannot be shared because they are files in the package, not submodules.
fn report_vendored(ctx: &mut Ctx, project: &Project) -> Result<()> {
    for (by, entry) in &project.config.dependencies {
        let path = path_of(by, entry)?;
        let Some(manifest) = package::read(&path.under(&project.dir)) else {
            continue;
        };
        let nested = ctx.git.submodules(&path.under(&project.dir))?;

        for (name, wanted) in &manifest.dependencies {
            let inner = path_of(name, wanted)?;
            let vendored = path
                .join(&inner)
                .under(&project.dir)
                .join("module.jai")
                .is_file()
                && !nested.iter().any(|s| s.path == inner);

            if vendored && !copies_of(project, name).is_empty() {
                let _ = writeln!(
                    ctx.err,
                    "warning: {by} vendors its own copy of {name} at {inner}, which can't be shared"
                );
            }
        }
    }

    Ok(())
}

/// What `dep check` adds under dedupe: every requirement is covered by a copy that satisfies it.
pub(super) fn problems(ctx: &Ctx, project: &Project) -> Result<Vec<String>> {
    let mut out = Vec::new();

    if !enabled(project) {
        return Ok(out);
    }

    let known = commits(ctx, project)?;

    for (name, asked) in gather(ctx, project)?.0 {
        let copies = copies_of(project, &name);

        if copies.is_empty() {
            let by: Vec<&str> = asked.iter().map(|n| n.by.as_str()).collect();

            out.push(format!(
                "{name} is needed by {} but is not a dependency: run `jacket dep update`",
                by.join(", ")
            ));

            continue;
        }

        let mut held = Vec::new();

        for (key, entry) in &copies {
            let reading = Reading::of(entry).map_err(|e| DepError::new(format!("{name}: {e}")))?;
            let path = path_of(key, entry)?;
            let commit = known.get(&path).cloned();
            let tag = commit
                .as_ref()
                .and_then(|c| tag_of(ctx, project, entry, &path, c));

            held.push((reading, tag, commit));
        }

        for need in &asked {
            if !held.iter().any(|(reading, tag, commit)| {
                admits(reading, need, tag.as_deref(), commit.as_ref())
            }) {
                let have: Vec<String> = held
                    .iter()
                    .map(|(_, tag, commit)| {
                        tag.clone().unwrap_or_else(|| {
                            format!("rev {}", commit.as_ref().map_or("", CommitId::short))
                        })
                    })
                    .collect();

                out.push(format!(
                    "{name} is at {}, but {}",
                    have.join(", "),
                    need.says()
                ));
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn need(by: &str, version: Option<&str>, tag: Option<&str>) -> Need {
        Need::new(
            by.into(),
            Dependency {
                git: "x/q".into(),
                version: version.map(String::from),
                tag: tag.map(String::from),
                ..Dependency::default()
            },
        )
        .unwrap()
    }

    fn tags(names: &[&str]) -> Vec<RemoteTag> {
        names
            .iter()
            .enumerate()
            .map(|(i, name)| RemoteTag {
                name: crate::dep::types::RefName::parse(name).unwrap(),
                commit: CommitId::numbered(i as u64),
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

    fn url() -> RepoUrl {
        super::super::spec::Source::parse("x/q")
            .unwrap()
            .resolved(std::path::Path::new("/"))
            .url()
    }

    fn split(
        needs: &[Need],
        all: &[RemoteTag],
        own: Option<&Own>,
    ) -> Result<Vec<(String, Vec<usize>)>> {
        partition("q", &url(), &Reading::default(), needs, all, own, None)
            .map(|groups| groups.into_iter().map(|g| (g.key, g.needs)).collect())
    }

    #[test]
    fn incompatible_ranges_split_into_groups_and_the_largest_comes_first() {
        let all = tags(&["v1.0.0", "v1.2.0", "v1.4.0", "v2.0.0"]);
        let needs = [
            need("a", Some("^1.0"), None),
            need("b", Some("^2"), None),
            need("c", Some("^1.1"), None),
        ];

        assert_eq!(
            split(&needs, &all, None).unwrap(),
            [("1".to_string(), vec![0, 2]), ("2".to_string(), vec![1])]
        );
    }

    #[test]
    fn a_range_spanning_groups_joins_the_one_already_needed() {
        let all = tags(&["v1.0.0", "v1.4.0", "v2.0.0"]);
        let needs = [
            need("a", Some("^1"), None),
            need("b", Some(">=1.0"), None),
            need("c", Some("^1.2"), None),
        ];

        assert_eq!(
            split(&needs, &all, None).unwrap(),
            [("1".to_string(), vec![0, 1, 2])]
        );
    }

    #[test]
    fn zero_versions_are_grouped_by_minor() {
        let all = tags(&["v0.1.0", "v0.1.5", "v0.2.0"]);
        let needs = [need("a", Some("^0.1"), None), need("b", Some("^0.2"), None)];
        let keys: Vec<String> = split(&needs, &all, None)
            .unwrap()
            .into_iter()
            .map(|(k, _)| k)
            .collect();

        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"0.1".to_string()) && keys.contains(&"0.2".to_string()));
    }

    #[test]
    fn requirers_of_one_group_that_admit_no_common_version_conflict() {
        let all = tags(&["v1.0.0", "v1.2.0", "v1.4.0"]);
        let needs = [
            need("a", Some("=1.0.0"), None),
            need("b", Some(">=1.2"), None),
        ];
        let error = split(&needs, &all, None).unwrap_err();

        assert!(error.message.contains("no version"), "{error:?}");
    }

    #[test]
    fn the_projects_pin_holds_its_group_and_a_range_in_that_group_that_rejects_it_fails() {
        let all = tags(&["v1.0.0", "v1.2.0", "v1.4.0", "v2.0.0"]);
        let own = Own {
            key: "1".into(),
            tag: Some("v1.2.0".into()),
            commit: Some(CommitId::numbered(1)),
        };
        let needs = [need("a", Some("^1.0"), None), need("b", Some("^2"), None)];

        assert_eq!(
            split(&needs, &all, Some(&own)).unwrap(),
            [("1".to_string(), vec![0]), ("2".to_string(), vec![1])]
        );
        let needs = [need("a", Some("^1.3"), None)];
        let error = split(&needs, &all, Some(&own)).unwrap_err();

        assert!(error.message.contains("pinned at v1.2.0"), "{error:?}");
    }
}
