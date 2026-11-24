//! The `jacket dep` commands. Each reads the project, asks git through [`Git`] and writes
//! `jacket.toml` through [`manifest`].
use super::git::{Git, Submodule};
use super::{Action, Ctx, DepError, Project, Result, manifest, spec};
use jacket_config::Dependency;
use std::collections::BTreeSet;
use std::path::Path;

/// The submodule folder of an entry, relative to the project.
pub fn path_of(name: &str, entry: &Dependency) -> String {
    entry
        .path
        .clone()
        .unwrap_or_else(|| format!("modules/{name}"))
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(7)]
}

/// Whether `text` could be a commit hash (7 to 40 hex digits).
fn is_rev(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// What an entry asks for, as `jacket dep list` writes it.
fn asked(entry: &Dependency) -> String {
    match (&entry.version, &entry.tag, &entry.rev) {
        (Some(range), ..) => range.clone(),
        (_, Some(tag), _) => tag.clone(),
        (_, _, Some(rev)) => format!("rev {rev}"),
        _ => "-".into(),
    }
}

/// A commit chosen for an entry.
pub struct Pin {
    pub commit: String,
    pub tag: Option<String>,
    pub rev: Option<String>,
}

pub fn dispatch(action: Action, cwd: &Path, ctx: &mut Ctx) -> Result<bool> {
    let mut project = Project::open(cwd)?;
    match action {
        Action::Add {
            spec,
            name,
            module,
        } => add(ctx, &mut project, &spec, name, module).map(|()| true),
        Action::Remove {
            name,
        } => remove(ctx, &mut project, &name).map(|()| true),
        Action::List => list(ctx, &project).map(|()| true),
        Action::Update {
            name,
            to,
        } => update(ctx, &mut project, name.as_deref(), to).map(|()| true),
        Action::Sync => sync(ctx, &project).map(|()| true),
        Action::Check {
            offline,
        } => check(ctx, &project, offline),
        Action::Init {
            reseed,
        } => init(ctx, &mut project, reseed).map(|()| true),
    }
}

fn say(ctx: &mut Ctx, text: String) {
    let _ = writeln!(ctx.out, "{text}");
}

fn require_repo(ctx: &Ctx, project: &Project) -> Result<()> {
    ctx.git.toplevel(&project.dir).map(drop).map_err(|_| {
        DepError::new(format!(
            "{} is not inside a git repository",
            project.dir.display()
        ))
        .note("run `git init` first; dependencies are git submodules")
    })
}

fn choose(ctx: &Ctx, url: &str, at: Option<&str>) -> Result<Pin> {
    let tags = ctx.git.remote_tags(url)?;
    match at {
        Some(at) => {
            if let Some(tag) = tags.iter().find(|t| t.name == at) {
                return Ok(Pin {
                    commit: tag.commit.clone(),
                    tag: Some(tag.name.clone()),
                    rev: None,
                });
            }
            if is_rev(at) {
                return Ok(Pin {
                    commit: at.to_string(),
                    tag: None,
                    rev: Some(at.to_string()),
                });
            }
            Err(DepError::new(format!("{url} has no tag `{at}`")))
        }
        None => {
            let head = ctx.git.remote_head(url)?;
            Ok(Pin {
                rev: Some(short(&head).to_string()),
                commit: head,
                tag: None,
            })
        }
    }
}

fn add(
    ctx: &mut Ctx,
    project: &mut Project,
    spec_text: &str,
    name: Option<String>,
    module: Option<String>,
) -> Result<()> {
    require_repo(ctx, project)?;
    let parsed = spec::parse(spec_text);
    let url = spec::url(&parsed.git);
    let name = name.unwrap_or_else(|| spec::repo_name(&parsed.git));
    if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(
            DepError::new(format!("`{name}` is not a usable dependency name"))
                .note("pass one with --name"),
        );
    }
    if project.config.dependencies.contains_key(&name) {
        return Err(DepError::new(format!("`{name}` is already a dependency")));
    }
    let path = format!("modules/{name}");
    let known = ctx.git.submodules(&project.dir)?;
    if known.iter().any(|s| s.path == path) {
        return Err(DepError::new(format!("{path} is already a submodule"))
            .note("`jacket dep init` adopts existing submodules"));
    }
    let full = project.dir.join(&path);
    if full.exists() && full.read_dir().is_ok_and(|mut d| d.next().is_some()) {
        ctx.git
            .remove_tree(&project.dir, &path)
            .map_err(|e| DepError::new(format!("{path} already exists: {e}")))?;
    }
    let pin = choose(ctx, &url, parsed.at.as_deref())?;
    ctx.git.add(&project.dir, &url, &path, &pin.commit)?;
    let entry = Dependency {
        git: parsed.git,
        tag: pin.tag.clone(),
        rev: pin.rev.clone(),
        module,
        ..Dependency::default()
    };
    let text = manifest::set_dependency(&project.text, &name, &entry).map_err(DepError::new)?;
    project.save(text)?;
    let head = ctx.git.head(&project.dir, &path)?;
    say(
        ctx,
        format!(
            "added {name} {} at {path} ({})",
            pin.tag
                .as_deref()
                .map_or_else(|| format!("rev {}", short(&head)), str::to_string),
            short(&head)
        ),
    );
    Ok(())
}

