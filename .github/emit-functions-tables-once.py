from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one block, found {count}")
    file.write_text(text.replace(old, new))


replace(
    "crates/wow-annotations/src/ketho.rs",
    '''    /// Ketho `GetSystem` plus its analysis-only `---@meta _` file header.
    pub fn render(&self, system: &System) -> Result<String, RenderError> {
''',
    '''    /// Validate one callable with the renderer's own declaration rules,
    /// without constructing and discarding a candidate output file.
    pub(crate) fn validate_function(
        &self,
        owner: &Owner,
        function: &Function,
    ) -> Result<(), RenderError> {
        validate_owner(owner)?;
        identifier(&function.name)?;
        validate_fields(&function.arguments, true)?;
        validate_fields(&function.returns, true)?;
        if let Some(documentation) = &function.documentation {
            if documentation.len() > MAX_ITEMS {
                return Err(RenderError::InputLimit);
            }
            for text in documentation {
                safe_text(text)?;
                if text.trim_start().starts_with('@') {
                    return Err(RenderError::UnsafeDocumentation);
                }
            }
        }
        for field in function.arguments.iter().chain(&function.returns) {
            self.validate_field(field)?;
        }
        Ok(())
    }

    /// Validate one structure/callback declaration with the same helpers used
    /// by final mapped emission. Aggregate ordering and byte limits stay final.
    pub(crate) fn validate_table(&self, table: &Table) -> Result<(), RenderError> {
        identifier(table.name())?;
        match table {
            Table::Structure { fields, .. } => {
                validate_fields(fields, false)?;
                for field in fields {
                    self.validate_field(field)?;
                }
            }
            Table::Callback { arguments, .. } => {
                validate_fields(arguments, false)?;
                for argument in arguments {
                    if argument.inner_type.is_some() {
                        return Err(RenderError::UnsupportedType);
                    }
                    self.validate_field(argument)?;
                }
            }
            Table::CallbackSignature {
                arguments, returns, ..
            } => self.validate_callback_signature(arguments, returns)?,
        }
        Ok(())
    }

    fn validate_field(&self, field: &Field) -> Result<(), RenderError> {
        drop(self.field_type(field)?);
        if let Some(default) = &field.default_text {
            safe_text(default)?;
        }
        Ok(())
    }

    /// Ketho `GetSystem` plus its analysis-only `---@meta _` file header.
    pub fn render(&self, system: &System) -> Result<String, RenderError> {
''',
)

replace(
    "crates/wow-annotations/src/ketho.rs",
    '''        for (index, function) in system.functions.iter().enumerate() {
            identifier(&function.name)?;
            if !function_names.insert(&function.name) {
''',
    '''        for (index, function) in system.functions.iter().enumerate() {
            self.validate_function(&system.owner, function)?;
            if !function_names.insert(&function.name) {
''',
)

replace(
    "crates/wow-annotations/src/ketho.rs",
    '''        for (index, table) in system.tables.iter().enumerate() {
            let name = table.name();
            identifier(name)?;
            if !table_names.insert(name) {
''',
    '''        for (index, table) in system.tables.iter().enumerate() {
            let name = table.name();
            self.validate_table(table)?;
            if !table_names.insert(name) {
''',
)

replace(
    "crates/wow-annotations/src/ketho/callbacks.rs",
    '''    Renderer, safe_text, validate_fields,
''',
    '''    Renderer, validate_fields,
''',
)

replace(
    "crates/wow-annotations/src/ketho/callbacks.rs",
    '''impl Renderer {
    /// The same type lowering serves fields and function-type components.
''',
    '''impl Renderer {
    pub(super) fn validate_callback_signature(
        &self,
        arguments: &[Field],
        returns: &[Field],
    ) -> Result<(), RenderError> {
        validate_fields(arguments, true)?;
        // Variadic return packs are a distinct consumer contract, not a tuple.
        validate_fields(returns, false)?;
        for argument in arguments {
            if argument.variadic && argument.default_text.is_some() {
                return Err(RenderError::InvalidVariadic);
            }
            self.validate_field(argument)?;
        }
        for result in returns {
            self.validate_field(result)?;
        }
        Ok(())
    }

    /// The same type lowering serves fields and function-type components.
''',
)

