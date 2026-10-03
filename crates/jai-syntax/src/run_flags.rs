//! Typed scheduling policy retained from source #run directives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RunFlags {
    pub stallable: bool,
}