fn remove(ctx: &mut Ctx, project: &mut Project, name: &str) -> Result<()> {
    let Some(entry) = project.config.dependencies.get(name) else {
        return Err(DepError::new(format!("`{name}` is not a dependency")));
    };
    let path = path_of(name, entry);
    let known = ctx.git.submodules(&project.dir)?;
    if known.iter().any(|s| s.path == path) {
        ctx.git.remove(&project.dir, &path)?;
    }
    let (text, _) = manifest::remove_dependency(&project.text, name);
    project.save(text)?;
    say(ctx, format!("removed {name} ({path})"));
    Ok(())
}

/// The tag pointing at `commit` that names the resolved version: the last in sorted order.
fn resolved_tag(git: &dyn Git, project: &Project, path: &str, commit: &str) -> Option<String> {
    let mut tags = git.tags_at(&project.dir, path, commit).ok()?;
    tags.sort();
    tags.pop()
}

fn list(ctx: &mut Ctx, project: &Project) -> Result<()> {
    if project.config.dependencies.is_empty() {
        say(ctx, "no dependencies".into());
        return Ok(());
    }
    let known = ctx.git.submodules(&project.dir)?;
    let mut rows = vec![["NAME", "ASKED", "RESOLVED", "COMMIT", "PATH"].map(String::from)];
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry);
        let sub = known.iter().find(|s| s.path == path);
        let (resolved, commit) = match sub {
            Some(sub) if ctx.git.initialised(&project.dir, &path) => (
                resolved_tag(ctx.git, project, &path, &sub.commit).unwrap_or_else(|| "-".into()),
                short(&sub.commit).to_string(),
            ),
            Some(sub) => ("not checked out".into(), short(&sub.commit).to_string()),
            None => ("not a submodule".into(), "-".into()),
        };
        rows.push([name.clone(), asked(entry), resolved, commit, path]);
    }
    let widths: Vec<usize> = (0..5)
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
    Ok(())
}

fn moved(name: &str, tag: &str, remote: &str, pinned: &str) -> DepError {
    DepError::new(format!(
        "tag {tag} of {name} has moved: the remote has it at {}, the pin is {}",
        short(remote),
        short(pinned)
    ))
    .note("a tag should never be rewritten; ask the maintainer, or pin a commit with `rev`")
}

