macro_rules! config {
    () => {};
    (
        $(#[$outer_meta:meta])*
        struct $name:ident {
            $(
                $(#[$inner_meta:meta])*
                $field:ident : $field_ty:ty
            ),*
            $(,)?
        }

        $($tail:tt)*
    ) => {
        $(#[$outer_meta])*
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(
                $(#[$inner_meta])*
                pub $field : $field_ty
            ),*
        }

        $crate::decl::config! { $($tail)* }
    };

    (
        $(#[$outer_meta:meta])*
        enum $name:ident {
            $($content:tt)*
        }

        $($tail:tt)*
    ) => {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        $(#[$outer_meta])*
        pub enum $name {
            $($content)*
        }

        $crate::decl::config! { $($tail)* }
    };

    (
        $(#[$outer_meta:meta])*
        int $name:ident<$integral:ident> {
            $(
                $(#[$inner_meta:meta])*
                $variant:ident
            ),*
            $(,)?
        }
        $($tail:tt)*
    ) => {
        #[integral_enum::integral_enum($integral)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        $(#[$outer_meta])*
        pub enum $name {
            $(
                $(#[$inner_meta])*
                $variant
            ),*
        }

        $crate::decl::config! { $($tail)* }
    };
}

pub(crate) use config;
