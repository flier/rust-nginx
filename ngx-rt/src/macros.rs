#[doc(hidden)]
#[macro_export]
macro_rules! property {
    () => {};

    ($(#[$attr:meta])* $name:ident : bool ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> bool {
            unsafe { $crate::AsRawRef::as_raw(self).$name() != 0 }
        }
    };
    ($(#[$attr:meta])* $name:ident : bool ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name : bool );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident : bool ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> bool {
            unsafe { $crate::AsRawRef::as_raw(self).$name != 0 }
        }
    };
    ($(#[$attr:meta])* $name:ident : bool ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name : bool );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident : Str ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> Option<$crate::core::Str> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                $crate::core::Str::from_raw(p)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident : Str ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name : Str );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident as Header ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> Option<$crate::http::Header> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                <$crate::core::hash::TableEltRef as $crate::FromRawRef>::from_raw(p).map(From::from)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident as Header ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name as Header );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident : Headers ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> $crate::http::Headers {
            $crate::http::Headers::from(
                unsafe {
                    let p = & $crate::AsRawRef::as_raw(self).$name as *const _ as *mut _;

                    ::foreign_types::ForeignTypeRef::from_ptr_mut(p)
                }
            )
        }
    };

    ($(#[$attr:meta])* $name:ident : Headers ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name : Headers );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident as & $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> Option<&$ty> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                <$ty as $crate::FromRawRef>::from_raw(p)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident as & $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* $name as & $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident as &mut $ty:ty ) => {
        $crate::property!($(#[$attr])* $name as & $ty);

        ::paste::paste! {
            $(#[$attr])*
            #[inline(always)]
            pub fn [< $name _mut >](&mut self) -> Option<&mut $ty> {
                unsafe {
                    let p = $crate::AsRawMut::as_raw_mut(self).$name;

                    <$ty as $crate::FromRawMut>::from_raw_mut(p)
                }
            }
        }
    };
    ($(#[$attr:meta])* $name:ident as &mut $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* $name as &mut $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident () as $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> Option<$ty> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name();

                <$ty>::from_raw(p)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident () as $ty:ty ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name () as $ty );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident as $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> Option<$ty> {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name;

                <$ty as $crate::FromRawRef>::from_raw(p)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident as $ty:ty ; $($rest:tt)* ) => {
        $crate::property!( $(#[$attr])* $name as $ty );
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* & $name:ident : & $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> & $ty {
            unsafe {
                let p = & $crate::AsRawRef::as_raw(self).$name as *const _ as *mut _;

                <$ty as ::foreign_types::ForeignTypeRef>::from_ptr(p)
            }
        }
    };
    ($(#[$attr:meta])* & $name:ident : & $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* & $name : & $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* &mut $name:ident : &mut $ty:ty ) => {
        $crate::property!($(#[$attr])* & $name : & $ty);

        ::paste::paste! {
            $(#[$attr])*
            #[inline(always)]
            pub fn [< $name _mut >](&mut self) -> &mut $ty {
                unsafe {
                    let p = &mut $crate::AsRawMut::as_raw_mut(self).$name as *mut _;

                    <$ty as ::foreign_types::ForeignTypeRef>::from_ptr_mut(p)
                }
            }
        }
    };
    ($(#[$attr:meta])* &mut $name:ident : &mut $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* &mut $name : &mut $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident : & $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> &$ty {
            unsafe {
                let p = $crate::AsRawRef::as_raw(self).$name as *const _ as *mut _;

                <$ty as ::foreign_types::ForeignTypeRef>::from_ptr(p)
            }
        }
    };
    ($(#[$attr:meta])* $name:ident : & $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* $name : & $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident : &mut $ty:ty ) => {
        $crate::property!($(#[$attr])* $name : & $ty);

        ::paste::paste! {
            $(#[$attr])*
            #[inline(always)]
            pub fn [< $name _mut >](&mut self) -> &mut $ty {
                unsafe {
                    let p = $crate::AsRawMut::as_raw_mut(self).$name as *mut _;

                    <$ty as ::foreign_types::ForeignTypeRef>::from_ptr_mut(p)
                }
            }
        }
    };
    ($(#[$attr:meta])* $name:ident : &mut $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* $name : &mut $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident () : $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> $ty {
            unsafe { $crate::AsRawRef::as_raw(self).$name() }
        }
    };
    ($(#[$attr:meta])* $name:ident () : $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* $name () : $ty);
        $crate::property!( $($rest)* );
    };

    ($(#[$attr:meta])* $name:ident : $ty:ty ) => {
        $(#[$attr])*
        #[inline(always)]
        pub fn $name(&self) -> $ty {
            unsafe { $crate::AsRawRef::as_raw(self).$name }
        }
    };
    ($(#[$attr:meta])* $name:ident : $ty:ty ; $($rest:tt)* ) => {
        $crate::property!($(#[$attr])* $name : $ty);
        $crate::property!( $($rest)* );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! flag {
    () => {};

    ($(#[$attr:meta])* $name:ident) => {
        $crate::property!($(#[$attr])* $name : bool);
    };

    ($(#[$attr:meta])* $name:ident ; $($rest:tt)* ) => {
        $crate::flag!($(#[$attr])* $name);
        $crate::flag!( $($rest)* );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! str {
    () => {};

    ($(#[$attr:meta])* $name:ident ) => {
        $crate::property!($(#[$attr])* $name: Str);
    };

    ($(#[$attr:meta])* $name:ident ; $($rest:tt)* ) => {
        $crate::str!($(#[$attr])* $name);
        $crate::str!( $($rest)* );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! header {
    () => {};

    ($(#[$attr:meta])* $name:ident) => {
        $crate::property!($(#[$attr])* $name as Header);
    };

    ($(#[$attr:meta])* $name:ident ; $($rest:tt)* ) => {
        $crate::header!($(#[$attr])* $name);
        $crate::header!( $($rest)* );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! callback {
    () => {};

    ($(#[$attr:meta])* $name:ident : $ty:tt ) => {
        pub fn $name(&self) -> Option<$ty> {
            unsafe { $crate::AsRawRef::as_raw(self).$name.map( $ty ) }
        }
    };

    ($(#[$attr:meta])* $name:ident : $ty:tt ; $($rest:tt)* ) => {
        $crate::callback!($(#[$attr])* $name : $ty );
        $crate::callback!( $($rest)* );
    };
}
