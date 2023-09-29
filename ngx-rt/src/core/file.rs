use foreign_types::foreign_type;

use crate::{ffi, never_drop};

foreign_type! {
    pub unsafe type File: Send {
        type CType = ffi::ngx_file_t;

        fn drop = never_drop::<ffi::ngx_file_t>;
    }
}

foreign_type! {
    pub unsafe type Path: Send {
        type CType = ffi::ngx_path_t;

        fn drop = never_drop::<ffi::ngx_path_t>;
    }
}

foreign_type! {
    pub unsafe type TempFile: Send {
        type CType = ffi::ngx_temp_file_t;

        fn drop = never_drop::<ffi::ngx_temp_file_t>;
    }
}

foreign_type! {
    pub unsafe type ExtRenameFile: Send {
        type CType = ffi::ngx_ext_rename_file_t;

        fn drop = never_drop::<ffi::ngx_ext_rename_file_t>;
    }
}

foreign_type! {
    pub unsafe type CopyFile: Send {
        type CType = ffi::ngx_copy_file_t;

        fn drop = never_drop::<ffi::ngx_copy_file_t>;
    }
}
