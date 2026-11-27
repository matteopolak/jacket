//! The `jacket dep` commands. Each reads the project, asks git through [`Git`] and writes
//! `jacket.toml` through [`manifest`].
use super::check;
use super::dedupe;
use super::forge::{self, Repo};
use super::git::Git;
use super::git::RemoteTag;
use super::inspect::{inspect, requirements};
use super::package;
use super::policy::{self, Policy};
use super::resolve::{self, Ask, Asked, Reading, Recorded};
use super::snapshot::Snapshot;
use super::spec::{self, At, RepoUrl, Source};
use super::types::{Age, CommitId, SubPath, Target};
use super::version::{Scheme, Tagged};
use super::{Action, Ctx, DepError, Project, Result, manifest};
use jacket_config::Dependency;
use std::path::Path;

/// The submodule folder of an entry, relative to the project.
pub fn path_of(name: &str, entry: &Dependency) -> Result<SubPath> {
    let text = entry
        .path
        .clone()
        .unwrap_or_else(|| format!("modules/{name}"));
    SubPath::parse(&text).map_err(|e| DepError::new(format!("{name}: {e}")))
}

/// The repository an entry names. A project's entries are checked when it is read, so this
/// fails only for an entry built since.
pub(super) fn source_of(entry: &Dependency) -> Result<Source> {
    Source::parse(&entry.git).map_err(DepError::new)
}

/// The address git fetches for an entry; a relative local path is read from the project's folder.
pub(super) fn url_of(project: &Project, entry: &Dependency) -> Result<RepoUrl> {
    source_of(entry).map(|s| s.resolved(&project.dir).url())
}

pub(super) fn short<T: AsRef<str> + ?Sized>(commit: &T) -> &str {
    let commit = commit.as_ref();
    &commit[..commit.len().min(7)]
}

/// What an entry asks for, as `jacket dep list` writes it.
pub(super) fn asked(entry: &Dependency) -> String {
    let text = match Asked::of(entry) {
        Ok(Asked::Range(range)) => range.to_string(),
        Ok(Asked::Tag(tag)) => tag.to_string(),
        Ok(Asked::Rev(rev)) => format!("rev {rev}"),
        Ok(Asked::Unpinned) | Err(_) => "-".into(),
    };
    if entry.shared {
        format!("{text} (shared)")
    } else {
        text
    }
}

/// A commit chosen for an entry, and how the entry records it.
pub struct Pin {
    pub commit: Target,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub version: Option<String>,
}

pub fn dispatch(action: Action, cwd: &Path, ctx: &mut Ctx) -> Result<bool> {
    let mut project = Project::open(cwd)?;
    ctx.git.preflight(&project.dir)?;
    if asks_forges(&action, &project) {
        ctx.forge.preflight()?;
    }
    match action {
        Action::Add {
            spec,
            name,
            module,
            scheme,
            tag_prefix,
        } => add(
            ctx,
            &mut project,
            cwd,
            spec,
            AddOptions {
                name,
                module,
                scheme,
                tag_prefix,
            },
        )
        .map(|()| true),
        Action::Remove {
            name,
        } => remove(ctx, &mut project, &name).map(|()| true),
        Action::List {
            all_tags,
            offline,
        } => list(ctx, &project, all_tags, offline).map(|()| true),
        Action::Update {
            name,
            to,
        } => update(ctx, &mut project, name.as_deref(), to).map(|()| true),
        Action::Sync => sync(ctx, &project).map(|()| true),
        Action::Check {
            offline,
        } => check::check(ctx, &project, offline),
        Action::Init {
            reseed,
        } => init(ctx, &mut project, reseed).map(|()| true),
    }
}

/// Whether the command may ask a forge for release times: a release-age policy applies to the
/// project or one of its entries, and the command is not offline.
fn asks_forges(action: &Action, project: &Project) -> bool {
    match action {
        Action::Remove {
            ..
        }
        | Action::Sync => return false,
        Action::List {
            offline: true, ..
        }
        | Action::Check {
            offline: true,
        } => return false,
        _ => {}
    }
    let resolver = &project.config.resolver;
    let active = |entry: &Dependency| {
        Policy::from_config(resolver, entry).is_ok_and(|policy| policy.active())
    };
    active(&Dependency::default()) || project.config.dependencies.values().any(active)
}

pub(super) fn say(ctx: &mut Ctx, text: String) {
    let _ = writeln!(ctx.out, "{text}");
}

fn newest_list(tags: &[Tagged]) -> String {
    let names: Vec<&str> = tags.iter().take(5).map(|t| t.tag.as_str()).collect();
    if names.is_empty() {
        "it has no tags that read as versions".to_string()
    } else {
        format!("its newest versions are {}", names.join(", "))
    }
}

