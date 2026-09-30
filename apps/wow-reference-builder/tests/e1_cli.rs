#[test]
fn missing_build_arguments_are_usage_errors() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        wow_reference_builder::run(
            ["build"].into_iter().map(str::to_owned),
            &mut stdout,
            &mut stderr,
        ),
        2
    );
    assert!(stdout.is_empty());
    assert!(String::from_utf8_lossy(&stderr).contains("missing"));
}

#[test]
fn duplicate_and_unknown_options_fail_before_io() {
    for arguments in [
        vec!["validate", "--pack", "a", "--pack", "b"],
        vec!["rebuild-compare", "--unknown", "x"],
        vec!["build", "--json", "--json"],
    ] {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = wow_reference_builder::run(
            arguments.into_iter().map(str::to_owned),
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit, 2);
        assert!(stdout.is_empty());
    }
}
