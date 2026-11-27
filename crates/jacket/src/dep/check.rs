//! `jacket dep check`: the project, its submodules and the remotes' tags agree.
use super::commands::{module_folders, moved, path_of, say, short, source_of, url_of};
use super::dedupe;
use super::git::Submodule;
use super::resolve::{self, Asked, Reading};
use super::types::{RefName, Revision, SubPath};
use super::{Ctx, Project, Result};
use jacket_config::Dependency;
use std::collections::BTreeSet;

pub(super) fn check(ctx: &mut Ctx, project: &Project, offline: bool) -> Result<bool> {
    let known = ctx.git.submodules(&project.dir)?;
    let mut problems: Vec<String> = Vec::new();
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry)?;
        let Some(sub) = known.iter().find(|s| s.path == path) else {
            problems.push(format!("{name}: {path} is not a submodule"));
            continue;
        };
        if let Some(url) = &sub.url
            && !source_of(entry).is_ok_and(|own| own.same_repo(url))
        {
            problems.push(format!(
                "{name}: .gitmodules has {url}, jacket.toml has {}",
                entry.git
            ));
        }
        if !ctx.git.initialised(&project.dir, &path)? {
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
    let managed: BTreeSet<SubPath> = project
        .config
        .dependencies
        .iter()
        .map(|(n, e)| path_of(n, e))
        .collect::<Result<_>>()?;
    let folders = module_folders(project);
    for sub in &known {
        if !managed.contains(&sub.path)
            && folders.iter().any(|f| f == sub.path.parent().unwrap_or(""))
        {
            problems.push(format!(
                "{} is a submodule that jacket.toml does not list; run `jacket dep init`",
                sub.path
            ));
        }
    }
    problems.extend(dedupe::problems(ctx, project)?);
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
    let path = &sub.path;
    let tag: RefName = match Asked::of(entry) {
        Err(why) => {
            problems.push(format!("{name}: {why}"));
            return;
        }
        Ok(Asked::Rev(rev)) => {
            if !sub.commit.starts_with(&rev) {
                problems.push(format!(
                    "{name}: rev {rev} does not match the pinned commit {}",
                    short(&sub.commit)
                ));
            }
            return;
        }
        Ok(Asked::Unpinned) => return,
        Ok(Asked::Tag(tag)) => {
            match ctx
                .git
                .local_commit(&project.dir, path, &Revision::from(&tag))
            {
                Ok(Some(local)) if local != sub.commit => problems.push(format!(
                    "{name}: tag {tag} is {} in the checkout, the pin is {}",
                    short(&local),
                    short(&sub.commit)
                )),
                Ok(None) => problems.push(format!("{name}: the checkout has no tag {tag}")),
                _ => {}
            }
            tag
        }
        Ok(Asked::Range(requirement)) => {
            let at = ctx
                .git
                .tags_at(&project.dir, path, &sub.commit)
                .unwrap_or_default();
            let Ok(reading) = Reading::of(entry) else {
                return;
            };
            let admitted: Vec<RefName> = at
                .into_iter()
                .filter(|t| {
                    reading
                        .parse(t.as_str())
                        .is_some_and(|v| requirement.range().matches(&v))
                })
                .collect();
            if admitted.is_empty() {
                problems.push(format!(
                    "{name}: the pinned commit {} has no tag satisfying {requirement}; run `jacket dep update {name}`",
                    short(&sub.commit)
                ));
                return;
            }
            match resolve::resolved(&reading, &admitted) {
                Some(tag) => tag.clone(),
                None => return,
            }
        }
    };
    if offline {
        return;
    }
    let asked = url_of(project, entry).and_then(|u| ctx.git.remote_tags(&u));
    match asked {
        Ok(remote) => match remote.iter().find(|t| t.name == tag) {
            Some(t) if t.commit != sub.commit => {
                problems.push(moved(name, &tag, &t.commit, &sub.commit).message);
            }
            None => problems.push(format!("{name}: the remote no longer has tag {tag}")),
            _ => {}
        },
        Err(error) => problems.push(format!("{name}: {error}")),
    }
}