/// How a version stands against the release-age policy.
enum Verdict {
    Allowed,
    TooNew { age: i64, required: i64 },
    Unknown(String),
}

/// The policy as it applies to one dependency.
pub(super) struct Gate {
    name: String,
    entry: Dependency,
    policy: Policy,
    repo: Option<Repo>,
}

impl Gate {
    pub(super) fn new(
        project: &Project,
        name: &str,
        entry: &Dependency,
        url: &RepoUrl,
    ) -> Result<Self> {
        let resolver = &project.config.resolver;
        let policy = Policy::from_config(resolver, entry)
            .map_err(|e| DepError::new(format!("{name}: {e}")))?;
        Ok(Self {
            name: name.to_string(),
            entry: entry.clone(),
            policy,
            repo: url.host_and_repo().map(|(host, path)| Repo {
                host,
                path,
            }),
        })
    }

    pub(super) fn active(&self) -> bool {
        self.policy.active()
    }

    fn check(&self, ctx: &mut Ctx, tag: &str, commit: &CommitId) -> Verdict {
        if !self.active() {
            return Verdict::Allowed;
        }
        let Some(repo) = &self.repo else {
            return Verdict::Unknown("a local path keeps no release times".into());
        };
        match forge::published(ctx.forge, ctx.cache, repo, tag, commit) {
            Ok(published) => {
                let age = ctx.now - published;
                let required = self.policy.required(&self.name, tag);
                if age >= required {
                    Verdict::Allowed
                } else {
                    Verdict::TooNew {
                        age,
                        required,
                    }
                }
            }
            Err(unknown) => Verdict::Unknown(unknown.to_string()),
        }
    }

    /// The error for a version the policy does not accept. `entry` is what the override note
    /// shows: the entry as it would read with the waiver.
    fn refuse(&self, subject: &str, verdict: &Verdict, waived: &Dependency) -> DepError {
        let message = match verdict {
            Verdict::TooNew {
                age,
                required,
            } => {
                let mut said = format!(
                    "{} {subject} was published {}; min_release_age is {}",
                    self.name,
                    policy::ago(*age),
                    self.policy.min_age.unwrap_or(Age::ZERO)
                );
                if let Some(jitter) = self.policy.jitter
                    && self.policy.seed.is_some()
                {
                    said.push_str(&format!(
                        ", plus up to {jitter} of jitter (this version waits {})",
                        policy::span(*required)
                    ));
                }
                said
            }
            Verdict::Unknown(why) => {
                format!("{} {subject}'s age can't be checked: {why}", self.name)
            }
            Verdict::Allowed => format!("{} {subject} is allowed", self.name),
        };
        let mut waived = waived.clone();
        waived.min_release_age = Some("0".into());
        let mut error = DepError::new(message).note(format!(
            "to accept it, add `min_release_age = \"0\"` to its entry:\n{}",
            manifest::line(&self.name, &waived)
        ));
        error.too_new = matches!(verdict, Verdict::TooNew { .. });
        error
    }
}

/// The entry as `add` would write it for `pin`, for notes.
fn entry_for(base: &Dependency, pin: &Pin) -> Dependency {
    Dependency {
        tag: pin.tag.clone(),
        rev: pin.rev.clone(),
        version: pin.version.clone(),
        ..base.clone()
    }
}

