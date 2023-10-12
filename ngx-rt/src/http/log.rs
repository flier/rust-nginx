macro_rules! define_http_logger {
    ( $( $name:ident => $level:ident ,)* ) => {
        define_http_logger! { __impl =>
            ($d:tt) => {
                ::paste::paste! {
                    $(
                        #[macro_export]
                        macro_rules! [< http_ $name >] {
                            ($d log:expr, $d( $d args:tt )*) => {
                                ::std::convert::AsRef::<$d crate::core::LogRef>::as_ref($d log)
                                    .http().core($crate::core::LogLevel::$level, format!($d ($d args)*))
                            };
                        }
                    )*
                }
            }
        }
    };
    ( __impl => $($body:tt)* ) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

define_http_logger! {
    stderr => StdErr,
    emerg => Emerg,
    alert => Alert,
    critical => Critical,
    error => Error,
    warn => Warn,
    notice => Notice,
    info => Info,
    debug => Debug,
}
