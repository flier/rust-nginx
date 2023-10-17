use std::{
    alloc::Layout,
    mem,
    ptr::{self, NonNull},
};

use foreign_types::{foreign_type, ForeignTypeRef};
use num_enum::FromPrimitive;

use crate::{
    core::{ArrayRef, Str},
    native_callback, never_drop, AsRawMut, AsRawRef,
};

use super::{ComplexValueRef, EngineRef};

#[native_callback]
pub type CodeFn = fn(e: &EngineRef);

// #[native_callback]
// pub type LenCodeFn = fn(e: &EngineRef) -> isize;

foreign_type! {
    pub unsafe type CopyCode: Send {
        type CType = ffi::ngx_http_script_copy_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_copy_code_t>;
    }
}

impl CopyCodeRef {
    callback! {
        code: CodeFn;
    }
    property! {
        #[allow(clippy::len_without_is_empty)]
        len: usize { get; set; };
    }
}

foreign_type! {
    pub unsafe type VarCode: Send {
        type CType = ffi::ngx_http_script_var_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_var_code_t>;
    }
}

impl VarCodeRef {
    callback! {
        code: CodeFn;
    }
    property! {
        index: usize { get; set; };
    }
}

foreign_type! {
    pub unsafe type VarHandlerCode: Send {
        type CType = ffi::ngx_http_script_var_handler_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_var_handler_code_t>;
    }
}

impl VarHandlerCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        handler: ffi::ngx_http_set_variable_pt { get; set; };
        data: usize { get; set; };
    }
}

foreign_type! {
    pub unsafe type CopyCaptureCode: Send {
        type CType = ffi::ngx_http_script_copy_capture_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_copy_capture_code_t>;
    }
}

impl CopyCaptureCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        n: usize { get; set; };
    }
}

foreign_type! {
    pub unsafe type RegexCode: Send {
        type CType = ffi::ngx_http_script_regex_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_regex_code_t>;
    }
}

impl RegexCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        // regex: &RegexRef;
        size: usize;
        status: usize;
        next: usize;
    }

    flag! {
        test;
        negative_test;
        uri;
        args;
        /// add the r->args to the new arguments
        add_args;
        redirect;
        break_cycle;
    }

    str! {
        name;
    }
}

foreign_type! {
    pub unsafe type RegexEndCode: Send {
        type CType = ffi::ngx_http_script_regex_end_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_regex_end_code_t>;
    }
}

impl RegexEndCodeRef {
    callback! {
        code: CodeFn;
    }

    flag! {
        uri;
        args;
        /// add the r->args to the new arguments
        add_args;
        redirect;
    }
}

foreign_type! {
    pub unsafe type FullNameCode: Send {
        type CType = ffi::ngx_http_script_full_name_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_full_name_code_t>;
    }
}

impl FullNameCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        conf_prefix: usize { get; set; };
    }
}

foreign_type! {
    pub unsafe type ReturnCode: Send {
        type CType = ffi::ngx_http_script_return_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_return_code_t>;
    }
}

impl ReturnCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        status: usize { get; set; };
        &mut text: &mut ComplexValueRef;
    }
}

foreign_type! {
    pub unsafe type FileCode: Send {
        type CType = ffi::ngx_http_script_file_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_file_code_t>;
    }
}

impl FileCodeRef {
    callback! {
        code: CodeFn;
    }

    pub fn op(&self) -> FileOp {
        FileOp::from(unsafe { self.as_raw().op as u32 })
    }

    pub fn set_op(&mut self, op: FileOp) {
        unsafe { self.as_raw_mut().op = op as u32 as usize };
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, FromPrimitive)]
pub enum FileOp {
    #[default]
    Plain = ffi::ngx_http_script_file_op_e_ngx_http_script_file_plain,
    NotPlain = ffi::ngx_http_script_file_op_e_ngx_http_script_file_not_plain,
    Dir = ffi::ngx_http_script_file_op_e_ngx_http_script_file_dir,
    NotDir = ffi::ngx_http_script_file_op_e_ngx_http_script_file_not_dir,
    Exists = ffi::ngx_http_script_file_op_e_ngx_http_script_file_exists,
    NotExists = ffi::ngx_http_script_file_op_e_ngx_http_script_file_not_exists,
    Exec = ffi::ngx_http_script_file_op_e_ngx_http_script_file_exec,
    NotExec = ffi::ngx_http_script_file_op_e_ngx_http_script_file_not_exec,
}