/// The commit `ask` means among the remote's `tags`, among those the policy accepts.
fn pick(
    ctx: &mut Ctx,
    gate: &Gate,
    url: &RepoUrl,
    reading: &Reading,
    tags: &[RemoteTag],
    ask: Ask,
) -> Result<Pin> {
    let base = Dependency {
        tag: None,
        rev: None,
        version: None,
        ..gate.entry.clone()
    };
    match ask {
        Ask::Tag(tag) => {
            let found = tags
                .iter()
                .find(|t| t.name == tag)
                .ok_or_else(|| DepError::new(format!("{url} has no tag `{tag}`")))?;
            let pin = Pin {
                commit: found.commit.clone().into(),
                tag: Some(tag.to_string()),
                rev: None,
                version: None,
            };
            match gate.check(ctx, &tag, &found.commit) {
                Verdict::Allowed => Ok(pin),
                refused => Err(gate.refuse(&tag, &refused, &entry_for(&base, &pin))),
            }
        }
        Ask::Rev(rev) => {
            let pin = Pin {
                commit: Target::Rev(rev.clone()),
                tag: None,
                rev: Some(rev.as_str().to_string()),
                version: None,
            };
            if gate.active() {
                let why =
                    Verdict::Unknown("a commit has no release, so no recorded publish time".into());
                return Err(gate.refuse(&format!("rev {rev}"), &why, &entry_for(&base, &pin)));
            }
            Ok(pin)
        }
        Ask::Range(requirement) => {
            let text = requirement.as_str();
            let found = resolve::matching(reading, tags, Some(requirement.range()));
            if found.is_empty() {
                let all = resolve::matching(reading, tags, None);
                return Err(DepError::new(format!("no tag of {url} satisfies `{text}`"))
                    .note(newest_list(&all)));
            }
            let recorded = Dependency {
                version: Some(text.to_string()),
                ..base
            };
            accept_newest(ctx, gate, &found, &recorded, |newest| Pin {
                commit: newest.commit.clone().into(),
                tag: None,
                rev: None,
                version: Some(text.to_string()),
            })
        }
        Ask::Latest => {
            let found = resolve::matching(reading, tags, None);
            if !found.is_empty() {
                return accept_newest(ctx, gate, &found, &base, |newest| {
                    match resolve::recorded(reading, newest) {
                        Recorded::Version(range) => Pin {
                            commit: newest.commit.clone().into(),
                            tag: None,
                            rev: None,
                            version: Some(range),
                        },
                        Recorded::Tag(tag) => Pin {
                            commit: newest.commit.clone().into(),
                            tag: Some(tag),
                            rev: None,
                            version: None,
                        },
                    }
                });
            }
            if !tags.is_empty() {
                let _ = writeln!(
                    ctx.err,
                    "warning: {url} has {} but none read as {} versions, so the default branch's commit is pinned\nnote: pin a tag with `@<tag>`, or pass --tag-prefix or --scheme",
                    plural(tags.len(), "tag"),
                    scheme_name(reading)
                );
            }
            let head = ctx.git.remote_head(url)?;
            let pin = Pin {
                rev: Some(short(&head).to_string()),
                commit: head.into(),
                tag: None,
                version: None,
            };
            if gate.active() {
                let why =
                    Verdict::Unknown("a commit has no release, so no recorded publish time".into());
                let subject = format!("rev {}", short(&pin.commit));
                return Err(gate.refuse(&subject, &why, &entry_for(&base, &pin)));
            }
            Ok(pin)
        }
    }
}

/// The newest of `found` (newest first) that the policy accepts. A version that is too new is
/// skipped for an older one; one whose age cannot be checked stops the search.
pub(super) fn accept_newest(
    ctx: &mut Ctx,
    gate: &Gate,
    found: &[Tagged],
    waived: &Dependency,
    shape: impl Fn(&Tagged) -> Pin,
) -> Result<Pin> {
    let mut newest_refusal: Option<(String, Verdict)> = None;
    for candidate in found {
        match gate.check(ctx, &candidate.tag, &candidate.commit) {
            Verdict::Allowed => {
                return Ok(shape(candidate));
            }
            Verdict::Unknown(why) => {
                return Err(gate.refuse(&candidate.tag, &Verdict::Unknown(why), waived));
            }
            too_new @ Verdict::TooNew {
                ..
            } => {
                newest_refusal.get_or_insert((candidate.tag.clone(), too_new));
            }
        }
    }
    let (tag, verdict) = newest_refusal.expect("candidates were given");
    Err(gate.refuse(&tag, &verdict, waived))
}

/// Jitter without a seed applies none; says so and how to get one.
fn warn_without_seed(ctx: &mut Ctx, project: &Project) {
    let resolver = &project.config.resolver;
    if resolver.release_age_jitter.is_some() && resolver.seed.is_none() {
        let _ = writeln!(
            ctx.err,
            "warning: release_age_jitter is set but [resolver] seed is not, so no jitter is applied\nnote: run `jacket dep init` to write a seed"
        );
    }
}

fn plural(n: usize, word: &str) -> String {
    format!(
        "{n} {word}{}",
        if n == 1 {
            ""
        } else {
            "s"
        }
    )
}

fn scheme_name(reading: &Reading) -> &'static str {
    match reading.scheme {
        super::version::Scheme::Semver => "semver",
        super::version::Scheme::Calendar => "calendar",
        super::version::Scheme::Integer => "integer",
    }
}

struct AddOptions {
    name: Option<String>,
    module: Option<String>,
    scheme: Option<Scheme>,
    tag_prefix: Option<String>,
}

