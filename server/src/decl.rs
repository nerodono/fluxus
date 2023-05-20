macro_rules! config {
    (
        $(#[$outer_meta:meta])*
        struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident : $field_type:ty
            ),*
            $(,)?
        }

        $($tail:tt)*
    ) => {
        #[derive(
            Debug,
            serde::Serialize,
            serde::Deserialize,
        )]
        $(#[$outer_meta])*
        pub struct $name {
            $(
                $(#[$field_meta])*
                pub $field_name : $field_type
            ),*
        }

        $crate::decl::config! { $($tail)* }
    };
    (
        $(#[$outer_meta:meta])*
        enum $name:ident {
            $($body:tt)*
        }

        $($tail:tt)*
    ) => {
        #[derive(
            Debug,
            serde::Serialize,
            serde::Deserialize,
        )]
        $(#[$outer_meta])*
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($body)*
        }

        $crate::decl::config! { $($tail)* }
    };

    (
        $(#[$outer_meta:meta])*
        int<$integral_type:ident> $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_name:ident
            ),*
            $(,)?
        }
        $($tail:tt)*
    ) => {
        #[derive(
            serde::Serialize,
            serde::Deserialize
        )]
        $(#[$outer_meta])*
        #[serde(rename_all = "snake_case")]
        #[integral_enum::integral_enum($integral_type)]
        pub enum $name {
            $(
                $(#[$field_meta])*
                $field_name
            ),*
        }

        $crate::decl::config! { $($tail)* }
    };

    () => {};
}

pub(crate) use config;