fn update(
    ctx: &mut Ctx,
    project: &mut Project,
    only: Option<&str>,
    to: Option<String>,
) -> Result<()> {
    require_repo(ctx, project)?;
    let names: Vec<String> = match only {
        Some(name) if project.config.dependencies.contains_key(name) => vec![name.to_string()],
        Some(name) => return Err(DepError::new(format!("`{name}` is not a dependency"))),
        None => project.config.dependencies.keys().cloned().collect(),
    };
    if to.is_some() && names.len() != 1 {
        return Err(DepError::new("--to needs one dependency name"));
    }
    let known = ctx.git.submodules(&project.dir)?;
    for name in names {
        let entry = project.config.dependencies[&name].clone();
        let path = path_of(&name, &entry);
        let url = spec::url(&entry.git);
        let pinned = known
            .iter()
            .find(|s| s.path == path)
            .map(|s| s.commit.clone());
        let next = match (&to, &entry) {
            (Some(tag), _) => choose(ctx, &url, Some(tag))?,
            (None, e) if e.tag.is_some() => {
                let tag = e.tag.as_deref().unwrap_or_default();
                let remote = ctx.git.remote_tags(&url)?;
                match (remote.iter().find(|t| t.name == tag), &pinned) {
                    (Some(t), Some(p)) if &t.commit != p => {
                        return Err(moved(&name, tag, &t.commit, p));
                    }
                    (None, _) => {
                        return Err(DepError::new(format!(
                            "{name}: the remote no longer has tag {tag}"
                        )));
                    }
                    _ => {}
                }
                say(
                    ctx,
                    format!("{name} is pinned to {tag}; use --to to change it"),
                );
                continue;
            }
            _ => choose(ctx, &url, None)?,
        };
        if pinned.as_deref() == Some(next.commit.as_str()) {
            say(ctx, format!("{name} is up to date"));
            continue;
        }
        ctx.git.move_to(&project.dir, &path, &next.commit)?;
        let updated = Dependency {
            tag: next.tag.clone(),
            rev: next.rev.clone(),
            version: None,
            ..entry
        };
        let text =
            manifest::set_dependency(&project.text, &name, &updated).map_err(DepError::new)?;
        project.save(text)?;
        say(
            ctx,
            format!(
                "updated {name} to {} ({})",
                asked(&updated),
                short(&next.commit)
            ),
        );
    }
    Ok(())
}

fn sync(ctx: &mut Ctx, project: &Project) -> Result<()> {
    require_repo(ctx, project)?;
    let known = ctx.git.submodules(&project.dir)?;
    let mut paths = Vec::new();
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry);
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
    say(ctx, format!("checked out {} dependencies", paths.len()));
    Ok(())
}

/// The module folders a compile searches by itself: `<dir>/modules` and `[project] import_path`.
fn module_folders(project: &Project) -> Vec<String> {
    let mut folders = vec!["modules".to_string()];
    for dir in &project.config.project.import_path.0 {
        let dir = dir.trim_end_matches('/').trim_start_matches("./");
        if !dir.is_empty() && !folders.iter().any(|f| f == dir) {
            folders.push(dir.to_string());
        }
    }
    folders
}

fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(p, _)| p)
}

