//! Owned stdio for the explicit foreground execution profile.
//! Acquisition, readiness and all I/O stay on one App thread. No OS handle is
//! passed to the semantic worker. See apps/wow/LSP_IO_SAFETY.md.
use std::io;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;
#[cfg(unix)]
use unix::Pipes;
#[cfg(windows)]
use windows::Pipes;

pub(super) const READ_CHUNK_BYTES: usize = 8192;
pub(super) const WRITE_CHUNK_BYTES: usize = 512;

pub(super) enum Incoming {
    Pending,
    Bytes(usize),
    Eof,
}

pub(super) struct OwnedStdio {
    #[cfg(any(unix, windows))]
    pipes: Pipes,
}
impl OwnedStdio {
    pub(super) fn acquire() -> io::Result<Self> {
        #[cfg(any(unix, windows))]
        {
            Ok(Self {
                pipes: Pipes::acquire()?,
            })
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "owned stdio target unsupported",
            ))
        }
    }
    pub(super) fn read(&mut self, buffer: &mut [u8]) -> io::Result<Incoming> {
        if buffer.is_empty() || buffer.len() > READ_CHUNK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "owned stdio read bound",
            ));
        }
        #[cfg(any(unix, windows))]
        {
            self.pipes.read(buffer)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "owned stdio target unsupported",
            ))
        }
    }
    pub(super) fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() || buffer.len() > WRITE_CHUNK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "owned stdio write bound",
            ));
        }
        #[cfg(any(unix, windows))]
        {
            self.pipes.write(buffer)
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "owned stdio target unsupported",
            ))
        }
    }
    pub(super) fn close(mut self) -> io::Result<()> {
        #[cfg(any(unix, windows))]
        {
            self.pipes.close()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Ok(())
        }
    }
}
