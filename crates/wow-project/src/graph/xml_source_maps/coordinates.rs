use super::*;

pub(super) fn line_starts(
    text: &str,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Vec<usize>> {
    budget.charge_serialized(&("xml-map-line-offset", 0usize), stop)?;
    let mut result = vec![0];
    let bytes = text.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if index % 4096 == 0 {
            crate::analyzer::checkpoint(stop)?;
        }
        if *byte == b'\n' || (*byte == b'\r' && bytes.get(index + 1) != Some(&b'\n')) {
            let offset = index.checked_add(1).ok_or_else(exhausted)?;
            budget.charge_serialized(&("xml-map-line-offset", offset), stop)?;
            result.push(offset);
        }
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(result)
}

pub(super) fn validate_xml_span(
    text: &str,
    lines: &[usize],
    span: &XmlSourceSpan,
) -> ProjectResult<()> {
    let start = usize::try_from(span.byte_start).map_err(|_| invalid())?;
    let end = usize::try_from(span.byte_end).map_err(|_| invalid())?;
    text.get(start..end).ok_or_else(invalid)?;
    let first = lines
        .partition_point(|offset| *offset <= start)
        .checked_sub(1)
        .ok_or_else(invalid)?;
    let last = lines
        .partition_point(|offset| *offset <= end)
        .checked_sub(1)
        .ok_or_else(invalid)?;
    if span.start_line != first as u64 + 1
        || span.end_line != last as u64 + 1
        || span.start_byte_column != (start - lines[first]) as u64 + 1
        || span.end_byte_column != (end - lines[last]) as u64 + 1
    {
        return Err(invalid());
    }
    Ok(())
}

pub(super) fn visit(total: &mut usize, count: usize) -> ProjectResult<()> {
    *total = total
        .checked_add(count)
        .filter(|count| *count <= MAX_PIECES)
        .ok_or_else(exhausted)?;
    Ok(())
}

pub(super) fn validate_body(
    body: &XmlInlineLua,
    xml: &str,
    lines: &[usize],
    body_span: &XmlSourceSpan,
    visited: &mut usize,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    if body.byte_length != body.text().len() as u64
        || body.content_digest != crate::identity::source_digest(body.text().as_bytes())
    {
        return Err(invalid());
    }
    let mut virtual_end = 0;
    let mut xml_end = body_span.byte_start;
    for segment in body.segments() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(&("xml-map-segment-validation", segment), stop)?;
        validate_xml_span(xml, lines, &segment.xml_span)?;
        if segment.lua_byte_start != virtual_end
            || segment.lua_byte_start >= segment.lua_byte_end
            || segment.xml_span.byte_start < xml_end
            || segment.xml_span.byte_start >= segment.xml_span.byte_end
            || segment.xml_span.byte_end > body_span.byte_end
        {
            return Err(invalid());
        }
        let start = usize::try_from(segment.lua_byte_start).map_err(|_| invalid())?;
        let end = usize::try_from(segment.lua_byte_end).map_err(|_| invalid())?;
        let lua = body.text().get(start..end).ok_or_else(invalid)?;
        let original = xml
            .get(
                usize::try_from(segment.xml_span.byte_start).map_err(|_| invalid())?
                    ..usize::try_from(segment.xml_span.byte_end).map_err(|_| invalid())?,
            )
            .ok_or_else(invalid)?;
        match segment.kind {
            XmlLuaMapKind::Identity if lua != original => return Err(invalid()),
            XmlLuaMapKind::XmlNewline if lua != "\n" || !matches!(original, "\r" | "\r\n") => {
                return Err(invalid());
            }
            XmlLuaMapKind::XmlEntity if !original.starts_with('&') || !original.ends_with(';') => {
                return Err(invalid());
            }
            _ => {}
        }
        virtual_end = segment.lua_byte_end;
        xml_end = segment.xml_span.byte_end;
    }
    if virtual_end != body.byte_length {
        return Err(invalid());
    }
    // Empty native units deliberately have no invented caret or source piece.
    for segment in body.segments() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &(
                "xml-map-native-coordinate-check",
                segment,
                &segment.xml_span,
                &segment.xml_span,
            ),
            stop,
        )?;
        let start = usize::try_from(segment.lua_byte_start).map_err(|_| invalid())?;
        let end = usize::try_from(segment.lua_byte_end).map_err(|_| invalid())?;
        precharge_mapping(body, start as u64, end as u64, visited, budget, stop)?;
        if body.map_range(start, end)? != [segment.xml_span.clone()] {
            return Err(invalid());
        }
        for (position, expected) in [
            (start, segment.xml_span.byte_start),
            (end, segment.xml_span.byte_end),
        ] {
            precharge_mapping(
                body,
                position as u64,
                position as u64,
                visited,
                budget,
                stop,
            )?;
            let candidates = body.map_position(position)?;
            if !candidates
                .iter()
                .any(|span| span.byte_start == expected && span.byte_end == expected)
            {
                return Err(invalid());
            }
            for span in &candidates {
                validate_xml_span(xml, lines, span)?;
            }
        }
    }
    crate::analyzer::checkpoint(stop)
}

