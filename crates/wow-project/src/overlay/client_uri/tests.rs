use super::*;
#[test]
fn local_uri_percent_aliases_preserve_utf8_and_decode_only_once() -> OverlayResult<()> {
    let (prefix, root) = if cfg!(windows) {
        ("file:///c%3A/alpha/", "/C:/alpha/")
    } else {
        ("file:///alpha/", "/alpha/")
    };
    for (encoded, decoded) in [
        ("%E4%B8%AD%20%25.lua", "中 %.lua"),
        ("m%61in.lua", "main.lua"),
        ("literal%252e.lua", "literal%2e.lua"),
        ("x%28y%29.lua", "x(y).lua"),
    ] {
        assert_eq!(
            decode_local_file_uri(&format!("{prefix}{encoded}"))?,
            format!("{root}{decoded}")
        );
    }
    if cfg!(windows) {
        assert_eq!(
            decode_local_file_uri("file:///C:/alpha/main.lua")?,
            decode_local_file_uri("file:///c%3a/alpha/main.lua")?
        );
    }
    Ok(())
}
#[test]
fn untrusted_local_uri_escape_authority_control_and_limits_are_refused() {
    let prefix = if cfg!(windows) {
        "file:///C:/alpha/"
    } else {
        "file:///alpha/"
    };
    for tail in [
        "../x.lua",
        "%2e%2e/x.lua",
        "./x.lua",
        "a//x.lua",
        "a%2Fx.lua",
        "a%5Cx.lua",
        "x.lua?query",
        "x.lua#fragment",
        "x.lua%00",
        "%FF.lua",
        "%C0%AF.lua",
        "%.lua",
        "%GG.lua",
        "bad. /x.lua",
        "NUL.lua",
        "main.lua:ads",
        "x\\y.lua",
        "raw space.lua",
        "中.lua",
    ] {
        assert!(
            decode_local_file_uri(&format!("{prefix}{tail}")).is_err(),
            "{tail}"
        );
    }
    for uri in [
        "file://server/share/main.lua",
        "https://example.invalid/main.lua",
        "file:////root/main.lua",
        "file:/root/main.lua",
    ] {
        assert!(decode_local_file_uri(uri).is_err(), "{uri}");
    }
    assert!(decode_local_file_uri(&format!("{prefix}{}", "x".repeat(65537))).is_err());
    assert!(decode_path("long", 1).is_err());
}
