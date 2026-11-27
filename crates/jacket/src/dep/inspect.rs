//! What a dependency's own `jacket.toml` adds to `add` and `list`.
use super::commands::{asked, path_of};
use super::git::Submodule;
use super::package;
use super::resolve::{self, Reading};
use super::types::SubPath;
use super::{Ctx, DepError, Project, Result};

/// What `add` settles after the checkout: the dependency's name and folder, and the module
/// folder to record when the repository does not say it itself.
pub(super) struct Inspected {
    pub name: String,
    pub path: SubPath,
    pub module: Option<String>,
}

/// Reads the checkout's `jacket.toml` as data and refuses what this jacket cannot use.
pub(super) fn inspect(
    ctx: &mut Ctx,
    project: &Project,
    name: &str,
    explicit: bool,
    path: &SubPath,
    asked_module: Option<&str>,
    tag: Option<&str>,
) -> Result<Inspected> {
    let checkout = path.under(&project.dir);
    let manifest = package::read(&checkout).unwrap_or_default();
    package::requires_jacket(&manifest.package, env!("CARGO_PKG_VERSION"))
        .map_err(|why| DepError::new(format!("{name}: {why}")))?;
    let module = package::find_module(&checkout, &manifest, asked_module).map_err(|why| {
        DepError::new(format!("{name}: {why}"))
            .note("pass the folder holding module.jai with --module <path>")
    })?;
    let imported = package::import_name(&module, name);
    if let Some(declared) = &manifest.package.name {
        let clash = !module.is_empty() && *declared != imported;
        if clash {
            return Err(DepError::new(format!(
                "{name}: [package] name is {declared}, but its module folder is called {imported}"
            ))
            .note("a module is imported by its folder's name; the package must use that name"));
        }
    }
    if let Some(note) = package::platform_note(&manifest.package, std::env::consts::OS) {
        let _ = writeln!(ctx.err, "warning: {name}: {note}");
    }
    if let (Some(version), Some(tag)) = (&manifest.package.version, tag) {
        let reading = Reading::default();
        if let (Some(declared), Some(tagged)) = (reading.parse(version), reading.parse(tag))
            && declared != tagged
        {
            let _ = writeln!(
                ctx.err,
                "warning: {name}: [package] version is {version}, but the tag is {tag}"
            );
        }
    }
    let record = (!module.is_empty()
        && manifest
            .package
            .module
            .as_deref()
            .map(|m| m.trim_matches('/'))
            != Some(&module))
    .then_some(module.clone());
    let mut found = Inspected {
        name: name.to_string(),
        path: path.clone(),
        module: record,
    };
    // A root module is imported by its folder, which a package may name itself.
    if let (false, true, Some(declared)) = (explicit, module.is_empty(), &manifest.package.name)
        && declared != name
    {
        let to = SubPath::parse(&format!("modules/{declared}"))
            .map_err(|e| DepError::new(format!("{name}: {e}")))?;
        if to.under(&project.dir).exists() || project.config.dependencies.contains_key(declared) {
            return Err(DepError::new(format!(
                "{name}: the package calls itself {declared}, but {to} is taken"
            ))
            .note(format!("pass --name {name} to keep this name")));
        }
        ctx.git.rename(&project.dir, path, &to)?;
        found.name.clone_from(declared);
        found.path = to;
    }
    Ok(found)
}

/// What each checked-out dependency's own `[dependencies]` ask for, against the version its
/// nested submodule is pinned at.
pub(super) fn requirements(
    ctx: &Ctx,
    project: &Project,
    known: &[Submodule],
) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for (name, entry) in &project.config.dependencies {
        let path = path_of(name, entry)?;
        if !known.iter().any(|s| s.path == path) || !ctx.git.initialised(&project.dir, &path)? {
            continue;
        }
        let dir = path.under(&project.dir);
        let Some(manifest) = package::read(&dir) else {
            continue;
        };
        let nested = ctx.git.submodules(&dir)?;
        for (inner, wanted) in &manifest.dependencies {
            let inner_path = path_of(inner, wanted)?;
            let asked = asked(wanted);
            let have = nested
                .iter()
                .find(|s| s.path == inner_path)
                .and_then(|sub| {
                    let tags = ctx.git.tags_at(&dir, &inner_path, &sub.commit).ok()?;
                    let reading = Reading::of(wanted).ok()?;
                    resolve::resolved(&reading, &tags).map(ToString::to_string)
                });
            let verdict = match (&have, &wanted.version) {
                (Some(tag), Some(range)) => {
                    let reading = Reading::of(wanted);
                    let ok = reading.as_ref().is_ok_and(|r| {
                        r.range(range)
                            .ok()
                            .zip(r.parse(tag))
                            .is_some_and(|(range, v)| range.matches(&v))
                    });
                    if ok {
                        format!("pinned at {tag}")
                    } else {
                        format!("pinned at {tag}, which does not satisfy it")
                    }
                }
                (Some(tag), None) => format!("pinned at {tag}"),
                (None, _) => "not checked out".to_string(),
            };
            lines.push(format!("{name} needs {inner} {asked}: {verdict}"));
        }
    }
    Ok(lines)
}
