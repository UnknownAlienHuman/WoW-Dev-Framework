//! Host-only execution policy. These flags never reach the source loader/guest.
use std::ffi::OsString;
use wow_literal_host::Limits;

pub fn parse(args: &[OsString]) -> Result<(Limits, usize), Box<dyn std::error::Error>> {
    let mut limits = Limits::default();
    let (mut fuel, mut memory) = (false, false);
    let mut next = 0;
    while let Some(flag) = args.get(next).and_then(|arg| arg.to_str()) {
        match flag {
            "--fuel" if !fuel => {
                limits.fuel = decimal(args.get(next + 1))?.parse()?;
                fuel = true;
            }
            "--memory-bytes" if !memory => {
                limits.memory_bytes = decimal(args.get(next + 1))?.parse()?;
                memory = true;
            }
            _ if flag.starts_with("--") => {
                return Err("unknown or duplicate host limit flag".into());
            }
            _ => break,
        }
        next += 2;
    }
    limits.validate()?;
    Ok((limits, next))
}
fn decimal(value: Option<&OsString>) -> Result<&str, Box<dyn std::error::Error>> {
    let text = value
        .and_then(|arg| arg.to_str())
        .ok_or("missing host limit value")?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("host limit must be an unsigned decimal integer".into());
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(Into::into).collect()
    }
    #[test]
    fn explicit_bounds_and_source_arguments_are_separate() -> Result<(), Box<dyn std::error::Error>>
    {
        let input = args(&[
            "--fuel",
            "200000000",
            "--memory-bytes",
            "67108864",
            "checkout",
            "HEAD",
        ]);
        let (limits, used) = parse(&input)?;
        assert_eq!(used, 4);
        assert_eq!(limits.fuel, 200_000_000);
        assert_eq!(limits.memory_bytes, 64 * 1024 * 1024);
        assert_eq!(input[used], "checkout");
        assert_eq!(parse(&args(&["checkout"]))?, (Limits::default(), 0));
        Ok(())
    }
    #[test]
    fn malformed_duplicate_and_excessive_limits_are_rejected() {
        for input in [
            vec!["--fuel"],
            vec!["--fuel", "0"],
            vec!["--fuel", "500000001"],
            vec!["--fuel", "-1"],
            vec!["--fuel", "+1"],
            vec!["--fuel", "1.0"],
            vec!["--fuel", "18446744073709551616"],
            vec!["--fuel", "1", "--fuel", "2"],
            vec!["--memory-bytes", "0"],
            vec!["--memory-bytes", "134217729"],
            vec!["--memory-bytes", "1", "--memory-bytes", "2"],
            vec!["--unknown", "1"],
        ] {
            assert!(parse(&args(&input)).is_err(), "{input:?}");
        }
    }
}
