pub use galaxy_net_raw as raw;
pub use galaxy_shrinker as shrinker;

pub mod reader;
pub mod writer;

pub mod error;
pub mod utils;

pub mod schemas;

#[macro_export]
#[doc(hidden)]
macro_rules! __raw_impl {
    (@method<$ty:ty>{ $immutable:ident, $mutable:ident } $($path:tt)*) => {
        #[doc = concat!(
            "Get immutable reference to the underlying ",
            stringify!($immutable)
        )]
        pub fn $immutable(&self) -> &$ty {
            &self.$($path)*
        }

        #[doc = concat!(
            "Get mutable reference to the underlying ",
            stringify!($immutable)
        )]
        pub fn $mutable(&mut self) -> &mut $ty {
            &mut self.$($path)*
        }
    };

    (@stream<$ty:ty> $($path:tt)*) => {
        $crate::__raw_impl! { @method<$ty>{ stream, stream_mut } $($path)* }
    };

    (@compressor<$ty:ty> $($path:tt)*) => {
        $crate::__raw_impl! { @method<$ty>{ compressor, compressor_mut } $($path)* }
    };

    (@decompressor<$ty:ty> $($path:tt)*) => {
        $crate::__raw_impl! { @method<$ty>{ decompressor, decompressor_mut } $($path)* }
    };
}
