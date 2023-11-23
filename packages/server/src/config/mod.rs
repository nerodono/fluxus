macro_rules! entity {
    (
        $(#[$outer_meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_ident:ident : $field_tp:ty
            ),*
            $(,)?
        }

        $($tail:tt)*
    ) => {
        $(#[$outer_meta])*
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field_ident : $field_tp
            ),*
        }

        entity!($($tail)*);
    };
    () => {};
}

pub mod logging;
pub mod protocols;
pub mod runtime;

pub mod root;