foreign_type! {
    pub unsafe type IfCode: Send {
        type CType = ffi::ngx_http_script_if_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_if_code_t>;
    }
}

impl IfCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        next: usize;
    }

    pub fn loc_conf<T>(&self) -> Option<NonNull<*mut T>> {
        unsafe { NonNull::new(self.as_raw().loc_conf.cast()) }
    }

    pub fn set_loc_conf<T>(&mut self, loc_conf: Option<NonNull<*mut T>>) {
        unsafe {
            self.as_raw_mut().loc_conf = loc_conf.map_or_else(ptr::null_mut, |p| p.as_ptr().cast())
        }
    }
}

foreign_type! {
    pub unsafe type ComplexValueCode: Send {
        type CType = ffi::ngx_http_script_complex_value_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_complex_value_code_t>;
    }
}

impl ComplexValueCodeRef {
    callback! {
        code: CodeFn;
    }
}

foreign_type! {
    pub unsafe type ValueCode: Send {
        type CType = ffi::ngx_http_script_value_code_t;

        fn drop = never_drop::<ffi::ngx_http_script_value_code_t>;
    }
}

impl ValueCodeRef {
    callback! {
        code: CodeFn;
    }

    property! {
        value: usize { get; set; };
    }

    pub fn text(&self) -> Option<Str> {
        unsafe {
            let r = self.as_raw();

            NonNull::new(r.text_data as *mut _).map(|p| Str::unchecked_new(p, r.text_len))
        }
    }
}

pub fn copy_len_code(buf: &mut ArrayRef<u8>, len: usize) -> Option<&mut CopyCodeRef> {
    start_code::<CopyCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = mem::transmute::<
            ffi::ngx_http_script_len_code_pt,
            ffi::ngx_http_script_code_pt,
        >(Some(ffi::ngx_http_script_copy_len_code));
        code.as_raw_mut().len = len;
        code
    })
}

pub fn copy_code<'a>(buf: &'a mut ArrayRef<u8>, s: &Str) -> Option<&'a mut CopyCodeRef> {
    let size = Layout::from_size_align(
        mem::size_of::<CopyCodeRef>() + s.len(),
        mem::size_of::<usize>(),
    )
    .expect("layout")
    .pad_to_align()
    .size();

    start_code_buf(buf, size).and_then(|s| unsafe {
        s.as_mut_ptr().cast::<CopyCodeRef>().as_mut().map(|code| {
            code.as_raw_mut().code = Some(ffi::ngx_http_script_copy_code);

            ptr::copy(
                s.as_ptr(),
                s.as_mut_ptr().add(mem::size_of::<CopyCodeRef>()),
                s.len(),
            );

            code
        })
    })
}

pub fn copy_var_len_code(buf: &mut ArrayRef<u8>, index: usize) -> Option<&mut VarCodeRef> {
    start_code::<VarCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = mem::transmute::<
            ffi::ngx_http_script_len_code_pt,
            ffi::ngx_http_script_code_pt,
        >(Some(ffi::ngx_http_script_copy_var_len_code));
        code.as_raw_mut().index = index;
        code
    })
}

pub fn copy_var_code(buf: &mut ArrayRef<u8>, index: usize) -> Option<&mut VarCodeRef> {
    start_code::<VarCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_copy_var_code);
        code.as_raw_mut().index = index;
        code
    })
}

pub fn copy_capture_len_code(buf: &mut ArrayRef<u8>, n: usize) -> Option<&mut CopyCaptureCodeRef> {
    start_code::<CopyCaptureCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = mem::transmute::<
            ffi::ngx_http_script_len_code_pt,
            ffi::ngx_http_script_code_pt,
        >(Some(ffi::ngx_http_script_copy_capture_len_code));
        code.as_raw_mut().n = n;
        code
    })
}

pub fn copy_capture_code(buf: &mut ArrayRef<u8>, n: usize) -> Option<&mut CopyCaptureCodeRef> {
    start_code::<CopyCaptureCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_copy_capture_code);
        code.as_raw_mut().n = n;
        code
    })
}

pub fn mark_args_code(buf: &mut ArrayRef<u8>) -> Option<()> {
    start_code_type::<ffi::ngx_http_script_len_code_pt>(buf).map(|code| {
        code.replace(ffi::ngx_http_script_mark_args_code);
    })
}

pub fn start_args_code(buf: &mut ArrayRef<u8>) -> Option<()> {
    start_code_type::<ffi::ngx_http_script_code_pt>(buf).map(|code| {
        code.replace(ffi::ngx_http_script_start_args_code);
    })
}