fn add(
    ctx: &mut Ctx,
    project: &mut Project,
    cwd: &Path,
    parsed: spec::Spec,
    options: AddOptions,
) -> Result<()> {
    let AddOptions {
        name,
        module,
        scheme,
        tag_prefix,
    } = options;
    // A relative local path is read from where the user typed it, once.
    let source = parsed.git.resolved(cwd);
    let url = source.url();
    let explicit = name.is_some();
    let name = name.unwrap_or_else(|| source.name());
    if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(
            DepError::new(format!("`{name}` is not a usable dependency name"))
                .note("pass one with --name"),
        );
    }
    if project.config.dependencies.contains_key(&name) {
        return Err(DepError::new(format!("`{name}` is already a dependency")));
    }
    let path = SubPath::parse(&format!("modules/{name}")).map_err(DepError::new)?;
    let known = ctx.git.submodules(&project.dir)?;
    if known.iter().any(|s| s.path == path) {
        return Err(DepError::new(format!("{path} is already a submodule"))
            .note("`jacket dep init` adopts existing submodules"));
    }
    let full = project.dir.join(&path);
    let vendored = full.exists() && full.read_dir().is_ok_and(|mut d| d.next().is_some());
    if vendored && !explicit {
        return Err(
            DepError::new(format!("{path} already exists and is not a submodule")).note(format!(
                "to replace that copy with a pinned submodule, pass --name {name}"
            )),
        );
    }
    let mut entry = Dependency {
        git: source.to_string(),
        scheme: scheme.map(|s| s.name().to_string()),
        tag_prefix,
        module,
        ..Dependency::default()
    };
    let reading = Reading::of(&entry)?;
    warn_without_seed(ctx, project);
    let gate = Gate::new(project, &name, &entry, &url)?;
    let tags = ctx.git.remote_tags(&url)?;
    let ask = resolve::classify(&reading, &tags, parsed.at.as_ref())?;
    let pin = pick(ctx, &gate, &url, &reading, &tags, ask)?;
    // Everything that can fail without changing anything has been asked; now the tree changes.
    let undo = Snapshot::take(ctx, project)?;
    if vendored {
        ctx.git
            .remove_tree(&project.dir, &path)
            .map_err(|e| DepError::new(format!("{path} already exists: {e}")))?;
    }
    if let Err(error) = ctx.git.add(&project.dir, &url, &path, &pin.commit) {
        return Err(undone(ctx, project, &undo, vendored, error));
    }
    entry.tag.clone_from(&pin.tag);
    entry.rev.clone_from(&pin.rev);
    entry.version.clone_from(&pin.version);
    let tag_now = match pin.tag.clone() {
        Some(tag) => Some(tag),
        None => match ctx.git.head(&project.dir, &path) {
            Ok(head) => resolved_tag(ctx.git, project, &entry, &path, &head),
            Err(error) => return Err(undone(ctx, project, &undo, vendored, error)),
        },
    };
    let found = match inspect(
        ctx,
        project,
        &name,
        explicit,
        &path,
        entry.module.as_deref(),
        tag_now.as_deref(),
    ) {
        Ok(found) => found,
        // Nothing is left behind by a dependency that cannot be used.
        Err(error) => return Err(undone(ctx, project, &undo, vendored, error)),
    };
    let (name, path) = (found.name, found.path);
    entry.module = found.module;
    if project.config.dependencies.contains_key(&name) {
        let error = DepError::new(format!("`{name}` is already a dependency"));
        return Err(undone(ctx, project, &undo, vendored, error));
    }
    if path.as_str() != format!("modules/{name}") {
        entry.path = Some(path.to_string());
    }
    let mut text = match manifest::set_dependency(&project.text, &name, &entry) {
        Ok(text) => text,
        Err(error) => {
            return Err(undone(ctx, project, &undo, vendored, DepError::new(error)));
        }
    };
    if project.config.resolver.seed.is_none() {
        text = manifest::set_seed(&text, &fresh_seed());
    }
    if let Err(error) = project.save(text) {
        return Err(undone(ctx, project, &undo, vendored, error));
    }
    let head = match ctx.git.head(&project.dir, &path) {
        Ok(head) => head,
        Err(error) => return Err(undone(ctx, project, &undo, vendored, error)),
    };
    let shown = resolved_tag(ctx.git, project, &entry, &path, &head)
        .or_else(|| entry.tag.clone())
        .unwrap_or_else(|| format!("rev {}", short(&head)));
    let asked = entry
        .version
        .as_deref()
        .map_or(String::new(), |v| format!(" ({v})"));
    // Sharing can add or move other copies; all of them go if it fails.
    if let Err(error) =
        dedupe::run(ctx, project, false).and_then(|()| dedupe::checkout_nested(ctx, project))
    {
        return Err(undone(ctx, project, &undo, vendored, error));
    }
    say(
        ctx,
        format!("added {name} {shown}{asked} at {path} ({})", short(&head)),
    );
    Ok(())
}

