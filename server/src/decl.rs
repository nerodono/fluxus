#[allow(unused_macros)]
macro_rules! continue_ {
    ($r:expr) => {
        match $r {
            Ok(v) => v,
            Err(..) => {
                continue;
            }
        }
    };
}

macro_rules! permit_issuers {
    ($(
        $permit_name:ident
    ),*) => {paste::paste! {
        $(
            #[cfg(feature = "" [<$permit_name:lower>] "")]
            pub fn [<issue_ $permit_name:lower _permit>](&self) -> Option<[<$permit_name:camel Permit>]> {
                if matches!(self.data, ProxyData::[<$permit_name:camel>](..)) {
                    Some(unsafe { [<$permit_name:camel Permit>]::new(self.tx.clone()) })
                } else {
                    None
                }
            }
        )*
    }};
}

macro_rules! unchecked_unwraps {
    ($for_ty:ident => $($variant:ident : $varty:ty),* $(,)?) => {paste::paste! {
        impl $for_ty {
            $(
                #[cfg(feature = "" [<$variant:lower>] "")]
                /// # Safety
                ///
                /// Safety is ensured by the caller. Unsafe due to ability of unwrapping wrong variant
                pub unsafe fn [<unwrap_ $variant:lower _unchecked>](&mut self) -> &mut $varty {
                    match self {
                        Self::$variant(ref mut v) => v,
                        #[allow(unreachable_patterns)]
                        _ => std::hint::unreachable_unchecked(),
                    }
                }
            )*
        }
    }};
}

macro_rules! chan_permits {
    (@new $enum:ident unsafe) => {
        /// # Safety
        ///
        /// Unsafe due to ability of producing wrong permits
        pub const unsafe fn new(chan: tokio::sync::mpsc::UnboundedSender<$enum>) -> Self {
            Self { chan }
        }
    };

    (@new $enum:ident safe) => {
        pub const fn new(chan: tokio::sync::mpsc::UnboundedSender<$enum>) -> Self {
            Self { chan }
        }
    };

    (@struct $variant:ident $enum:ident) => {paste::paste!{
        #[derive(Clone)]
        pub struct [<$variant:camel Permit>] {
            chan: tokio::sync::mpsc::UnboundedSender<$enum>,
        }
    }};

    (@struct $variant:ident $enum:ident $explicit_name:ident) => {
        #[derive(Clone)]
        pub struct $explicit_name {
            chan: tokio::sync::mpsc::UnboundedSender<$enum>
        }
    };

    ($keyword:ident, $enum:ident::[
        $($variant:ident $($explicit_name:ident)? : $command_ty:ty),*
    ]) => {paste::paste! {
        $(
            cfg_if::cfg_if! {
                if #[cfg(feature = "" [<$variant:lower>] "")] {
                    $crate::decl::chan_permits!(
                        @struct $variant $enum $($explicit_name)?
                    );

                    impl [<$variant Permit>] {
                        $crate::decl::chan_permits!(@new $enum $keyword);

                        pub fn send(&self, command: $command_ty) -> Result<(), crate::error::PermitSendError> {
                            self.chan.send($enum::$variant(command)).map_err(
                                |_| crate::error::PermitSendError::Closed
                            )
                        }
                    }
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
                $variant:ident $(= $tag_expr:expr)?
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
                $variant $(= $tag_expr)?
            ),*
        }

        $crate::decl::config! { $($tail)* }
    };
}

pub(crate) use chan_permits;
pub(crate) use config;
#[allow(unused_imports)]
pub(crate) use continue_;
pub(crate) use permit_issuers;
pub(crate) use unchecked_unwraps;
