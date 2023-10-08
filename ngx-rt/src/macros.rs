#[macro_export]
macro_rules! property {
    ($name:ident () : bool) => {
        #[inline(always)]
        pub fn $name(&self) -> bool {
            unsafe { $crate::AsRawRef::as_raw(self).$name() != 0 }
        }
    };
    ($name:ident : bool) => {
        #[inline(always)]
        pub fn $name(&self) -> bool {
            unsafe { $crate::AsRawRef::as_raw(self).$name != 0 }
        }
    };
    ($name:ident : Str) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<$crate::core::Str> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                $crate::core::Str::from_raw(p)
            }
        }
    };
    ($name:ident as Header) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<$crate::http::Header> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                <$crate::core::hash::TableEltRef as $crate::FromRawRef>::from_raw(p).map(From::from)
            }
        }
    };
    ($name:ident : Headers) => {
        #[inline(always)]
        pub fn $name(&self) -> $crate::http::Headers {
            $crate::http::Headers(
                unsafe {
                    let p = & $crate::AsRawRef::as_raw(self).$name as *const _ as *mut _;

                    $crate::core::ListRef::from_ptr(p)
                }
                .into_iter(),
            )
        }
    };
    ($name:ident as & $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<&$ty> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                <$ty as $crate::FromRawRef>::from_raw(p)
            }
        }
    };
    ($name:ident as &mut $ty:ty) => {
        property!($name as & $ty);

        ::paste::paste! {
            #[inline(always)]
            pub fn [< $name _mut >](&mut self) -> Option<&mut $ty> {
                unsafe {
                    let p = $crate::AsRawMut::as_raw_mut(self).$name;

                    <$ty as $crate::FromRawMut>::from_raw_mut(p)
                }
            }
        }
    };
    ($name:ident () as $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<$ty> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name();

                <$ty>::from_raw(p)
            }
        }
    };
    ($name:ident as $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> Option<$ty> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                <$ty as $crate::FromRawRef>::from_raw(p)
            }
        }
    };
    (& $name:ident : & $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> & $ty {
            unsafe {
                let p = & $crate::AsRawRef::as_raw(self).$name as *const _ as *mut _;

                <$ty as ::foreign_types::ForeignTypeRef>::from_ptr(p)
            }
        }
    };
    (&mut $name:ident : &mut $ty:ty) => {
        property!(& $name : & $ty);

        ::paste::paste! {
            #[inline(always)]
            pub fn [< $name _mut >](&mut self) -> &mut $ty {
                unsafe {
                    let p = &mut $crate::AsRawMut::as_raw_mut(self).$name as *mut _;

                    <$ty as ::foreign_types::ForeignTypeRef>::from_ptr_mut(p)
                }
            }
        }
    };
    ($name:ident : & $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> &$ty {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name as *const _ as *mut _;

                <$ty as ::foreign_types::ForeignTypeRef>::from_ptr(p)
            }
        }
    };
    ($name:ident : &mut $ty:ty) => {
        property!($name : & $ty);

        ::paste::paste! {
            #[inline(always)]
            pub fn [< $name _mut >](&mut self) -> &mut $ty {
                unsafe {
                    let p = $crate::AsRawMut::as_raw_mut(self).$name as *mut _;

                    <$ty as ::foreign_types::ForeignTypeRef>::from_ptr_mut(p)
                }
            }
        }
    };
    ($name:ident : $ty:ty) => {
        #[inline(always)]
        pub fn $name(&self) -> $ty {
            unsafe { $crate::AsRawRef::as_raw(self).$name }
        }
    };
}

#[macro_export]
macro_rules! flag {
    ($name:ident ()) => {
        $crate::property!($name (): bool);
    };
}

#[macro_export]
macro_rules! str {
    ($name:ident) => {
        $crate::property!($name: Str);
    };
}
