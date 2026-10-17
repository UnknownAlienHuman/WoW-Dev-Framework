//! The only App unsafe exception: four fixed pipe metadata/readiness calls.
#![allow(unsafe_code)]
use std::{fs::File, io, os::windows::io::AsRawHandle, ptr};
use windows_sys::Win32::System::Pipes::{
    GetNamedPipeHandleStateW, GetNamedPipeInfo, PeekNamedPipe, SetNamedPipeHandleState,
};

pub(super) fn kind(file: &File) -> io::Result<u32> {
    let mut flags = 0;
    // SAFETY: the File borrow keeps its handle live. flags is one exclusive
    // initialized stack u32 through this synchronous call; other outputs are null.
    let ok = unsafe {
        GetNamedPipeInfo(
            file.as_raw_handle(),
            &mut flags,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(flags)
    }
}
pub(super) fn mode(file: &File) -> io::Result<u32> {
    let mut mode = 0;
    // SAFETY: the borrowed File owns a live handle, and mode is exclusively
    // writable until return. Optional collection/name outputs are null; size is 0.
    let ok = unsafe {
        GetNamedPipeHandleStateW(
            file.as_raw_handle(),
            &mut mode,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        )
    };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(mode)
    }
}
pub(super) fn set_mode(file: &File, mode: u32) -> io::Result<()> {
    // SAFETY: the File borrow keeps the handle live and mode remains initialized
    // and immutable through this synchronous call. Remote collection inputs are null.
    let ok =
        unsafe { SetNamedPipeHandleState(file.as_raw_handle(), &mode, ptr::null(), ptr::null()) };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
pub(super) fn ready(file: &File) -> io::Result<u32> {
    let mut count = 0;
    // SAFETY: the File handle stays live; count is one exclusive stack output.
    // No source buffer is passed or retained: data/name outputs are null, size 0.
    // The caller owns the sole I/O thread and never has pending I/O on this handle.
    let ok = unsafe {
        PeekNamedPipe(
            file.as_raw_handle(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            &mut count,
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(count)
    }
}
