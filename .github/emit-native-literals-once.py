from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"unexpected block count in {path}: {count}")
    file.write_text(text.replace(old, new))


# Keep literal admission rules in the native Ketho renderer. Annotation projection
# asks this owner whether one declaration is representable, then the aggregate
# renderer emits the final file once.
replace(
    "crates/wow-ketho-literals/src/lib.rs",
    """    /// Ketho `GetEventLiterals`. Retains the open `string` base of FrameEvent;
    /// this artifact is not an exhaustive runtime event whitelist.
    pub fn render_events(&self, events: &[EventLiteral]) -> Result<String, LiteralError> {
""",
    """    /// Validate one event declaration without emitting a discarded artifact.
    pub fn validate_event(event: &EventLiteral) -> Result<(), LiteralError> {
        literal_name(&event.name)?;
        event_payload(&event.payload)
    }

    /// Validate one enum declaration without assembling the aggregate file.
    pub fn validate_enum(declaration: &EnumDeclaration) -> Result<(), LiteralError> {
        identifier(&declaration.name)?;
        members(&declaration.values, MemberOrder::Value, true)?;
        Ok(())
    }

    /// Validate one constants group without assembling the aggregate file.
    pub fn validate_constant_group(group: &ConstantGroup) -> Result<(), LiteralError> {
        identifier(&group.name)?;
        members(&group.values, group.order, false)?;
        Ok(())
    }

    /// Ketho `GetEventLiterals`. Retains the open `string` base of FrameEvent;
    /// this artifact is not an exhaustive runtime event whitelist.
    pub fn render_events(&self, events: &[EventLiteral]) -> Result<String, LiteralError> {
""",
)
replace(
    "crates/wow-ketho-literals/src/lib.rs",
    """            literal_name(&event.name)?;
            unique(&mut previous, &event.name)?;
            safe_text(&event.payload)?;
            if event.payload.contains('`') {
                return Err(LiteralError::UnsafeDocumentation);
            }
""",
    """            literal_name(&event.name)?;
            unique(&mut previous, &event.name)?;
            event_payload(&event.payload)?;
""",
)
replace(
    "crates/wow-ketho-literals/src/lib.rs",
    """fn unique<'a>(previous: &mut Option<&'a str>, name: &'a str) -> Result<(), LiteralError> {
""",
    """fn event_payload(payload: &str) -> Result<(), LiteralError> {
    safe_text(payload)?;
    if payload.contains('`') {
        return Err(LiteralError::UnsafeDocumentation);
    }
    Ok(())
}

fn unique<'a>(previous: &mut Option<&'a str>, name: &'a str) -> Result<(), LiteralError> {
""",
)

replace(
    "crates/wow-annotations/src/literals.rs",
    """    pub fn render_events(&self, values: &[EventLiteral]) -> Result<String, RenderError> {
        self.0.render_events(values).map_err(error)
    }
""",
    """    pub fn validate_event(value: &EventLiteral) -> Result<(), RenderError> {
        NativeRenderer::validate_event(value).map_err(error)
    }
    pub fn validate_enum(value: &EnumDeclaration) -> Result<(), RenderError> {
        NativeRenderer::validate_enum(value).map_err(error)
    }
    pub fn validate_constant_group(value: &ConstantGroup) -> Result<(), RenderError> {
        NativeRenderer::validate_constant_group(value).map_err(error)
    }
    pub fn render_events(&self, values: &[EventLiteral]) -> Result<String, RenderError> {
        self.0.render_events(values).map_err(error)
    }
""",
)

replace(
    "crates/wow-annotations/src/native.rs",
    "match literals.render_enums(std::slice::from_ref(&declaration), &[]) {",
    "match LiteralRenderer::validate_enum(&declaration) {",
)
replace(
    "crates/wow-annotations/src/native.rs",
    "match literals.render_enums(&[], std::slice::from_ref(&declaration)) {",
    "match LiteralRenderer::validate_constant_group(&declaration) {",
)
replace(
    "crates/wow-annotations/src/native.rs",
    """            events.push(EventLiteral {
                name: event.literal_name.into(),
                payload: event
                    .payload
                    .iter()
                    .map(|f| f.name)
                    .collect::<Vec<_>>()
                    .join(", "),
            });
            event_sources.push(link(document, event.raw));
""",
    """            let declaration = EventLiteral {
                name: event.literal_name.into(),
                payload: event
                    .payload
                    .iter()
                    .map(|f| f.name)
                    .collect::<Vec<_>>()
                    .join(", "),
            };
            match LiteralRenderer::validate_event(&declaration) {
                Ok(()) => {
                    events.push(declaration);
                    event_sources.push(link(document, event.raw));
                }
                Err(error) => issues.push(issue(
                    document,
                    event.raw,
                    format!("renderer_{error:?}"),
                )),
            }
""",
)
replace(
    "crates/wow-annotations/src/native.rs",
    """        if !enums.is_empty() || !constants.is_empty() {
            match literals.render_enums(&enums, &constants) {
                Ok(text) => {
                    let _ = text; // validate this registration before aggregation
                    all_enums.extend(enums);
                    lowering::reserve_vec(
                        &mut all_enum_sources,
                        enum_sources.len(),
                        &mut lowering_budget,
                        cancelled,
                    )?;
                    all_enum_sources.extend(enum_sources);
                    all_constants.extend(constants);
                    all_literal_sources.extend(literal_sources);
                }
                Err(error) => {
                    issues.push(issue(document, system.raw, format!("literals_{error:?}")))
                }
            }
        }
        if !events.is_empty() {
            match literals.render_events(&events) {
                Ok(text) => {
                    let _ = text;
                    all_events.extend(events);
                    all_event_sources.extend(event_sources);
                }
                Err(error) => issues.push(issue(document, system.raw, format!("events_{error:?}"))),
            }
        }
""",
    """        if !enums.is_empty() || !constants.is_empty() {
            all_enums.extend(enums);
            lowering::reserve_vec(
                &mut all_enum_sources,
                enum_sources.len(),
                &mut lowering_budget,
                cancelled,
            )?;
            all_enum_sources.extend(enum_sources);
            all_constants.extend(constants);
            all_literal_sources.extend(literal_sources);
        }
        if !events.is_empty() {
            all_events.extend(events);
            all_event_sources.extend(event_sources);
        }
""",
)

replace(
    "docs/IMPLEMENTATION.md",
    """#204 removes discarded trial renders in `native.rs`: reuse emission helpers
and retain their text/ranges for assembly. Reject only the invalid declaration
or group while keeping independent valid output; preserve aggregate limits.
""",
    """#204 has removed enum/constant/event trial renders: their native owner now
validates each declaration and final assembly emits the aggregate files once.
Functions/tables still need retained mapped emission instead of discarded trial
renders. Reject only the invalid declaration while preserving aggregate limits.
""",
)
replace(
    "docs/REFACTOR_QUEUE.md",
    "| Independent | [#204 — single emission](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/204) | Retain emitted text/maps instead of trial rendering; keep valid neighbors and aggregate bounds. |\n",
    "| Independent | [#204 — single emission](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/204) | Literal trial renders are removed; retain mapped function/table output instead of rendering it twice. |\n",
)
