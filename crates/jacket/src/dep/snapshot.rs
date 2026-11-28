//! What a command can put back: `jacket.toml` and every submodule's commit, taken before it
//! changes anything.
use super::git::Submodule;
use super::{Ctx, DepError, Project, Result};

/// The project's manifest text and submodules at one moment.
pub(super) struct Snapshot {
    text: String,
    subs: Vec<Submodule>,
}

impl Snapshot {
    pub(super) fn take(ctx: &Ctx, project: &Project) -> Result<Self> {
        Ok(Self {
            text: project.text.clone(),
            subs: ctx.git.submodules(&project.dir)?,
        })
    }

    /// Puts the project back as it was taken and returns `error`, with a note for each step that
    /// could not be undone. Submodules added since are removed, removed ones are added again at
    /// their old commit, and moved ones are moved back.
    pub(super) fn restore(
        &self,
        ctx: &Ctx,
        project: &mut Project,
        mut error: DepError,
    ) -> DepError {
        let now = match ctx.git.submodules(&project.dir) {
            Ok(now) => now,
            // Without knowing what is there, nothing is removed or moved.
            Err(failed) => {
                return error.note(format!(
                    "could not read the submodules to undo the changes: {}",
                    failed.message
                ));
            }
        };
        let mut note = |what: String| error.notes.push(what);

        for sub in now
            .iter()
            .filter(|n| !self.subs.iter().any(|s| s.path == n.path))
        {
            if let Err(failed) = ctx.git.remove(&project.dir, &sub.path) {
                note(format!("could not remove {}: {}", sub.path, failed.message));
            }
        }

        for old in &self.subs {
            let here = now.iter().find(|n| n.path == old.path);
            // A copy of another repository in its place is not moved: the old one comes back.
            // With no url on either side it is not known to be the same repository.
            let same_repo = here.is_some_and(|n| match (&n.url, &old.url) {
                (Some(a), Some(b)) => a.same_repo(b),
                _ => false,
            });
            let other_repo = here.is_some() && !same_repo;

            if let Some(n) = here
                && (n.url.is_none() || old.url.is_none())
            {
                // Neither moved nor replaced: it cannot be told which repository it is.
                if n.commit != old.commit {
                    note(format!(
                        "could not tell whether {} is the repository it was, so it was left at {}",
                        old.path,
                        n.commit.short()
                    ));
                }

                continue;
            }

            let moved = match here {
                Some(n) if n.commit == old.commit && !other_repo => continue,
                Some(_) if !other_repo => {
                    ctx.git
                        .move_to(&project.dir, &old.path, &old.commit.clone().into())
                }
                found => {
                    let gone = match found {
                        Some(_) => ctx.git.remove(&project.dir, &old.path),
                        None => Ok(()),
                    };

                    gone.and_then(|()| match &old.url {
                        Some(url) => ctx.git.add(
                            &project.dir,
                            &url.url(),
                            &old.path,
                            &old.commit.clone().into(),
                        ),
                        None => Err(DepError::new("it has no url in .gitmodules")),
                    })
                }
            };

            if let Err(failed) = moved {
                note(format!(
                    "could not put {} back at {}: {}",
                    old.path,
                    old.commit.short(),
                    failed.message
                ));
            }
        }

        if std::fs::read_to_string(&project.file).ok().as_deref() != Some(self.text.as_str())
            && let Err(failed) = project.save(self.text.clone())
        {
            note(format!("could not restore jacket.toml: {}", failed.message));
        }

        error
    }
}
