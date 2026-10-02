from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    item = Path(path)
    text = item.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(
            f"patch marker count for {path}: expected 1, found {count}: {old[:120]!r}"
        )
    item.write_text(text.replace(old, new, 1))


path = "crates/wow-emmy/src/references.rs"
replace(
    path,
    """/// Resolve an additional reviewed callable-query profile in the same exact
/// semantic session. The returned symbol lookup remains scoped to `queries`;
/// callable-only lookup identity is retained by the function-call sidecar.
pub fn analyze_member_call_session_with_callable_queries(
    main: &LuaWorkspaceSnapshot,
    libraries: &[&LuaWorkspaceSnapshot],
    queries: &[String],
    callable_queries: &[String],
    include_function_calls: bool,
    stop: &std::sync::atomic::AtomicBool,
) -> EmmyMemberCallResult<MemberCallSession> {
    analyze_member_call_session_impl(
        main,
        libraries,
        None,
        None,
        queries,
        callable_queries,
        include_function_calls,
        stop,
    )
}
""",
    """/// Exact query lanes for one already-populated member-call session.
#[derive(Debug, Clone, Copy)]
pub struct MemberCallSessionQueryProfile<'a> {
    symbol_queries: &'a [String],
    callable_queries: &'a [String],
}

impl<'a> MemberCallSessionQueryProfile<'a> {
    #[must_use]
    pub const fn new(
        symbol_queries: &'a [String],
        callable_queries: &'a [String],
    ) -> Self {
        Self {
            symbol_queries,
            callable_queries,
        }
    }
}

/// Resolve an additional reviewed callable-query profile in the same exact
/// semantic session. The returned symbol lookup remains scoped to the symbol
/// lane; callable-only lookup identity is retained by the function-call sidecar.
pub fn analyze_member_call_session_with_callable_queries(
    main: &LuaWorkspaceSnapshot,
    libraries: &[&LuaWorkspaceSnapshot],
    query_profile: MemberCallSessionQueryProfile<'_>,
    include_function_calls: bool,
    stop: &std::sync::atomic::AtomicBool,
) -> EmmyMemberCallResult<MemberCallSession> {
    analyze_member_call_session_impl(
        main,
        libraries,
        None,
        None,
        query_profile.symbol_queries,
        query_profile.callable_queries,
        include_function_calls,
        stop,
    )
}
""",
)
replace(
    path,
    """pub fn analyze_member_call_session_with_virtual_and_callable_queries(
    main: &LuaWorkspaceSnapshot,
    libraries: &[&LuaWorkspaceSnapshot],
    virtual_units: &LuaWorkspaceSnapshot,
    project_generation: ProjectGenerationId,
    queries: &[String],
    callable_queries: &[String],
    include_function_calls: bool,
    stop: &std::sync::atomic::AtomicBool,
) -> EmmyMemberCallResult<MemberCallSession> {
    analyze_member_call_session_impl(
        main,
        libraries,
        Some(virtual_units),
        Some(project_generation),
        queries,
        callable_queries,
        include_function_calls,
        stop,
    )
}
""",
    """pub fn analyze_member_call_session_with_virtual_and_callable_queries(
    main: &LuaWorkspaceSnapshot,
    libraries: &[&LuaWorkspaceSnapshot],
    virtual_units: &LuaWorkspaceSnapshot,
    project_generation: ProjectGenerationId,
    query_profile: MemberCallSessionQueryProfile<'_>,
    include_function_calls: bool,
    stop: &std::sync::atomic::AtomicBool,
) -> EmmyMemberCallResult<MemberCallSession> {
    analyze_member_call_session_impl(
        main,
        libraries,
        Some(virtual_units),
        Some(project_generation),
        query_profile.symbol_queries,
        query_profile.callable_queries,
        include_function_calls,
        stop,
    )
}
""",
)

path = "crates/wow-emmy/src/lib.rs"
replace(
    path,
    """    EmmyReferenceResolution, analyze_member_call_session_with_callable_queries,
    analyze_member_call_session_with_virtual_and_callable_queries, analyze_member_calls,
""",
    """    EmmyReferenceResolution, MemberCallSessionQueryProfile,
    analyze_member_call_session_with_callable_queries,
    analyze_member_call_session_with_virtual_and_callable_queries, analyze_member_calls,
""",
)

path = "crates/wow-project/src/analyzer.rs"
replace(
    path,
    """    let session = match pending_xml_lua
        .as_ref()
        .and_then(crate::xml_lua::PreparedProjectXmlLuaAnalysis::virtual_workspace)
    {
        Some(virtual_workspace) => {
            wow_emmy::references::analyze_member_call_session_with_virtual_and_callable_queries(
                &main_workspace,
                &library_refs,
                virtual_workspace,
                generation.project_generation(),
                queries,
                &callable_queries,
                function_calls,
                stop,
            )
        }
        None => wow_emmy::references::analyze_member_call_session_with_callable_queries(
            &main_workspace,
            &library_refs,
            queries,
            &callable_queries,
            function_calls,
            stop,
        ),
    }
""",
    """    let query_profile =
        wow_emmy::MemberCallSessionQueryProfile::new(queries, &callable_queries);
    let session = match pending_xml_lua
        .as_ref()
        .and_then(crate::xml_lua::PreparedProjectXmlLuaAnalysis::virtual_workspace)
    {
        Some(virtual_workspace) => {
            wow_emmy::references::analyze_member_call_session_with_virtual_and_callable_queries(
                &main_workspace,
                &library_refs,
                virtual_workspace,
                generation.project_generation(),
                query_profile,
                function_calls,
                stop,
            )
        }
        None => wow_emmy::references::analyze_member_call_session_with_callable_queries(
            &main_workspace,
            &library_refs,
            query_profile,
            function_calls,
            stop,
        ),
    }
""",
)