/// Undoes a half-made `add`: everything is put back as `before` took it, and a vendored copy
/// that was replaced is named. Returns `error` with a note for each step that could not be undone.
fn undone(
    ctx: &mut Ctx,
    project: &mut Project,
    before: &Snapshot,
    vendored: bool,
    error: DepError,
) -> DepError {
    let mut error = before.restore(ctx, project, error);
    if vendored {
        error = error.note(
            "the copy that was replaced was removed; restore it with `git checkout HEAD -- <path>`",
        );
    }
    error
}

fn remove(ctx: &mut Ctx, project: &mut Project, name: &str) -> Result<()> {
    let Some(entry) = project.config.dependencies.get(name) else {
        return Err(DepError::new(format!("`{name}` is not a dependency")));
    };
    if entry.shared {
        return Err(
            DepError::new(format!("{name} is shared: other dependencies need it"))
                .note("it is removed with the last of them"),
        );
    }
    let path = path_of(name, entry)?;
    let known = ctx.git.submodules(&project.dir)?;
    let before = Snapshot::take(ctx, project)?;
    let (text, _) = manifest::remove_dependency(&project.text, name);
    project.save(text)?;
    // The entry stays until the submodule is gone, and a package that cannot be shared without it
    // brings both back.
    let result = (|| {
        if known.iter().any(|s| s.path == path) {
            ctx.git.remove(&project.dir, &path)?;
        }
        dedupe::run(ctx, project, false)
    })();
    if let Err(error) = result {
        return Err(before.restore(ctx, project, error));
    }
    say(ctx, format!("removed {name} ({path})"));
    Ok(())
}

/// The tag pointing at `commit` that names the resolved version.
pub(super) fn resolved_tag(
    git: &dyn Git,
    project: &Project,
    entry: &Dependency,
    path: &SubPath,
    commit: &CommitId,
) -> Option<String> {
    let tags = git.tags_at(&project.dir, path, commit).ok()?;
    let reading = Reading::of(entry).ok()?;
    resolve::resolved(&reading, &tags).map(ToString::to_string)
}

/// What a remote offers beyond the pin, for `list`'s last column.
fn newer(
    ctx: &Ctx,
    gate: &Gate,
    entry: &Dependency,
    url: &RepoUrl,
    resolved: Option<&str>,
    commit: Option<&CommitId>,
) -> String {
    let Ok(tags) = ctx.git.remote_tags(url) else {
        return "unreachable".into();
    };
    let Ok(reading) = Reading::of(entry) else {
        return "-".into();
    };
    let asked = Asked::of(entry).unwrap_or(Asked::Unpinned);
    if let Asked::Range(requirement) = &asked {
        let range = requirement.range().clone();
        let current = resolved.and_then(|t| reading.parse(t));
        return match (
            resolve::matching(&reading, &tags, Some(&range)).first(),
            current,
        ) {
            (Some(best), Some(current)) if best.version > current => {
                format!(
                    "{}{}",
                    best.tag,
                    age_note(ctx, gate, &best.tag, &best.commit)
                )
            }
            (Some(best), None) if Some(&best.commit) != commit => {
                format!(
                    "{}{}",
                    best.tag,
                    age_note(ctx, gate, &best.tag, &best.commit)
                )
            }
            _ => "-".into(),
        };
    }
    if matches!(asked, Asked::Rev(_)) {
        return match ctx.git.remote_head(url) {
            Ok(head) if Some(&head) != commit => format!("{} (new commits)", short(&head)),
            _ => "-".into(),
        };
    }
    "-".into()
}

/// What the cache alone says about a version's age, for `list` (which never asks a forge).
fn age_note(ctx: &Ctx, gate: &Gate, tag: &str, commit: &CommitId) -> &'static str {
    if !gate.active() {
        return "";
    }
    match gate
        .repo
        .as_ref()
        .and_then(|r| ctx.cache.known(r, tag, commit))
    {
        Some(published) if ctx.now - published >= gate.policy.required(&gate.name, tag) => "",
        Some(_) => " (too new)",
        None => " (age unchecked)",
    }
}

