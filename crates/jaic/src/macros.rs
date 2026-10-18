//! Crate-internal declaration macros.

/// Declares a closed enum together with the lookups that replace matching on raw values.
///
/// Two spellings:
///
/// - `enum Name { A = 1, B = 2 }` keys members by integer. It derives `ALL`, `from_raw` (the
///   member whose integer is `raw`) and `declared` (names and values, for drift tests against a
///   Jai twin).
/// - `enum Name by name { A = "a", B = "b" }` keys members by text, as directive and foreign
///   symbol names arrive. It derives `ALL`, `from_name` and `name`.
///
/// Both derive `Clone, Copy, Debug, PartialEq, Eq`.
macro_rules! raw_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident by name {
            $($(#[$vmeta:meta])* $variant:ident = $text:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $name {
            $($(#[$vmeta])* $variant),+
        }

        #[allow(dead_code)]
        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// The member spelled `name`.
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($text => Some($name::$variant),)+
                    _ => None,
                }
            }

            /// How the member is spelled.
            pub fn name(self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$vmeta:meta])* $variant:ident = $value:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $name {
            $($(#[$vmeta])* $variant = $value),+
        }

        #[allow(dead_code)]
        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// The member whose Jai integer value is `raw`.
            pub fn from_raw(raw: u64) -> Option<Self> {
                Self::ALL.iter().copied().find(|member| *member as u64 == raw)
            }

            /// Member names and values, as the drift test compares them with the Jai side.
            #[doc(hidden)]
            pub fn declared() -> Vec<(&'static str, i64)> {
                vec![$((stringify!($variant), $value)),+]
            }
        }
    };
}

pub(crate) use raw_enum;
