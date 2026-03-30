/// Generates a `String` new-type struct with `Deref`, `Serialize`/`Deserialize`, and optionally
/// `Display`, `From<String>`, `From<&str>`, `FromStr`, and `Into<String>` impls.
///
/// Variants: `impl_new_string!(Name)` for all impls; `no_display`, `no_from`, or `no_into`
/// to omit the corresponding conversion group.
macro_rules! impl_new_string {
    (base $name:ident) => {
        #[doc = "AWS AppSync specific GraphQL scalar type implemented a [String] new-type"]
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(transparent)]
        pub struct $name(String);
        impl core::ops::Deref for $name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
    (display $name:ident) => {
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }
    };
    (from $name:ident) => {
        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
        impl core::str::FromStr for $name {
            type Err = core::convert::Infallible;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::from(s))
            }
        }
    };
    (into $name:ident) => {
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
    (no_display $name:ident) => {
        impl_new_string!(base $name);
        impl_new_string!(from $name);
        impl_new_string!(into $name);
    };
    (no_from $name:ident) => {
        impl_new_string!(base $name);
        impl_new_string!(display $name);
        impl_new_string!(into $name);
    };
    (no_into $name:ident) => {
        impl_new_string!(base $name);
        impl_new_string!(display $name);
        impl_new_string!(from $name);
    };
    ($name:ident) => {
        impl_new_string!(base $name);
        impl_new_string!(display $name);
        impl_new_string!(from $name);
        impl_new_string!(into $name);
    };
}

pub mod datetime;
pub mod email;
pub mod phone;
pub mod timestamp;
pub mod url;
