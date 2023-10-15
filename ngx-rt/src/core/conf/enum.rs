use std::ptr::NonNull;

use crate::ffi;

#[macro_export]
macro_rules! ngx_enum_values {
    (
        $vis:vis enum $name:ident {
            $( $key:literal => $value:expr ),*
        }
    ) => {
        $vis const $name: [ $crate::ffi::ngx_conf_enum_t; $crate::ngx_enum_values!( __count $( $key )* ) + 1usize ] = [
            $(
                $crate::ngx_enum_values!(__value $crate::ngx_str!( $key ), $value),
            )*
            $crate::ngx_enum_values!(__value $crate::ngx_str!(), 0),
        ];
    };
    ( __value $key:expr, $value:expr ) => {
        $crate::ffi::ngx_conf_enum_t {
            name: $key,
            value: $value as usize,
        }
    };
    ( __count $($tts:tt)* ) => {
        0usize $(+ $crate::ngx_enum_values!( __replace_expr $tts 1usize ) )*
    };
    ( __replace_expr $_t:tt $sub:expr ) => {
        $sub
    }
}

pub const fn values<const N: usize>(
    values: &[ffi::ngx_conf_enum_t; N],
) -> NonNull<ffi::ngx_conf_enum_t> {
    unsafe { NonNull::new_unchecked(values.as_slice().as_ptr() as *mut _) }
}