// Charge the native candidate window before its range/caret result is copied.
// Caret deduplication can reduce this conservative visit/emission bound.
fn precharge_mapping(
    body: &XmlInlineLua,
    start: u64,
    end: u64,
    visited: &mut usize,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let caret = start == end;
    let segments = body.segments();
    let first = segments.partition_point(|segment| {
        if caret {
            segment.lua_byte_end < start
        } else {
            segment.lua_byte_end <= start
        }
    });
    let last = segments.partition_point(|segment| {
        if caret {
            segment.lua_byte_start <= end
        } else {
            segment.lua_byte_start < end
        }
    });
    let candidates = segments.get(first..last).ok_or_else(invalid)?;
    visit(
        visited,
        candidates.len().checked_mul(2).ok_or_else(exhausted)?,
    )?;
    for segment in candidates {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &("xml-map-native-mapping-copy", start, end, &segment.xml_span),
            stop,
        )?;
    }
    crate::analyzer::checkpoint(stop)
}

pub(super) fn validate_observations(
    unit: &XmlLuaUnitAnalysis,
    body: &XmlInlineLua,
    xml: &str,
    lines: &[usize],
    visited: &mut usize,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let mut coordinates = Coordinates {
        body,
        xml,
        lines,
        visited,
        budget,
        stop,
    };
    for diagnostic in &unit.diagnostics {
        coordinates.check(
            diagnostic.parser_diagnostic.byte_start,
            diagnostic.parser_diagnostic.byte_end,
            diagnostic.mapping,
            &diagnostic.xml_spans,
        )?;
    }
    for diagnostic in &unit.semantic_diagnostics {
        coordinates.span(&diagnostic.source)?;
    }
    for reference in &unit.member_references {
        coordinates.span(&reference.receiver_source)?;
        coordinates.span(&reference.member_source)?;
        coordinates.span(&reference.reference_source)?;
    }
    for call in &unit.member_calls {
        coordinates.span(&call.callee_source)?;
        coordinates.span(&call.call_source)?;
    }
    crate::analyzer::checkpoint(stop)
}

struct Coordinates<'a> {
    body: &'a XmlInlineLua,
    xml: &'a str,
    lines: &'a [usize],
    visited: &'a mut usize,
    budget: &'a mut ProducerBudget,
    stop: &'a AtomicBool,
}

impl Coordinates<'_> {
    fn span(&mut self, span: &XmlLuaMappedSpan) -> ProjectResult<()> {
        self.check(
            span.virtual_byte_start,
            span.virtual_byte_end,
            span.mapping,
            &span.xml_spans,
        )
    }

    fn check(
        &mut self,
        start: u64,
        end: u64,
        mapping: XmlLuaDiagnosticMapping,
        spans: &[XmlSourceSpan],
    ) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        visit(self.visited, spans.len())?;
        self.budget.charge_serialized(
            &(
                "xml-map-observation-coordinate-check",
                start,
                end,
                mapping,
                spans,
            ),
            self.stop,
        )?;
        if spans.is_empty()
            || start > end
            || (mapping == XmlLuaDiagnosticMapping::CaretBoundaries) != (start == end)
        {
            return Err(invalid());
        }
        for span in spans {
            crate::analyzer::checkpoint(self.stop)?;
            validate_xml_span(self.xml, self.lines, span)?;
        }
        precharge_mapping(self.body, start, end, self.visited, self.budget, self.stop)?;
        let start = usize::try_from(start).map_err(|_| invalid())?;
        let end = usize::try_from(end).map_err(|_| invalid())?;
        let actual = if start == end {
            self.body.map_position(start)?
        } else {
            self.body.map_range(start, end)?
        };
        if actual != spans {
            return Err(invalid());
        }
        crate::analyzer::checkpoint(self.stop)
    }
}
