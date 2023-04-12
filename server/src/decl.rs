macro_rules! unwrap_bind {
    ($pat:pat = $expr:expr) => {
        let $pat = $expr else { unreachable!() };
    };
}

macro_rules! chan_permits {
    (
        $enum:ident::[
            $(
                [$variant:ident, $type:ty]
            ),*
        ]
    ) => {paste::paste! {
        $(
            #[derive(Clone)]
            pub struct [<$variant Permit>](tokio::sync::mpsc::UnboundedSender<$enum>);

            impl [<$variant Permit>] {
                #[inline]
                pub fn send(&self, command: $type) -> Result<(), tokio::sync::mpsc::error::SendError<$enum>> {
                    self.0.send($enum::$variant(command))
                }
            }
        )*
    }};
}

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

pub(crate) use chan_permits;
pub(crate) use config;
pub(crate) use unwrap_bind;
