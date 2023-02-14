#[doc(hidden)]
#[macro_export]
macro_rules! __config_entry {
    () => {};

    (
        $(#[$meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $type:ty
            ),*
            $(,)?
        }

        $($tail:tt)*
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        $(#[$meta])*
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field : $type
            ),*
        }

        impl $name {
            #[doc = concat!(
                "Serializes and saves config entry to the file"
            )]
            pub fn save(&self, to: impl AsRef<std::path::Path>) {
                $crate::config::utils::generic_save(self, to)
            }

            #[doc = concat!(
                "Tries to load and deserialize ",
                stringify!($name),
                " config entry"
            )]
            pub fn try_load(
                path: impl AsRef<std::path::Path>
            ) -> Result<Self, $crate::config::error::ConfigLoadError>
            {
                $crate::config::utils::try_generic_load(path)
            }
        }

        $crate::__config_entry! { $($tail)* }
    };
}

pub mod base;
pub mod error;

pub mod utils;

pub use crate::__config_entry as config_entry;