fn list(ctx: &mut Ctx, project: &Project, all_tags: bool, offline: bool) -> Result<()> {
    if project.config.dependencies.is_empty() {
        say(ctx, "no dependencies".into());
        return Ok(());
    }
    let known = ctx.git.submodules(&project.dir)?;
    let mut header = ["NAME", "ASKED", "RESOLVED", "COMMIT", "PATH"]
        .map(String::from)
        .to_vec();
    if !offline {
        header.push("NEWER".into());
    }
    let mut rows = vec![header];
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry)?;
        let sub = known.iter().find(|s| s.path == path);
        let (resolved, commit) = match sub {
            Some(sub) if ctx.git.initialised(&project.dir, &path)? => (
                resolved_tag(ctx.git, project, entry, &path, &sub.commit),
                short(&sub.commit).to_string(),
            ),
            Some(sub) => (None, short(&sub.commit).to_string()),
            None => (None, "-".into()),
        };
        let shown = match (sub, &resolved) {
            (None, _) => "not a submodule".to_string(),
            (Some(_), Some(tag)) => tag.clone(),
            (Some(_), None) if !ctx.git.initialised(&project.dir, &path)? => {
                "not checked out".into()
            }
            _ => "-".into(),
        };
        let mut row = vec![name.clone(), asked(entry), shown, commit, path.to_string()];
        if !offline {
            let full = sub.map(|s| &s.commit);
            let url = url_of(project, entry)?;
            let gate = Gate::new(project, name, entry, &url)?;
            row.push(newer(ctx, &gate, entry, &url, resolved.as_deref(), full));
        }
        rows.push(row);
    }
    let columns = rows[0].len();
    let widths: Vec<usize> = (0..columns)
        .map(|c| rows.iter().map(|r| r[c].chars().count()).max().unwrap_or(0))
        .collect();
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(cell, w)| format!("{cell:<w$}"))
            .collect();
        say(ctx, cells.join("  ").trim_end().to_string());
    }
    for line in requirements(ctx, project, &known)? {
        say(ctx, line);
    }
    if all_tags {
        for (name, entry) in &project.config.dependencies {
            let tags = ctx.git.remote_tags(&url_of(project, entry)?)?;
            let reading = Reading::of(entry)?;
            let skipped: Vec<&str> = tags
                .iter()
                .filter(|t| reading.parse(t.name.as_str()).is_none())
                .map(|t| t.name.as_str())
                .collect();
            if !skipped.is_empty() {
                say(ctx, format!("{name}: not versions: {}", skipped.join(", ")));
            }
        }
    }
    Ok(())
}

pub(super) fn moved(name: &str, tag: &str, remote: &CommitId, pinned: &CommitId) -> DepError {
    DepError::new(format!(
        "tag {tag} of {name} has moved: the remote has it at {}, the pin is {}",
        short(remote),
        short(pinned)
    ))
    .note("a tag should never be rewritten; ask the maintainer, or pin a commit with `rev`")
}

/// One dependency's move, decided before anything changes.
struct Planned {
    name: String,
    entry: Dependency,
    path: SubPath,
    next: Pin,
}

/// Moves one dependency to its planned commit and records it; returns what to say.
fn apply(ctx: &mut Ctx, project: &mut Project, plan: &Planned, keep_range: bool) -> Result<String> {
    let Planned {
        name,
        entry,
        path,
        next,
        ..
    } = plan;
    ctx.git.move_to(&project.dir, path, &next.commit)?;
    let manifest = package::read(&path.under(&project.dir)).unwrap_or_default();
    package::requires_jacket(&manifest.package, env!("CARGO_PKG_VERSION"))
        .map_err(|why| DepError::new(format!("{name}: {why}")))?;
    let updated = Dependency {
        tag: next.tag.clone(),
        rev: next.rev.clone(),
        version: next.version.clone().or_else(|| {
            // A range stays as the user wrote it.
            entry.version.clone().filter(|_| keep_range)
        }),
        ..entry.clone()
    };
    let text = manifest::set_dependency(&project.text, name, &updated).map_err(DepError::new)?;
    project.save(text)?;
    let head = ctx.git.head(&project.dir, path)?;
    let now = resolved_tag(ctx.git, project, &updated, path, &head)
        .unwrap_or_else(|| format!("rev {}", short(&head)));
    Ok(format!("updated {name} to {now} ({})", short(&head)))
}

