use super::*;

use wow_project::overlay::OverlayErrorCode;

pub(super) fn stage_replacement(
    scope: &OverlaySessionScope,
    current: &OverlayWorkspace,
    targets: &[WorkspaceRebindTarget],
    added_uri: &str,
    uri_policy: OverlayUriPolicy,
    limits: OverlayLimits,
    stop: &AtomicBool,
) -> ServiceResult<OverlayWorkspace> {
    if !(2..=LSP_MAX_WORKSPACE_REBIND_CANDIDATES).contains(&targets.len()) {
        return Err(invalid_frontend_request(
            "LSP workspace rebinding targets are invalid",
        ));
    }
    if current
        .validate_workspace_uri_with_policy(scope, added_uri, uri_policy)
        .is_ok()
    {
        return Err(invalid_frontend_request(
            "LSP workspace folder replacement must select a different root",
        ));
    }

    let mut matched: Option<OverlayWorkspace> = None;
    for target in targets {
        if let Err(error) = checkpoint(stop) {
            return Err(discard_optional(matched, scope, error));
        }
        let mut candidate = match OverlayWorkspace::register(
            scope,
            &target.workspace_root,
            &target.workspace_id,
            limits,
            stop,
        ) {
            Ok(candidate) => candidate,
            Err(error) if error.code() == OverlayErrorCode::InvalidRoot => continue,
            Err(error) => return Err(discard_optional(matched, scope, owner(error))),
        };
        if let Err(error) = checkpoint(stop) {
            return Err(discard_all(candidate, matched, scope, error));
        }
        match candidate.validate_workspace_uri_with_policy(scope, added_uri, uri_policy) {
            Ok(()) => {
                if let Some(previous) = matched.take() {
                    return Err(discard_all(
                        candidate,
                        Some(previous),
                        scope,
                        invalid_frontend_request("LSP workspace folder replacement is ambiguous"),
                    ));
                }
                matched = Some(candidate);
            }
            Err(error) if error.code() == OverlayErrorCode::InvalidRoot => {
                if candidate.close(scope).is_err() {
                    return Err(discard_optional(matched, scope, staged_close_failure()));
                }
            }
            Err(error) => {
                return Err(discard_all(candidate, matched, scope, owner(error)));
            }
        }
    }
    if let Err(error) = checkpoint(stop) {
        return Err(discard_optional(matched, scope, error));
    }
    matched.ok_or_else(|| {
        invalid_frontend_request(
            "LSP workspace folder replacement could not be qualified under the host allowlist",
        )
    })
}

pub(super) fn close_staged_or_error(
    mut candidate: OverlayWorkspace,
    scope: &OverlaySessionScope,
    error: ServiceError,
) -> ServiceError {
    if candidate.close(scope).is_err() {
        staged_close_failure()
    } else {
        error
    }
}

fn staged_close_failure() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::InternalContractViolation,
        "staged LSP workspace close failed",
    )
}

fn discard_optional(
    candidate: Option<OverlayWorkspace>,
    scope: &OverlaySessionScope,
    error: ServiceError,
) -> ServiceError {
    candidate.map_or(error.clone(), |candidate| {
        close_staged_or_error(candidate, scope, error)
    })
}

fn discard_all(
    candidate: OverlayWorkspace,
    previous: Option<OverlayWorkspace>,
    scope: &OverlaySessionScope,
    error: ServiceError,
) -> ServiceError {
    let mut failure = close_staged_or_error(candidate, scope, error);
    if let Some(previous) = previous {
        // The second close is always attempted so every staged registration is
        // checked even when cleanup of the first candidate fails.
        failure = close_staged_or_error(previous, scope, failure);
    }
    failure
}
