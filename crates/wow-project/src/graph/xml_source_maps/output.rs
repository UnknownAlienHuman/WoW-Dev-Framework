use super::*;

pub(super) fn append(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    prepared: PreparedXmlSourceMaps<'_>,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<XmlSourceMapSummary> {
    validate_project(project, source, stop)?;
    if project.snapshot_id() != prepared.project.snapshot_id()
        || project.analyzer_snapshot_id() != prepared.project.analyzer_snapshot_id()
        || project.snapshot().analyzer_binding().xml_lua_analysis() != prepared.analysis
        || source.xml_lua_analysis.as_ref() != prepared.analysis
    {
        return Err(invalid());
    }
    let raw_rows = source.raw_inventory().ok_or_else(invalid)?.members().len();
    let rows = entities
        .len()
        .checked_add(relations.len())
        .and_then(|rows| rows.checked_add(raw_rows))
        .filter(|rows| *rows <= MAX_PIECES)
        .ok_or_else(exhausted)?;
    let mut wanted_sites = BTreeMap::new();
    let mut wanted_spans = BTreeMap::new();
    for (position, unit) in prepared.units.iter().enumerate() {
        crate::analyzer::checkpoint(stop)?;
        let key = (
            unit.scope.as_ref(),
            unit.native.document.as_str(),
            unit.fact.document_digest,
            unit.fact.occurrence_id.as_str(),
        );
        budget.charge_serialized(&("xml-map-required-site", key), stop)?;
        if wanted_sites.insert(key, position).is_some() {
            return Err(invalid());
        }
        validate_support(
            project,
            source,
            fact_support(&unit.fact),
            &unit.fact.document,
            unit.fact.span,
            budget,
            stop,
        )?;
        for piece in &unit.pieces {
            crate::analyzer::checkpoint(stop)?;
            budget.charge_serialized(&("xml-map-required-span", piece.support), stop)?;
            if let Some(existing) = wanted_spans.insert(piece.support.handle, piece.support)
                && existing != piece.support
            {
                return Err(invalid());
            }
        }
    }
    let mut sites = BTreeMap::new();
    let mut spans = BTreeMap::new();
    for draft in entities.iter() {
        crate::analyzer::checkpoint(stop)?;
        let proposal = &draft.proposal;
        if proposal.entity_kind_id() == "xml_source_script_site" {
            let key = proposal.semantic_key();
            let digest = ContentDigest::<CanonicalResult>::parse(string(key, "document_digest")?)
                .map_err(|_| invalid())?;
            if !digest.was_canonical() {
                return Err(invalid());
            }
            let address = (
                string(key, "scope")?,
                string(key, "document")?,
                digest.into_value(),
                string(key, "occurrence")?,
            );
            let Some(position) = wanted_sites.get(&address).copied() else {
                continue;
            };
            let unit = prepared.units.get(position).ok_or_else(invalid)?;
            let support = fact_support(&unit.fact);
            if draft.producer != PlatformGraphProducer::XmlStructure
                || proposal.confidence() != GraphConfidence::Proven
                || key.len() != 5
                || string(key, "state")? != unit.script_state.as_ref()
                || proposal.source_handle_ids() != [support.handle]
                || proposal.evidence_ids() != [support.evidence]
            {
                return Err(invalid());
            }
            budget.charge_serialized(
                &("xml-map-script-address-copy", proposal.proposal_id()),
                stop,
            )?;
            if sites
                .insert(position, Box::<str>::from(proposal.proposal_id()))
                .is_some()
            {
                return Err(invalid());
            }
        } else if proposal.entity_kind_id() == "source_span" {
            let [handle] = proposal.source_handle_ids() else {
                return Err(invalid());
            };
            let Some(support) = wanted_spans.get(handle) else {
                continue;
            };
            let original = source.source_handles().get(handle).ok_or_else(invalid)?;
            validate_support(
                project,
                source,
                *support,
                original.path().as_str(),
                original.span(),
                budget,
                stop,
            )?;
            if draft.producer != PlatformGraphProducer::Inventory
                || original.span().kind() != SourceSpanKind::ByteRange
                || proposal.confidence() != GraphConfidence::Proven
                || proposal.evidence_ids() != [support.evidence]
                || proposal.semantic_key().len() != 1
                || string(proposal.semantic_key(), "source_handle")? != handle.canonical()
            {
                return Err(invalid());
            }
            budget.charge_serialized(
                &("xml-map-span-address-copy", handle, proposal.proposal_id()),
                stop,
            )?;
            if spans
                .insert(*handle, Box::<str>::from(proposal.proposal_id()))
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    if sites.len() != prepared.units.len() || spans.len() != wanted_spans.len() {
        return Err(invalid());
    }
    drop(wanted_sites);
    let mut output = Output {
        entities,
        relations,
        budget,
        rows,
        raw_rows,
        stop,
    };
    for (position, unit) in prepared.units.iter().enumerate() {
        crate::analyzer::checkpoint(stop)?;
        let native = unit.native;
        let support = fact_support(&unit.fact);
        let mut key = BTreeMap::new();
        output.field(&mut key, "scope", &unit.scope)?;
        output.field(&mut key, "document", &native.document)?;
        output.field(
            &mut key,
            "document_digest",
            &unit.fact.document_digest.canonical(),
        )?;
        output.field(&mut key, "occurrence", &unit.fact.occurrence_id)?;
        output.field(&mut key, "unit_id", &native.unit_id)?;
        output.field(&mut key, "extracted_unit_id", &native.extracted_unit_id)?;
        output.field(&mut key, "virtual_path", &native.virtual_path)?;
        output.field(
            &mut key,
            "content_digest",
            &native.content_digest.canonical(),
        )?;
        output.integer(&mut key, "byte_length", native.byte_length)?;
        output.field(
            &mut key,
            "analysis_id",
            prepared.analysis.ok_or_else(invalid)?.analysis_id(),
        )?;
        output.identifier(
            &mut key,
            "semantic_state",
            state_name(native.semantic_state),
        )?;
        let context = xml_roles::canonical_text(&native.context, output.budget, stop)?;
        output.field(&mut key, "semantic_context", &context)?;
        output.field(
            &mut key,
            "mapped_observations_digest",
            &unit.observations.canonical(),
        )?;
        let id = output.entity("xml_source_virtual_lua_unit", key, support)?;
        output.relation(
            "xml_script_site_owns_virtual_lua",
            sites.get(&position).ok_or_else(invalid)?,
            &id,
            support,
        )?;
        for piece in &unit.pieces {
            crate::analyzer::checkpoint(stop)?;
            let mut key = BTreeMap::new();
            output.field(&mut key, "unit_id", &native.unit_id)?;
            output.integer(
                &mut key,
                "ordinal",
                u64::try_from(piece.ordinal).map_err(|_| exhausted())?,
            )?;
            output.integer(&mut key, "virtual_byte_start", piece.segment.lua_byte_start)?;
            output.integer(&mut key, "virtual_byte_end", piece.segment.lua_byte_end)?;
            output.identifier(&mut key, "mapping_kind", mapping_name(piece.segment.kind))?;
            let xml_span = xml_roles::canonical_text(&piece.segment.xml_span, output.budget, stop)?;
            output.field(&mut key, "xml_span", &xml_span)?;
            output.field(&mut key, "source_handle", &piece.support.handle.canonical())?;
            let piece_id = output.entity("xml_source_virtual_lua_map_piece", key, piece.support)?;
            output.relation(
                "xml_virtual_lua_contains_map_piece",
                &id,
                &piece_id,
                piece.support,
            )?;
            output.relation(
                "xml_map_piece_source_span",
                &piece_id,
                spans.get(&piece.support.handle).ok_or_else(invalid)?,
                piece.support,
            )?;
        }
    }
    let analysis_id = prepared.analysis.map(ProjectXmlLuaAnalysis::analysis_id);
    let semantic_state = prepared.analysis.map_or(
        XmlLuaSemanticState::NotEvaluatedNoInlineUnits,
        ProjectXmlLuaAnalysis::semantic_state,
    );
    output.budget.charge_serialized(
        &(
            "xml-map-summary",
            analysis_id,
            semantic_state,
            prepared.units.len(),
            prepared.piece_count,
            &prepared.omissions,
        ),
        stop,
    )?;
    let summary = XmlSourceMapSummary {
        analysis_id: analysis_id.map(Into::into),
        semantic_state,
        unit_count: prepared.units.len(),
        piece_count: prepared.piece_count,
        omissions: prepared.omissions,
    };
    output.budget.charge_serialized(&summary, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(summary)
}

fn string<'a>(
    key: &'a BTreeMap<Box<str>, GraphProposalValue>,
    field: &str,
) -> ProjectResult<&'a str> {
    match key.get(field) {
        Some(GraphProposalValue::String(value)) => Ok(value),
        _ => Err(invalid()),
    }
}

fn state_name(state: XmlLuaSemanticState) -> &'static str {
    match state {
        XmlLuaSemanticState::Complete => "complete",
        XmlLuaSemanticState::PartialFailedParse => "partial_failed_parse",
        XmlLuaSemanticState::NotEvaluatedNoInlineUnits => "not_evaluated_no_inline_units",
    }
}

