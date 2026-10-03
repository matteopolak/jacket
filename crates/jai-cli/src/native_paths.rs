//! Resolve native path aliases without requiring the final input to exist.
use crate::Error;
use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

pub fn lexical(path: &Path) -> Result<PathBuf, Error> {
    let path = absolute(path)?;
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component.as_os_str()),
        }
    }
    Ok(result)
}

pub fn resolve(path: &Path) -> Result<PathBuf, Error> {
    resolve_absolute(&absolute(path)?, 0)
}

fn absolute(path: &Path) -> Result<PathBuf, Error> {
    if path.is_absolute() {
        Ok(path.into())
    } else {
        let directory = env::current_dir().map_err(|cause| Error::Io {
            path: ".".into(),
            cause,
        })?;
        Ok(directory.join(path))
    }
}

fn resolve_absolute(path: &Path, links: usize) -> Result<PathBuf, Error> {
    if links >= 64 {
        return Err(Error::Source(
            "native path exceeds the symbolic link resolution limit".into(),
        ));
    }
    let mut result = PathBuf::new();
    let mut components = path.components();
    while let Some(component) = components.next() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(name) => {
                let candidate = result.join(name);
                match fs::symlink_metadata(&candidate) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        let target = fs::read_link(&candidate).map_err(|cause| Error::Io {
                            path: candidate,
                            cause,
                        })?;
                        let target = if target.is_absolute() {
                            target
                        } else {
                            result.join(target)
                        };
                        return resolve_absolute(&target.join(components.as_path()), links + 1);
                    }
                    Ok(_) => result = candidate,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        result = candidate
                    }
                    Err(cause) => {
                        return Err(Error::Io {
                            path: candidate,
                            cause,
                        });
                    }
                }
            }
            _ => result.push(component.as_os_str()),
        }
    }
    Ok(result)
}
