macro_rules! chan_permits {
    ($chan_ty_ident:ident, $for_e:ident :: [
        $($variant:ident : $inner_ty:ty),*
        $(,)?
    ]) => {
        paste::paste! {
            $(
                #[derive(Clone)]
                pub struct [<$variant Permit>] {
                    raw: $chan_ty_ident<$for_e>
                }

                impl [<$variant Permit>] {
                    /// # Safety
                    ///
                    /// Unsafe due to ability of producing wrong permit type
                    pub const unsafe fn new(raw: $chan_ty_ident<$for_e>) -> Self {
                        Self { raw }
                    }

                    #[inline]
                    pub fn send(
                        &self,
                        command: $inner_ty
                    ) -> Result<(), crate::error::PermitSendError>
                    {
                        if self.raw.send($for_e::$variant(command)).is_err() {
                            Err(crate::error::PermitSendError::Closed)
                        } else {
                            Ok(())
                        }
                    }
                }
            )*
        }
    };
}

macro_rules! define_unchecked_mut_unwraps {
    ($for_e:ident :: [
        $($variant:ident : $inner_ty:ty),*
        $(,)?
    ]) => {paste::paste! {
        impl $for_e {
            $(
                #[doc = concat!(
                    "Unwrap `", stringify!($variant), "` variant without checks.\n\n",
                    "# Safety'\n",
                    "Unsafe due to usafe of unreachable_unchecked on all other arms"
                )]
                pub unsafe fn [<unwrap_ $variant:lower _unchecked>](
                    &mut self
                ) -> &mut $inner_ty {
                    #[allow(unreachable_patterns)]
                    match self {
                        Self::$variant(ref mut variant) => variant,
                        _ => std::hint::unreachable_unchecked(),
                    }
                }
            )*
        }
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
pub(crate) use define_unchecked_mut_unwraps;