fn update(ctx: &mut Ctx, project: &mut Project, only: Option<&str>, to: Option<At>) -> Result<()> {
    let names: Vec<String> = match only {
        Some(name)
            if project
                .config
                .dependencies
                .get(name)
                .is_some_and(|e| e.shared) =>
        {
            return Err(DepError::new(format!(
                "{name} is shared: its version follows the packages that need it"
            ))
            .note("update those, or `jacket dep update` to move every shared version"));
        }
        Some(name) if project.config.dependencies.contains_key(name) => vec![name.to_string()],
        Some(name) => return Err(DepError::new(format!("`{name}` is not a dependency"))),
        None => project
            .config
            .dependencies
            .iter()
            .filter(|(_, e)| !e.shared)
            .map(|(name, _)| name.clone())
            .collect(),
    };
    if to.is_some() && names.len() != 1 {
        return Err(DepError::new("--to needs one dependency name"));
    }
    let known = ctx.git.submodules(&project.dir)?;
    warn_without_seed(ctx, project);
    let mut gates = Vec::new();
    for name in &names {
        let entry = &project.config.dependencies[name];
        gates.push(Gate::new(project, name, entry, &url_of(project, entry)?)?);
    }
    let repos: Vec<Repo> = gates
        .iter()
        .filter(|g| g.active() && !matches!(Asked::of(&g.entry), Ok(Asked::Tag(_))))
        .filter_map(|g| g.repo.clone())
        .collect();
    forge::prefetch(ctx.forge, ctx.cache, &repos);
    let mut plans: Vec<Planned> = Vec::new();
    for (name, gate) in names.into_iter().zip(gates) {
        let entry = project.config.dependencies[&name].clone();
        let path = path_of(&name, &entry)?;
        let url = url_of(project, &entry)?;
        let reading = Reading::of(&entry)?;
        let asked = Asked::of(&entry).map_err(|e| DepError::new(format!("{name}: {e}")))?;
        let pinned = known
            .iter()
            .find(|s| s.path == path)
            .map(|s| s.commit.clone());
        let tags = ctx.git.remote_tags(&url)?;
        // The tag the pin stands on must still point at the pinned commit.
        let named = match &asked {
            Asked::Tag(tag) => Some(tag.to_string()),
            _ => None,
        };
        if let (Some(pinned), true) = (&pinned, ctx.git.initialised(&project.dir, &path)?)
            && let Some(tag) = named
                .clone()
                .or_else(|| resolved_tag(ctx.git, project, &entry, &path, pinned))
        {
            match tags.iter().find(|t| t.name.as_str() == tag) {
                Some(t) if &t.commit != pinned => {
                    return Err(moved(&name, &tag, &t.commit, pinned));
                }
                None if named.is_some() => {
                    return Err(DepError::new(format!(
                        "{name}: the remote no longer has tag {tag}"
                    )));
                }
                _ => {}
            }
        }
        let ask = match (&to, &asked) {
            (Some(at), _) => resolve::classify(&reading, &tags, Some(at))?,
            (None, Asked::Tag(tag)) => {
                say(
                    ctx,
                    format!("{name} is pinned to {tag}; use --to to change it"),
                );
                continue;
            }
            (None, Asked::Range(requirement)) => Ask::Range(requirement.clone()),
            (None, Asked::Rev(_) | Asked::Unpinned) => Ask::Latest,
        };
        let next = match (&ask, &asked) {
            // A commit-pinned entry follows the default branch.
            (Ask::Latest, Asked::Rev(_)) => {
                let head = ctx.git.remote_head(&url)?;
                let pin = Pin {
                    rev: Some(short(&head).to_string()),
                    commit: head.into(),
                    tag: None,
                    version: None,
                };
                if gate.active() {
                    let why = Verdict::Unknown(
                        "a commit has no release, so no recorded publish time".into(),
                    );
                    let subject = format!("rev {}", short(&pin.commit));
                    return Err(gate.refuse(&subject, &why, &entry_for(&entry, &pin)));
                }
                pin
            }
            _ => match pick(ctx, &gate, &url, &reading, &tags, ask) {
                Ok(pin) => pin,
                // Newer versions exist but are not old enough yet: the pin stays.
                Err(error) if error.too_new && pinned.is_some() => {
                    say(ctx, format!("{name}: {}", error.message));
                    continue;
                }
                Err(error) => return Err(error),
            },
        };
        if pinned.as_ref().map(CommitId::as_str) == Some(next.commit.as_str()) {
            say(ctx, format!("{name} is up to date"));
            continue;
        }
        plans.push(Planned {
            name,
            entry,
            path,
            next,
        });
    }
    // Every remote has been asked and every version chosen; only now does anything change.
    let before = Snapshot::take(ctx, project)?;
    let mut lines = Vec::new();
    let result = (|| {
        for plan in &plans {
            lines.push(apply(ctx, project, plan, to.is_none())?);
        }
        dedupe::run(ctx, project, only.is_none())?;
        dedupe::checkout_nested(ctx, project)
    })();
    if let Err(error) = result {
        return Err(before.restore(ctx, project, error));
    }
    for line in lines {
        say(ctx, line);
    }
    Ok(())
}

fn sync(ctx: &mut Ctx, project: &Project) -> Result<()> {
    let known = ctx.git.submodules(&project.dir)?;
    let mut paths = Vec::new();
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry)?;
        if !known.iter().any(|s| s.path == path) {
            return Err(DepError::new(format!(
                "{name} is in jacket.toml but {path} is not a submodule"
            ))
            .note(format!("run `jacket dep remove {name}` and add it again")));
        }
        paths.push(path);
    }
    if paths.is_empty() {
        say(ctx, "no dependencies".into());
        return Ok(());
    }
    ctx.git.sync(&project.dir, &paths)?;
    dedupe::checkout_nested(ctx, project)?;
    say(ctx, format!("checked out {} dependencies", paths.len()));
    Ok(())
}

