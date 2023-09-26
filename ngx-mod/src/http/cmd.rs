use bitflags::bitflags;

use crate::ffi;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Type: u32 {
        /// In the http block.
        const MAIN_CONF = ffi::NGX_HTTP_MAIN_CONF;
        /// In a server block within the http block.
        const SRV_CONF = ffi::NGX_HTTP_SRV_CONF;
        /// In a location block within the http block.
        const LOC_CONF = ffi::NGX_HTTP_LOC_CONF;
        /// In an upstream block within the http block.
        const UPS_CONF = ffi::NGX_HTTP_UPS_CONF;
        /// In an if block within a server block in the http block.
        const SIF_CONF = ffi::NGX_HTTP_SIF_CONF;
        /// In an if block within a location block in the http block.
        const LIF_CONF = ffi::NGX_HTTP_LIF_CONF;
        /// In a limit_except block within the http block.
        const LMT_CONF = ffi::NGX_HTTP_LMT_CONF;
    }
}
