#[macro_export]
macro_rules! get {
    ($name:ident () : bool) => {
        #[inline(always)]
        pub fn $name(&self) -> bool {
            unsafe { crate::AsRawRef::as_raw(self).$name() != 0 }
        }
    };
    ($name:ident : bool) => {
        #[inline(always)]
        pub fn $name(&self) -> bool {
            unsafe { crate::AsRawRef::as_raw(self).$name != 0 }
        }
    };
    ($name:ident : Str) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<&$crate::core::Str> {
            unsafe { $crate::core::Str::from_raw(crate::AsRawRef::as_raw(self).$name) }
        }
    };
    ($name:ident : Headers) => {
        #[inline(always)]
        pub fn $name(&self) -> crate::http::Headers {
            crate::http::Headers(
                unsafe {
                    $crate::core::ListRef::from_ptr(
                        &crate::AsRawRef::as_raw(self).$name as *const _ as *mut _,
                    )
                }
                .into_iter(),
            )
        }
    };
    ($name:ident as & $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<&$ty> {
            unsafe { <$ty as $crate::FromRawRef>::from_raw(crate::AsRawRef::as_raw(self).$name) }
        }
    };
    ($name:ident () as $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<$ty> {
            unsafe { <$ty>::from_raw(crate::AsRawRef::as_raw(self).$name()) }
        }
    };
    ($name:ident as $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<$ty> {
            unsafe { <$ty as $crate::FromRawRef>::from_raw(crate::AsRawRef::as_raw(self).$name) }
        }
    };
    ($name:ident : & $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> &$ty {
            unsafe {
                <$ty as ::foreign_types::ForeignTypeRef>::from_ptr(
                    &crate::AsRawRef::as_raw(self).$name as *const _ as *mut _,
                )
            }
        }
    };
    ($name:ident : &mut $ty:ty) => {
        #[inline(always)]
        pub fn $name(&mut self) -> &mut $ty {
            unsafe {
                <$ty as ::foreign_types::ForeignTypeRef>::from_ptr_mut(
                    &crate::AsRawRef::as_raw(self).$name as *const _ as *mut _,
                )
            }
        }
    };
    ($name:ident : $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> $ty {
            unsafe { crate::AsRawRef::as_raw(self).$name }
        }
    };
}

#[macro_export]
macro_rules! flag {
    ($name:ident ()) => {
        crate::get!($name (): bool);
    };
}

#[macro_export]
macro_rules! str {
    ($name:ident) => {
        crate::get!($name: Str);
    };
}