fn mapping_name(kind: XmlLuaMapKind) -> &'static str {
    match kind {
        XmlLuaMapKind::Identity => "identity",
        XmlLuaMapKind::XmlNewline => "xml_newline",
        XmlLuaMapKind::XmlEntity => "xml_entity",
    }
}

struct Output<'a> {
    entities: &'a mut Vec<EntityDraft>,
    relations: &'a mut Vec<RelationDraft>,
    budget: &'a mut ProducerBudget,
    rows: usize,
    raw_rows: usize,
    stop: &'a AtomicBool,
}

impl Output<'_> {
    fn field(
        &mut self,
        key: &mut BTreeMap<Box<str>, GraphProposalValue>,
        field: &'static str,
        value: &str,
    ) -> ProjectResult<()> {
        self.value(key, field, value, false)
    }

    fn identifier(
        &mut self,
        key: &mut BTreeMap<Box<str>, GraphProposalValue>,
        field: &'static str,
        value: &str,
    ) -> ProjectResult<()> {
        self.value(key, field, value, true)
    }

    fn value(
        &mut self,
        key: &mut BTreeMap<Box<str>, GraphProposalValue>,
        field: &'static str,
        value: &str,
        identifier: bool,
    ) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        if value.len() > MAX_VALUE_BYTES {
            return Err(exhausted());
        }
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(invalid());
        }
        self.budget.charge_serialized(&(field, value), self.stop)?;
        let value = if identifier {
            GraphProposalValue::Identifier(value.into())
        } else {
            GraphProposalValue::String(value.into())
        };
        if key.insert(field.into(), value).is_some() {
            return Err(invalid());
        }
        crate::analyzer::checkpoint(self.stop)
    }

    fn integer(
        &mut self,
        key: &mut BTreeMap<Box<str>, GraphProposalValue>,
        field: &'static str,
        value: u64,
    ) -> ProjectResult<()> {
        let value = i64::try_from(value).map_err(|_| exhausted())?;
        self.budget.charge_serialized(&(field, value), self.stop)?;
        if key
            .insert(field.into(), GraphProposalValue::Integer(value))
            .is_some()
        {
            return Err(invalid());
        }
        crate::analyzer::checkpoint(self.stop)
    }

    fn next(&mut self, entity: bool) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        if (entity
            && self
                .entities
                .len()
                .checked_add(self.raw_rows)
                .ok_or_else(exhausted)?
                >= MAX_NODES)
            || (!entity && self.relations.len() >= MAX_EDGES)
        {
            return Err(exhausted());
        }
        self.rows = self
            .rows
            .checked_add(1)
            .filter(|count| *count <= MAX_PIECES)
            .ok_or_else(exhausted)?;
        Ok(())
    }

    fn entity(
        &mut self,
        kind: &'static str,
        key: BTreeMap<Box<str>, GraphProposalValue>,
        support: Support,
    ) -> ProjectResult<Box<str>> {
        self.next(true)?;
        self.budget
            .charge_serialized(&(kind, &key, support), self.stop)?;
        let id = crate::identity::canonical_id(
            "xml-map:",
            "wow-project/platform-xml-source-map-entity/1",
            &(kind, &key),
            ProjectPhase::View,
        )?;
        self.budget
            .charge_serialized(&("xml-map-entity-address", &id), self.stop)?;
        let draft = EntityDraft::new(
            PlatformGraphProducer::XmlStructure,
            GraphEntityProposal::new(
                id.clone(),
                kind,
                key,
                GraphConfidence::Proven,
                vec![support.handle],
                vec![support.evidence],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        self.budget.charge_serialized(&draft, self.stop)?;
        crate::analyzer::checkpoint(self.stop)?;
        self.entities.push(draft);
        Ok(id)
    }

    fn relation(
        &mut self,
        kind: &'static str,
        from: &str,
        to: &str,
        support: Support,
    ) -> ProjectResult<()> {
        self.next(false)?;
        self.budget
            .charge_serialized(&(kind, from, to, support), self.stop)?;
        let id = crate::identity::canonical_id(
            "xml-map-edge:",
            "wow-project/platform-xml-source-map-relation/1",
            &(kind, from, to),
            ProjectPhase::View,
        )?;
        let draft = RelationDraft::new(
            PlatformGraphProducer::XmlStructure,
            id,
            kind,
            GraphRelationProposalInput {
                source: GraphProposalEndpoint::Proposed(from.into()),
                target: GraphProposalEndpoint::Proposed(to.into()),
                confidence: GraphConfidence::Proven,
                source_handle_ids: vec![support.handle],
                evidence_ids: vec![support.evidence],
                coverage_ids: Vec::new(),
            },
        )
        .map_err(|_| invalid())?;
        self.budget.charge_serialized(&draft, self.stop)?;
        crate::analyzer::checkpoint(self.stop)?;
        self.relations.push(draft);
        Ok(())
    }
}
