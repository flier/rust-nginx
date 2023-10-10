use foreign_types::ForeignTypeRef;
use ngx_rt::core::{Cmds, Code, ModuleRef};

use crate::rt::{
    core::{CycleRef, LogRef},
    ffi,
};

pub const UNSET_INDEX: ffi::ngx_uint_t = ffi::ngx_uint_t::MAX;

pub trait UnsafeModule {
    /// Initialize the master process.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn init_master(log: *mut ffi::ngx_log_t) -> ffi::ngx_int_t;

    /// Initialize the module.
    ///
    /// This happens prior to the master process forking.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn init_module(cycle: *mut ffi::ngx_cycle_t) -> ffi::ngx_int_t;

    /// Initialize the process.
    ///
    /// This happens as the worker processes are forked.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn init_process(cycle: *mut ffi::ngx_cycle_t) -> ffi::ngx_int_t;

    /// Initialize the thread.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn init_thread(cycle: *mut ffi::ngx_cycle_t) -> ffi::ngx_int_t;

    /// Terminated the thread.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn exit_thread(cycle: *mut ffi::ngx_cycle_t);

    /// Terminated the child process.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn exit_process(cycle: *mut ffi::ngx_cycle_t);

    /// Terminated the master process.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it dereferences raw pointers.
    unsafe extern "C" fn exit_master(cycle: *mut ffi::ngx_cycle_t);
}

impl<T: Module + Sized> UnsafeModule for T {
    unsafe extern "C" fn init_master(log: *mut ffi::ngx_log_t) -> ffi::ngx_int_t {
        <T as Module>::init_master(LogRef::from_ptr(log))
            .err()
            .unwrap_or(Code::OK)
            .into()
    }

    unsafe extern "C" fn init_module(cycle: *mut ffi::ngx_cycle_t) -> ffi::ngx_int_t {
        <T as Module>::init_module(CycleRef::from_ptr(cycle))
            .err()
            .unwrap_or(Code::OK)
            .into()
    }

    unsafe extern "C" fn init_process(cycle: *mut ffi::ngx_cycle_t) -> ffi::ngx_int_t {
        <T as Module>::init_process(CycleRef::from_ptr(cycle))
            .err()
            .unwrap_or(Code::OK)
            .into()
    }

    unsafe extern "C" fn init_thread(cycle: *mut ffi::ngx_cycle_t) -> ffi::ngx_int_t {
        <T as Module>::init_thread(CycleRef::from_ptr(cycle))
            .err()
            .unwrap_or(Code::OK)
            .into()
    }

    unsafe extern "C" fn exit_thread(cycle: *mut ffi::ngx_cycle_t) {
        <T as Module>::exit_thread(CycleRef::from_ptr(cycle))
    }

    unsafe extern "C" fn exit_process(cycle: *mut ffi::ngx_cycle_t) {
        <T as Module>::exit_process(CycleRef::from_ptr(cycle))
    }

    unsafe extern "C" fn exit_master(cycle: *mut ffi::ngx_cycle_t) {
        <T as Module>::exit_master(CycleRef::from_ptr(cycle))
    }
}

pub trait Module: ModuleMetadata {
    /// Initialize the master process.
    fn init_master(_: &LogRef) -> Result<(), Code> {
        Ok(())
    }

    /// Initialize the module.
    fn init_module(_: &CycleRef) -> Result<(), Code> {
        Ok(())
    }

    /// Initialize the process.
    fn init_process(_: &CycleRef) -> Result<(), Code> {
        Ok(())
    }

    /// Initialize the thread.
    fn init_thread(_: &CycleRef) -> Result<(), Code> {
        Ok(())
    }

    /// Terminated the thread.
    fn exit_thread(_: &CycleRef) {}

    /// Terminated the child process.
    fn exit_process(_: &CycleRef) {}

    /// Terminated the master process.
    fn exit_master(_: &CycleRef) {}
}

pub trait ModuleMetadata {
    fn module() -> &'static ModuleRef;

    fn commands() -> Cmds<'static>;
}
