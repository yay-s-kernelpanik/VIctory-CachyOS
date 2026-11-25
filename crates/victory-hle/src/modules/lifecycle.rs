//! The title's life cycle. A console title is never told it is closing:
//! quitting it or launching a new game first suspends it (its suspend
//! routines run, and it has a second to save) and then ends the process.
//! Closing the title's window does the same here; a new launch then starts
//! `victory` again, which waits for this process to be gone.
#![allow(non_snake_case)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::too_many_arguments)]



use crate::HleModule;


#[cfg(not(windows))]
pub(crate) fn end_title(_why: &'static str, _then: Box<dyn FnOnce() + Send>) {
    _then();
    std::process::exit(0);
}

#[cfg(not(windows))]
pub(crate) fn relaunch(_args: Vec<std::ffi::OsString>) {
    tracing::info!("relaunch not implemented on Linux");
}

#[cfg(not(windows))]
pub fn wait_for_previous() {
    // Linux stub: no-op
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub(crate) fn module() -> HleModule {
    HleModule {
        name: "lifecycle",
        dlls: &[],
        exports: vec![],
    }
}