pub fn regex_start_code(buf: &mut ArrayRef<u8>) -> Option<&mut RegexCodeRef> {
    start_code::<RegexCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_regex_start_code);
        code
    })
}

pub fn regex_end_code(buf: &mut ArrayRef<u8>) -> Option<&mut RegexEndCodeRef> {
    start_code::<RegexEndCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_regex_end_code);
        code
    })
}

pub fn return_code(buf: &mut ArrayRef<u8>, status: usize) -> Option<&mut ReturnCodeRef> {
    start_code::<ReturnCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_return_code);
        code.as_raw_mut().status = status;
        code
    })
}

pub fn break_code(buf: &mut ArrayRef<u8>) -> Option<()> {
    start_code_type::<ffi::ngx_http_script_code_pt>(buf).map(|code| {
        code.replace(ffi::ngx_http_script_break_code);
    })
}

pub fn if_code(buf: &mut ArrayRef<u8>) -> Option<&mut IfCodeRef> {
    start_code::<IfCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_if_code);
        code
    })
}

pub fn equal_code(buf: &mut ArrayRef<u8>) -> Option<()> {
    start_code_type::<ffi::ngx_http_script_code_pt>(buf).map(|code| {
        code.replace(ffi::ngx_http_script_equal_code);
    })
}

pub fn not_equal_code(buf: &mut ArrayRef<u8>) -> Option<()> {
    start_code_type::<ffi::ngx_http_script_code_pt>(buf).map(|code| {
        code.replace(ffi::ngx_http_script_not_equal_code);
    })
}

pub fn file_code(buf: &mut ArrayRef<u8>) -> Option<&mut FileCodeRef> {
    start_code::<FileCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_file_code);
        code
    })
}

pub fn complex_value_code(buf: &mut ArrayRef<u8>) -> Option<&mut ComplexValueCodeRef> {
    start_code::<ComplexValueCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_complex_value_code);
        code
    })
}

pub fn value_code(buf: &mut ArrayRef<u8>) -> Option<&mut ValueCodeRef> {
    start_code::<ValueCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_value_code);
        code
    })
}

pub fn set_var_code(buf: &mut ArrayRef<u8>) -> Option<&mut VarCodeRef> {
    start_code::<VarCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_set_var_code);
        code
    })
}

pub fn var_set_handler_code(buf: &mut ArrayRef<u8>) -> Option<&mut VarHandlerCodeRef> {
    start_code::<VarHandlerCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_var_set_handler_code);
        code
    })
}

pub fn var_code(buf: &mut ArrayRef<u8>) -> Option<&mut VarCodeRef> {
    start_code::<VarCodeRef>(buf).map(|code| unsafe {
        code.as_raw_mut().code = Some(ffi::ngx_http_script_var_code);
        code
    })
}

pub fn nop_code(buf: &mut ArrayRef<u8>) -> Option<()> {
    start_code_type::<ffi::ngx_http_script_code_pt>(buf).map(|code| {
        code.replace(ffi::ngx_http_script_nop_code);
    })
}

pub fn start_code<T: ForeignTypeRef>(buf: &mut ArrayRef<u8>) -> Option<&mut T> {
    start_code_type::<T::CType>(buf)
        .map(|r: &mut <T as ForeignTypeRef>::CType| unsafe { T::from_ptr_mut(r as *mut _) })
}

fn start_code_type<T: Sized>(buf: &mut ArrayRef<u8>) -> Option<&mut T> {
    start_code_buf(buf, mem::size_of::<T>())
        .and_then(|s| unsafe { s.as_mut_ptr().cast::<T>().as_mut() })
}

fn start_code_buf(buf: &mut ArrayRef<u8>, size: usize) -> Option<&mut [u8]> {
    buf.reserve_n(size).map(|s| unsafe {
        ptr::write_bytes(s.as_mut_ptr(), 0, size);

        slice_assume_init_mut(s)
    })
}

// FIXME: use `MaybeUninit::slice_assume_init_mut` after it is stabilized.
unsafe fn slice_assume_init_mut<I, O>(slice: &mut [mem::MaybeUninit<I>]) -> &mut [O] {
    // SAFETY: similar to safety notes for `slice_get_ref`, but we have a
    // mutable reference which is also guaranteed to be valid for writes.
    unsafe { &mut *(slice as *mut [mem::MaybeUninit<I>] as *mut [O]) }
}