/// The module folders a compile searches by itself: `<dir>/modules` and `[project] import_path`.
pub(super) fn module_folders(project: &Project) -> Vec<String> {
    let mut folders = vec!["modules".to_string()];
    for dir in &project.config.project.import_path.0 {
        let dir = dir.trim_end_matches('/').trim_start_matches("./");
        if !dir.is_empty() && !folders.iter().any(|f| f == dir) {
            folders.push(dir.to_string());
        }
    }
    folders
}

/// A random lowercase-and-digit text, for `[resolver] seed`.
fn fresh_seed() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut seed = String::new();
    let state = std::collections::hash_map::RandomState::new();
    while seed.len() < 8 {
        let mut hasher = state.build_hasher();
        hasher.write_usize(seed.len());
        let mut n = hasher.finish();
        for _ in 0..4 {
            seed.push(char::from_digit((n % 36) as u32, 36).unwrap_or('0'));
            n /= 36;
        }
    }
    seed
}

fn init(ctx: &mut Ctx, project: &mut Project, reseed: bool) -> Result<()> {
    let known = ctx.git.submodules(&project.dir)?;
    let folders = module_folders(project);
    let mut text = project.text.clone();
    let mut adopted = 0;
    for sub in &known {
        let folder = sub.path.parent().unwrap_or("");
        if !folders.iter().any(|f| f == folder) {
            continue;
        }
        let name = sub.path.name().to_string();
        let mut listed = false;
        for e in project.config.dependencies.values() {
            listed |= path_of(&name, e)? == sub.path;
        }
        if listed || project.config.dependencies.contains_key(&name) {
            continue;
        }
        let Some(url) = &sub.url else {
            continue;
        };
        let mut entry = Dependency {
            git: url.to_string(),
            path: (sub.path.as_str() != format!("modules/{name}")).then(|| sub.path.to_string()),
            ..Dependency::default()
        };
        let tag = if ctx.git.initialised(&project.dir, &sub.path)? {
            resolved_tag(ctx.git, project, &entry, &sub.path, &sub.commit)
        } else {
            None
        };
        match tag {
            Some(tag) => entry.tag = Some(tag),
            None => entry.rev = Some(short(&sub.commit).to_string()),
        }
        let gate = Gate::new(project, &name, &entry, &url_of(project, &entry)?)?;
        let (subject, verdict) = match &entry.tag {
            Some(tag) => (tag.clone(), gate.check(ctx, tag, &sub.commit)),
            None if gate.active() => (
                format!("rev {}", short(&sub.commit)),
                Verdict::Unknown("a commit has no release, so no recorded publish time".into()),
            ),
            None => (String::new(), Verdict::Allowed),
        };
        if !matches!(verdict, Verdict::Allowed) {
            let error = gate.refuse(&subject, &verdict, &entry);
            let _ = writeln!(ctx.err, "warning: {}", error.message);
            for note in &error.notes {
                let mut lines = note.lines();
                let _ = writeln!(ctx.err, "note: {}", lines.next().unwrap_or(""));
                for rest in lines {
                    let _ = writeln!(ctx.err, "      {rest}");
                }
            }
        }
        text = manifest::set_dependency(&text, &name, &entry).map_err(DepError::new)?;
        say(ctx, format!("adopted {name} ({})", asked(&entry)));
        adopted += 1;
    }
    for folder in &folders {
        let dir = project.dir.join(folder);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut vendored: Vec<String> = entries
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                let path = e.path();
                !known.iter().any(|s| project.dir.join(&s.path) == path)
                    && (path.join("module.jai").is_file()
                        || path.extension().is_some_and(|x| x == "jai"))
            })
            .map(|e| format!("{folder}/{}", e.file_name().to_string_lossy()))
            .filter(|p| {
                !project
                    .config
                    .dependencies
                    .iter()
                    .any(|(n, e)| path_of(n, e).is_ok_and(|own| own.as_str() == p))
            })
            .collect();
        vendored.sort();
        for path in vendored {
            say(ctx, format!("vendored, not managed: {path}"));
        }
    }
    if reseed || project.config.resolver.seed.is_none() {
        text = manifest::set_seed(&text, &fresh_seed());
    }
    if text != project.text {
        project.save(text)?;
    }
    if adopted == 0 {
        say(ctx, "no submodules to adopt".into());
    }
    Ok(())
}
