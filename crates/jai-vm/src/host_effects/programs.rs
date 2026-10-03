//! Exact reviewed exec identities; selection performs no OS lookup or execution.
use super::{HostError, ProgramInvocation};
use std::{
    ffi::{OsStr, OsString},
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Copy, Debug)]
pub struct ProgramScopeLimits {
    pub entries: usize,
    pub bytes: usize,
    pub arguments: usize,
}
impl Default for ProgramScopeLimits {
    fn default() -> Self {
        Self {
            entries: 128,
            bytes: 1024 * 1024,
            arguments: 4096,
        }
    }
}
/// Created by a trusted embedding from an existing provider registration. The
/// registered native argv[0] must equal `argument_zero`; source cannot mint this.
#[derive(Clone, Debug)]
pub struct ReviewedProgramGrant {
    executable: OsString,
    argument_zero: OsString,
    invocation: ProgramInvocation,
    working_directory: PathBuf,
}
impl ReviewedProgramGrant {
    pub fn from_registered(
        executable: OsString,
        argument_zero: OsString,
        invocation: ProgramInvocation,
        canonical_working_directory: PathBuf,
    ) -> Result<Self, HostError> {
        if executable.is_empty()
            || executable.as_encoded_bytes().contains(&0)
            || argument_zero.as_encoded_bytes().contains(&0)
        {
            return Err(HostError::InvalidArguments);
        }
        if !canonical_working_directory.is_absolute()
            || canonical_working_directory
                .as_os_str()
                .as_encoded_bytes()
                .contains(&0)
            || canonical_working_directory
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
        {
            return Err(HostError::InvalidPath);
        }
        Ok(Self {
            executable,
            argument_zero,
            invocation,
            working_directory: canonical_working_directory,
        })
    }
    pub fn invocation(&self) -> &ProgramInvocation {
        &self.invocation
    }
    pub fn argument_zero(&self) -> &OsStr {
        &self.argument_zero
    }
}
#[derive(Clone, Debug)]
pub struct ReviewedPrograms {
    grants: Vec<ReviewedProgramGrant>,
    limits: ProgramScopeLimits,
    work_cost: u64,
}
impl ReviewedPrograms {
    pub fn new(
        grants: Vec<ReviewedProgramGrant>,
        limits: ProgramScopeLimits,
    ) -> Result<Self, HostError> {
        if grants.len() > limits.entries {
            return Err(HostError::Budget("reviewed program count"));
        }
        let mut bytes = 0usize;
        let mut metadata = 0usize;
        for (index, grant) in grants.iter().enumerate() {
            let arguments = grant.invocation.arguments.as_slice();
            metadata = metadata
                .checked_add(6)
                .and_then(|count| count.checked_add(arguments.len()))
                .ok_or(HostError::Budget("reviewed program metadata"))?;
            if arguments
                .len()
                .checked_add(1)
                .is_none_or(|count| count > limits.arguments)
            {
                return Err(HostError::Budget("reviewed argument count"));
            }
            for value in std::iter::once(grant.executable.as_os_str())
                .chain(std::iter::once(grant.argument_zero.as_os_str()))
                .chain(std::iter::once(grant.working_directory.as_os_str()))
                .chain(arguments.iter().map(OsString::as_os_str))
            {
                bytes = bytes
                    .checked_add(value.as_encoded_bytes().len())
                    .ok_or(HostError::Budget("reviewed program bytes"))?;
                if bytes > limits.bytes {
                    return Err(HostError::Budget("reviewed program bytes"));
                }
            }
            if grants[..index].iter().any(|previous| {
                previous.executable == grant.executable
                    && previous.argument_zero == grant.argument_zero
                    && previous.invocation.arguments == grant.invocation.arguments
                    && previous.working_directory == grant.working_directory
            }) {
                return Err(HostError::Denied("ambiguous reviewed invocation"));
            }
        }
        let work_cost = bytes
            .checked_add(metadata)
            .and_then(|count| u64::try_from(count).ok())
            .ok_or(HostError::Budget("reviewed program work"))?;
        Ok(Self {
            grants,
            limits,
            work_cost,
        })
    }
    /// Cached lookup/clone work including empty arguments and grant metadata.
    /// Charge this before scanning the registry or cloning a selected invocation.
    pub fn work_cost(&self) -> u64 {
        self.work_cost
    }
    pub fn resolve(
        &self,
        executable: &OsStr,
        argv: &[OsString],
        canonical_working_directory: &Path,
    ) -> Result<ProgramInvocation, HostError> {
        let Some(argument_zero) = argv.first() else {
            return Err(HostError::InvalidArguments);
        };
        if argv.len() > self.limits.arguments {
            return Err(HostError::Budget("source argument count"));
        }
        if executable.as_encoded_bytes().contains(&0)
            || canonical_working_directory
                .as_os_str()
                .as_encoded_bytes()
                .contains(&0)
        {
            return Err(HostError::InvalidArguments);
        }
        let mut bytes = executable
            .as_encoded_bytes()
            .len()
            .checked_add(
                canonical_working_directory
                    .as_os_str()
                    .as_encoded_bytes()
                    .len(),
            )
            .ok_or(HostError::Budget("source argument bytes"))?;
        for argument in argv {
            if argument.as_encoded_bytes().contains(&0) {
                return Err(HostError::InvalidArguments);
            }
            bytes = bytes
                .checked_add(argument.as_encoded_bytes().len())
                .ok_or(HostError::Budget("source argument bytes"))?;
        }
        if bytes > self.limits.bytes {
            return Err(HostError::Budget("source argument bytes"));
        }
        self.grants
            .iter()
            .find(|grant| {
                grant.executable == executable
                    && grant.argument_zero == *argument_zero
                    && grant.invocation.arguments.as_slice() == &argv[1..]
                    && grant.working_directory == canonical_working_directory
            })
            .map(|grant| grant.invocation.clone())
            .ok_or(HostError::Denied(
                "invocation has no exact reviewed capability",
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_effects::{FileRootId, ProcessArguments, ProgramId};
    fn grant() -> ReviewedProgramGrant {
        ReviewedProgramGrant::from_registered(
            "printf".into(),
            "printf".into(),
            ProgramInvocation {
                program: ProgramId::allocate(),
                arguments: ProcessArguments::new(["%s".into(), "two words; $(literal)".into()])
                    .unwrap(),
                working_root: FileRootId::allocate(),
            },
            "/own-fixture".into(),
        )
        .unwrap()
    }
    #[test]
    fn exact_exec_spelling_argv_zero_argument_boundaries_and_directory_are_required() {
        let grant = grant();
        let expected = grant.invocation().clone();
        let scope = ReviewedPrograms::new(vec![grant], Default::default()).unwrap();
        let argv = vec!["printf".into(), "%s".into(), "two words; $(literal)".into()];
        assert_eq!(
            scope
                .resolve(OsStr::new("printf"), &argv, Path::new("/own-fixture"))
                .unwrap(),
            expected
        );
        assert!(
            scope
                .resolve(
                    OsStr::new("/usr/bin/printf"),
                    &argv,
                    Path::new("/own-fixture")
                )
                .is_err()
        );
        assert!(
            scope
                .resolve(OsStr::new("printf"), &argv, Path::new("/other"))
                .is_err()
        );
        let mut changed = argv.clone();
        changed[0] = "other-program".into();
        assert!(
            scope
                .resolve(OsStr::new("printf"), &changed, Path::new("/own-fixture"))
                .is_err()
        );
        let changed = vec![
            "printf".into(),
            "%s".into(),
            "two".into(),
            "words; $(literal)".into(),
        ];
        assert!(
            scope
                .resolve(OsStr::new("printf"), &changed, Path::new("/own-fixture"))
                .is_err()
        );
        assert!(
            ReviewedPrograms::new(vec![], Default::default())
                .unwrap()
                .resolve(OsStr::new("printf"), &argv, Path::new("/own-fixture"))
                .is_err()
        );
    }
    #[test]
    fn reviewed_scope_bounds_and_ambiguous_grants_fail_closed() {
        let first = grant();
        assert!(
            ReviewedPrograms::new(vec![first.clone(), first.clone()], Default::default()).is_err()
        );
        assert!(
            ReviewedPrograms::new(
                vec![first.clone()],
                ProgramScopeLimits {
                    entries: 0,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert!(
            ReviewedPrograms::new(
                vec![first.clone()],
                ProgramScopeLimits {
                    bytes: 1,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert!(
            ReviewedPrograms::new(
                vec![first],
                ProgramScopeLimits {
                    arguments: 2,
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
    #[test]
    fn cached_work_charges_empty_argument_metadata_before_registry_scan() {
        let mut first = grant();
        first.invocation.arguments = super::super::ProcessArguments::new([]).unwrap();
        let base = ReviewedPrograms::new(vec![first.clone()], Default::default()).unwrap();
        first.invocation.arguments =
            super::super::ProcessArguments::new([OsString::new()]).unwrap();
        let empty_argument = ReviewedPrograms::new(vec![first], Default::default()).unwrap();
        assert!(base.work_cost() > 0);
        assert_eq!(empty_argument.work_cost(), base.work_cost() + 1);
        assert_eq!(
            empty_argument.clone().work_cost(),
            empty_argument.work_cost()
        );
    }
}