replace(
    "crates/wow-annotations/src/ketho/callbacks.rs",
    '''        validate_fields(arguments, true)?;
        // Variadic return packs are a distinct consumer contract, not a tuple.
        validate_fields(returns, false)?;
        for field in arguments.iter().chain(returns) {
            if let Some(default) = &field.default_text {
                safe_text(default)?;
            }
        }
''',
    '''        self.validate_callback_signature(arguments, returns)?;
''',
)

replace(
    "crates/wow-annotations/src/ketho/callbacks.rs",
    '''        for (index, argument) in arguments.iter().enumerate() {
            if argument.variadic && argument.default_text.is_some() {
                return Err(RenderError::InvalidVariadic);
            }
            if index != 0 {
''',
    '''        for (index, argument) in arguments.iter().enumerate() {
            if index != 0 {
''',
)

replace(
    "crates/wow-annotations/src/native.rs",
    '''                Ok(projected) => {
                    let candidate = System {
                        owner: input.owner.clone(),
                        functions: vec![projected.function],
                        tables: Vec::new(),
                    };
                    match renderer.render(&candidate) {
                        Ok(_) => {
                            name_projections.extend(projected.names);
                            if projected.escaped_documentation {
                                metadata_sidecars.push(MetadataSidecar {
                                    field: "Documentation:escaped_control_characters".into(),
                                    source: link(document, function.raw),
                                });
                            }
                            input.functions.extend(candidate.functions);
                            function_sources.push(link(document, function.raw));
                            function_facts.push(function);
                        }
                        Err(error) => issues.push(issue(
                            document,
                            function.raw,
                            format!("renderer_{error:?}"),
                        )),
                    }
                }
''',
    '''                Ok(projected) => {
                    match renderer.validate_function(&input.owner, &projected.function) {
                        Ok(()) => {
                            name_projections.extend(projected.names);
                            if projected.escaped_documentation {
                                metadata_sidecars.push(MetadataSidecar {
                                    field: "Documentation:escaped_control_characters".into(),
                                    source: link(document, function.raw),
                                });
                            }
                            input.functions.push(projected.function);
                            function_sources.push(link(document, function.raw));
                            function_facts.push(function);
                        }
                        Err(error) => issues.push(issue(
                            document,
                            function.raw,
                            format!("renderer_{error:?}"),
                        )),
                    }
                }
''',
)

replace(
    "crates/wow-annotations/src/native.rs",
    '''                Ok(Some(projected_table)) => {
                    let candidate = System {
                        owner: input.owner.clone(),
                        functions: Vec::new(),
                        tables: vec![projected_table],
                    };
                    match renderer.render(&candidate) {
                        Ok(_) => {
                            input.tables.extend(candidate.tables);
                            table_sources.push(link(document, raw));
                            table_facts.push(table);
                        }
                        Err(error) => {
                            issues.push(issue(document, raw, format!("renderer_{error:?}")))
                        }
                    }
                }
''',
    '''                Ok(Some(projected_table)) => match renderer.validate_table(&projected_table) {
                    Ok(()) => {
                        input.tables.push(projected_table);
                        table_sources.push(link(document, raw));
                        table_facts.push(table);
                    }
                    Err(error) => {
                        issues.push(issue(document, raw, format!("renderer_{error:?}")))
                    }
                },
''',
)

replace(
    "docs/IMPLEMENTATION.md",
    '''#204 removes discarded trial renders in `native.rs`: reuse emission helpers
and retain their text/ranges for assembly. Reject only the invalid declaration
or group while keeping independent valid output; preserve aggregate limits.
Do not duplicate rendering rules in a new validator. #202 shares a parsed
catalog tree between existing validators.
''',
    '''Functions, structures/callback tables, enums, constants and events now use
renderer-owned declaration validation; only the final mapped aggregate render
emits bytes. #202 likewise shares one parsed catalog tree between validators.
Keep these owner boundaries when extending Ketho behavior; do not reintroduce
trial artifacts or a second type/validation model.
''',
)

replace(
    "docs/REFACTOR_QUEUE.md",
    '''| Independent | [#204 — single emission](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/204) | Literal trial renders are removed; retain mapped function/table output instead of rendering it twice. |
''',
    '''| Completed | [#204 — single emission](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/204) | Renderer-owned validation admits declarations; final mapped assembly is the only byte emission. |
''',
)
