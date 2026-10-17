use super::Incoming;
use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::{fd::AsFd, unix::fs::FileTypeExt},
};

pub(super) struct Pipes {
    input: File,
    output: File,
    restore: Option<[OFlags; 2]>,
}
impl Pipes {
    pub(super) fn acquire() -> io::Result<Self> {
        let input = File::from(io::stdin().as_fd().try_clone_to_owned()?);
        let output = File::from(io::stdout().as_fd().try_clone_to_owned()?);
        Self::from_files(input, output)
    }
    fn from_files(input: File, output: File) -> io::Result<Self> {
        if !input.metadata()?.file_type().is_fifo() || !output.metadata()?.file_type().is_fifo() {
            return Err(io::Error::other("owned stdio requires pipes"));
        }
        let original = [fcntl_getfl(&input)?, fcntl_getfl(&output)?];
        let input_access = original[0] & OFlags::ACCMODE;
        let output_access = original[1] & OFlags::ACCMODE;
        if (input_access != OFlags::RDONLY && input_access != OFlags::RDWR)
            || (output_access != OFlags::WRONLY && output_access != OFlags::RDWR)
        {
            return Err(io::Error::other(
                "owned stdio requires readable input and writable output",
            ));
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        if original.iter().any(|flags| flags.contains(OFlags::PATH)) {
            return Err(io::Error::other(
                "owned stdio refuses path-only descriptors",
            ));
        }
        let mut pipes = Self {
            input,
            output,
            restore: Some(original),
        };
        let configured = fcntl_setfl(&pipes.input, original[0] | OFlags::NONBLOCK)
            .and_then(|()| fcntl_setfl(&pipes.output, original[1] | OFlags::NONBLOCK));
        if let Err(error) = configured {
            pipes.close()?;
            return Err(error.into());
        }
        Ok(pipes)
    }
    pub(super) fn read(&mut self, buffer: &mut [u8]) -> io::Result<Incoming> {
        match self.input.read(buffer) {
            Ok(0) => Ok(Incoming::Eof),
            Ok(bytes) => Ok(Incoming::Bytes(bytes)),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(Incoming::Pending)
            }
            Err(error) => Err(error),
        }
    }
    pub(super) fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self.output.write(buffer) {
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(0)
            }
            result => result,
        }
    }
    pub(super) fn close(&mut self) -> io::Result<()> {
        if let Some(flags) = self.restore.take() {
            let input = fcntl_setfl(&self.input, flags[0]);
            let output = fcntl_setfl(&self.output, flags[1]);
            input?;
            output?;
        }
        Ok(())
    }
}
impl Drop for Pipes {
    fn drop(&mut self) {
        if let Some(flags) = self.restore.take() {
            let _ = fcntl_setfl(&self.input, flags[0]);
            let _ = fcntl_setfl(&self.output, flags[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::OwnedFd;
    fn pair() -> io::Result<(File, File)> {
        let (read, write) = io::pipe()?;
        Ok((
            File::from(OwnedFd::from(read)),
            File::from(OwnedFd::from(write)),
        ))
    }
    #[test]
    fn actual_pipe_pending_data_eof_output_and_original_flags() -> io::Result<()> {
        let (input, mut peer_input) = pair()?;
        let (mut peer_output, output) = pair()?;
        let input_observer = input.try_clone()?;
        let output_observer = output.try_clone()?;
        let before = [
            fcntl_getfl(&input_observer)?,
            fcntl_getfl(&output_observer)?,
        ];
        let mut pipes = Pipes::from_files(input, output)?;
        assert!(fcntl_getfl(&input_observer)?.contains(OFlags::NONBLOCK));
        assert!(fcntl_getfl(&output_observer)?.contains(OFlags::NONBLOCK));
        let mut bytes = [0; 4];
        assert!(matches!(pipes.read(&mut bytes)?, Incoming::Pending));
        peer_input.write_all(b"data")?;
        assert!(matches!(pipes.read(&mut bytes)?, Incoming::Bytes(4)));
        assert_eq!(&bytes, b"data");
        drop(peer_input);
        assert!(matches!(pipes.read(&mut bytes)?, Incoming::Eof));
        assert_eq!(pipes.write(b"out")?, 3);
        peer_output.read_exact(&mut bytes[..3])?;
        assert_eq!(&bytes[..3], b"out");
        pipes.close()?;
        assert_eq!(fcntl_getfl(&input_observer)?, before[0]);
        assert_eq!(fcntl_getfl(&output_observer)?, before[1]);
        Ok(())
    }
    #[test]
    fn wrong_pipe_directions_refuse_before_mutating_either_descriptor() -> io::Result<()> {
        let (_input_reader, input_writer) = pair()?;
        let (output_reader, output_writer) = pair()?;
        let before_input = fcntl_getfl(&input_writer)?;
        let before_output = fcntl_getfl(&output_writer)?;
        assert!(Pipes::from_files(input_writer.try_clone()?, output_writer.try_clone()?).is_err());
        assert_eq!(fcntl_getfl(&input_writer)?, before_input);
        assert_eq!(fcntl_getfl(&output_writer)?, before_output);
        let (input_reader, _input_writer) = pair()?;
        let before_input = fcntl_getfl(&input_reader)?;
        let before_output = fcntl_getfl(&output_reader)?;
        assert!(Pipes::from_files(input_reader.try_clone()?, output_reader.try_clone()?).is_err());
        assert_eq!(fcntl_getfl(&input_reader)?, before_input);
        assert_eq!(fcntl_getfl(&output_reader)?, before_output);
        Ok(())
    }
    #[test]
    fn unwind_restores_preexisting_nonblocking_and_other_flags() -> io::Result<()> {
        let (input, _peer_input) = pair()?;
        let (_peer_output, output) = pair()?;
        fcntl_setfl(&output, fcntl_getfl(&output)? | OFlags::NONBLOCK)?;
        let input_observer = input.try_clone()?;
        let output_observer = output.try_clone()?;
        let before = [
            fcntl_getfl(&input_observer)?,
            fcntl_getfl(&output_observer)?,
        ];
        let pipes = Pipes::from_files(input, output)?;
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _pipes = pipes;
            std::panic::resume_unwind(Box::new("owned Unix pipe conformance panic"));
        }));
        assert!(panic.is_err());
        assert_eq!(fcntl_getfl(&input_observer)?, before[0]);
        assert_eq!(fcntl_getfl(&output_observer)?, before[1]);
        Ok(())
    }
}
