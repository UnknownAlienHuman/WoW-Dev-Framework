use super::Incoming;
use std::{
    fs::File,
    io::{self, Read, Write},
    os::windows::io::AsHandle,
};
use winx::file::{AccessMode, FileModeInformation};
mod ffi;

pub(super) struct Pipes {
    input: File,
    output: File,
    restore_output_mode: Option<u32>,
}
impl Pipes {
    pub(super) fn acquire() -> io::Result<Self> {
        let input = File::from(io::stdin().as_handle().try_clone_to_owned()?);
        let output = File::from(io::stdout().as_handle().try_clone_to_owned()?);
        // All guards run before changing any caller-owned mode.
        let mut original_modes = [0; 2];
        for (index, file) in [&input, &output].into_iter().enumerate() {
            if ffi::kind(file)? & !1 != 0 {
                return Err(refused("owned stdio requires byte pipes"));
            }
            let mode = ffi::mode(file)?;
            if mode & !1 != 0 {
                return Err(refused("owned stdio requires byte read mode"));
            }
            original_modes[index] = mode;
            let flags = winx::file::query_mode_information(file.as_handle())?;
            if !flags.intersects(
                FileModeInformation::FILE_SYNCHRONOUS_IO_ALERT
                    | FileModeInformation::FILE_SYNCHRONOUS_IO_NONALERT,
            ) {
                return Err(refused("owned stdio requires synchronous handles"));
            }
            let access = winx::file::query_access_information(file.as_handle())?;
            let required = if index == 0 {
                AccessMode::FILE_READ_DATA
            } else {
                AccessMode::FILE_WRITE_DATA
            };
            if !access.contains(required) {
                return Err(refused("owned stdio pipe access refused"));
            }
        }
        let mut pipes = Self {
            input,
            output,
            restore_output_mode: Some(original_modes[1]),
        };
        let configured = (|| {
            ffi::set_mode(&pipes.output, original_modes[1] | 1)?;
            if ffi::mode(&pipes.output)? != original_modes[1] | 1 {
                return Err(refused("owned stdio mode configuration mismatch"));
            }
            Ok(())
        })();
        if let Err(error) = configured {
            // Required restoration failure takes precedence over startup refusal.
            pipes.close()?;
            return Err(error);
        }
        Ok(pipes)
    }
    pub(super) fn read(&mut self, buffer: &mut [u8]) -> io::Result<Incoming> {
        let available = match ffi::ready(&self.input) {
            Ok(bytes) => bytes as usize,
            Err(error) if matches!(error.raw_os_error(), Some(109 | 233)) => {
                return Ok(Incoming::Eof);
            }
            Err(error) => return Err(error),
        };
        if available == 0 {
            return Ok(Incoming::Pending);
        }
        let selected = available.min(buffer.len());
        match self.input.read(&mut buffer[..selected]) {
            Ok(0) => Err(refused("owned stdio readiness changed unexpectedly")),
            Ok(bytes) => Ok(Incoming::Bytes(bytes)),
            Err(error) => Err(error),
        }
    }
    pub(super) fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.output.write(buffer)
    }
    pub(super) fn close(&mut self) -> io::Result<()> {
        if let Some(mode) = self.restore_output_mode.take() {
            ffi::set_mode(&self.output, mode)?;
            if ffi::mode(&self.output)? != mode {
                return Err(refused("owned stdio original mode restore mismatch"));
            }
        }
        Ok(())
    }
}
impl Drop for Pipes {
    fn drop(&mut self) {
        // Best effort only during unwinding. Normal/error shutdown calls close
        // explicitly and never reports graceful success over its failure.
        if let Some(mode) = self.restore_output_mode.take() {
            let _ = ffi::set_mode(&self.output, mode);
        }
    }
}
fn refused(message: &'static str) -> io::Error {
    io::Error::other(message)
}