fn check(ctx: &mut Ctx, project: &Project, offline: bool) -> Result<bool> {
    require_repo(ctx, project)?;
    let known = ctx.git.submodules(&project.dir)?;
    let mut problems: Vec<String> = Vec::new();
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry);
        let Some(sub) = known.iter().find(|s| s.path == path) else {
            problems.push(format!("{name}: {path} is not a submodule"));
            continue;
        };
        if let Some(url) = &sub.url
            && !spec::same_repo(url, &entry.git)
        {
            problems.push(format!(
                "{name}: .gitmodules has {url}, jacket.toml has {}",
                entry.git
            ));
        }
        if !ctx.git.initialised(&project.dir, &path) {
            problems.push(format!(
                "{name}: {path} is not checked out; run `jacket dep sync`"
            ));
            continue;
        }
        match ctx.git.head(&project.dir, &path) {
            Ok(head) if head != sub.commit => problems.push(format!(
                "{name}: {path} is at {}, the pin is {}; run `jacket dep sync`",
                short(&head),
                short(&sub.commit)
            )),
            Err(error) => problems.push(format!("{name}: {error}")),
            Ok(_) => {}
        }
        check_pin(ctx, project, name, entry, sub, offline, &mut problems);
    }
    let managed: BTreeSet<String> = project
        .config
        .dependencies
        .iter()
        .map(|(n, e)| path_of(n, e))
        .collect();
    let folders = module_folders(project);
    for sub in &known {
        if !managed.contains(&sub.path) && folders.iter().any(|f| f == parent(&sub.path)) {
            problems.push(format!(
                "{} is a submodule that jacket.toml does not list; run `jacket dep init`",
                sub.path
            ));
        }
    }
    if problems.is_empty() {
        say(
            ctx,
            format!(
                "{} dependencies agree with their pins",
                project.config.dependencies.len()
            ),
        );
        return Ok(true);
    }
    for problem in problems {
        let _ = writeln!(ctx.err, "error: {problem}");
    }
    Ok(false)
}

fn check_pin(
    ctx: &Ctx,
    project: &Project,
    name: &str,
    entry: &Dependency,
    sub: &Submodule,
    offline: bool,
    problems: &mut Vec<String>,
) {
    let path = path_of(name, entry);
    if let Some(rev) = &entry.rev
        && !sub.commit.starts_with(rev.as_str())
    {
        problems.push(format!(
            "{name}: rev {rev} does not match the pinned commit {}",
            short(&sub.commit)
        ));
    }
    let Some(tag) = &entry.tag else {
        return;
    };
    match ctx.git.local_commit(&project.dir, &path, tag) {
        Ok(Some(local)) if local != sub.commit => problems.push(format!(
            "{name}: tag {tag} is {} in the checkout, the pin is {}",
            short(&local),
            short(&sub.commit)
        )),
        Ok(None) => problems.push(format!("{name}: the checkout has no tag {tag}")),
        _ => {}
    }
    if offline {
        return;
    }
    match ctx.git.remote_tags(&spec::url(&entry.git)) {
        Ok(remote) => match remote.iter().find(|t| &t.name == tag) {
            Some(t) if t.commit != sub.commit => {
                problems.push(moved(name, tag, &t.commit, &sub.commit).message);
            }
            None => problems.push(format!("{name}: the remote no longer has tag {tag}")),
            _ => {}
        },
        Err(error) => problems.push(format!("{name}: {error}")),
    }
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
    require_repo(ctx, project)?;
    let known = ctx.git.submodules(&project.dir)?;
    let folders = module_folders(project);
    let mut text = project.text.clone();
    let mut adopted = 0;
    for sub in &known {
        let folder = parent(&sub.path);
        if !folders.iter().any(|f| f == folder) {
            continue;
        }
        let name = sub.path.rsplit('/').next().unwrap_or(&sub.path).to_string();
        if project
            .config
            .dependencies
            .values()
            .any(|e| path_of(&name, e) == sub.path)
            || project.config.dependencies.contains_key(&name)
        {
            continue;
        }
        let Some(url) = &sub.url else {
            continue;
        };
        let mut entry = Dependency {
            git: url.clone(),
            path: (sub.path != format!("modules/{name}")).then(|| sub.path.clone()),
            ..Dependency::default()
        };
        let tag = ctx
            .git
            .initialised(&project.dir, &sub.path)
            .then(|| resolved_tag(ctx.git, project, &sub.path, &sub.commit))
            .flatten();
        match tag {
            Some(tag) => entry.tag = Some(tag),
            None => entry.rev = Some(short(&sub.commit).to_string()),
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
            .filter_map(|e| e.ok())
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
                    .any(|(n, e)| &path_of(n, e) == p)
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
