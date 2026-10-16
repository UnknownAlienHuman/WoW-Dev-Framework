//! Kind-specific selectors never reinterpret declaration fields as API arguments.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeclarationShape {
    kind: NativeDeclarationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    fields: Vec<FieldFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    argument_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    arguments: Vec<FieldFilter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    returns: Vec<FieldFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    member_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    member_name: Option<String>,
}
impl DeclarationShape {
    pub(super) fn validate(&self) -> SearchResult<()> {
        let fields = self.field_count.is_some() || !self.fields.is_empty();
        let signature = self.argument_count.is_some()
            || self.return_count.is_some()
            || !self.arguments.is_empty()
            || !self.returns.is_empty();
        let values = self.member_count.is_some() || self.member_name.is_some();
        let valid = match self.kind {
            NativeDeclarationKind::Structure => !signature && !values,
            NativeDeclarationKind::Callback => !fields && !values,
            NativeDeclarationKind::Enumeration | NativeDeclarationKind::Constants => {
                !fields && !signature
            }
            NativeDeclarationKind::ScriptObject => !fields && !signature && !values,
        };
        if !valid {
            return Err(SearchError::InvalidProfile);
        }
        if let Some(name) = &self.member_name {
            valid_text(name)?;
        }
        for filters in [&self.fields, &self.arguments, &self.returns] {
            validate_fields(filters)?;
        }
        Ok(())
    }
    pub(super) fn matches(&self, data: &declarations::DeclarationDocument) -> bool {
        self.kind == data.kind
            && self
                .field_count
                .is_none_or(|count| count as usize == data.fields.len())
            && self
                .argument_count
                .is_none_or(|count| count as usize == data.arguments.len())
            && self
                .return_count
                .is_none_or(|count| count as usize == data.returns.len())
            && self
                .member_count
                .is_none_or(|count| count as usize == data.values.len())
            && self
                .member_name
                .as_ref()
                .is_none_or(|name| data.values.iter().any(|value| &value.name == name))
            && fields_match(&data.fields, &self.fields)
            && fields_match(&data.arguments, &self.arguments)
            && fields_match(&data.returns, &self.returns)
    }
}
