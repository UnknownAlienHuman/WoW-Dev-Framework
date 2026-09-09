//! Static external class validation against exact emitted header and method bytes.
use super::{Result, list, lower_terms, span_key, text};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Key<'a> = (&'a str, (usize, usize));

pub(super) fn verify<'a>(
    library: &'a Value,
    sources: &BTreeMap<&'a str, &'a Value>,
    mapped: &mut BTreeMap<Key<'a>, &'a str>,
    emitted: &mut BTreeSet<&'a str>,
) -> Result<bool> {
    let report = &library["aliases"];
    if report["schema"] != "wow-native-alias-projection/8" {
        if report.get("function_container_outcomes").is_some()
            || report
                .get("unresolved_function_container_returns")
                .is_some()
        {
            return Err("unexpected function container projection field".into());
        }
        return Ok(false);
    }
    let mut entries = Vec::new();
    for (&path, &resource) in sources {
        if resource.get("function_containers").is_some() {
            for container in list(resource, "function_containers")? {
                entries.push((path, resource, container));
            }
        }
    }
    let outcomes = list(report, "function_container_outcomes")?;
    if entries.is_empty() || outcomes.len() != entries.len() {
        return Err("missing external function container outcomes".into());
    }
    let mut blocked = false;
    let mut container_spans = BTreeSet::new();
    let mut emitted_returns = BTreeSet::new();
    for (ordinal, ((path, resource, container), outcome)) in
        entries.into_iter().zip(outcomes).enumerate()
    {
        let name = text(container, "name")?;
        let span = span_key(&container["span"])?;
        let header = span_key(&container["header_span"])?;
        let raw = text(resource, "text")?;
        let methods = list(container, "methods")?;
        if span.0 >= span.1
            || header.0 < span.0
            || header.0 >= header.1
            || header.1 > span.1
            || raw.get(span.0..span.1).is_none()
            || raw.get(header.0..header.1).is_none()
            || !container_spans.insert((path, span))
            || methods.is_empty()
            || outcome["ordinal"] != json!(ordinal)
            || outcome["name"] != name
        {
            return Err("function container source/outcome mismatch".into());
        }
        let mut previous = header.1;
        let mut method_spans = Vec::with_capacity(methods.len());
        for method in methods {
            let method_span = span_key(&method["span"])?;
            if method_span.0 < previous
                || method_span.0 >= method_span.1
                || method_span.1 > span.1
                || raw.get(method_span.0..method_span.1).is_none()
            {
                return Err("invalid external function container method span".into());
            }
            previous = method_span.1;
            method_spans.push(method_span);
            let mut return_end = method_span.0;
            for returned in list(method, "returns")? {
                let returned_span = span_key(&returned["span"])?;
                if returned_span.0 < method_span.0
                    || returned_span.0 < return_end
                    || returned_span.0 >= returned_span.1
                    || returned_span.1 > method_span.1
                    || raw.get(returned_span.0..returned_span.1).is_none()
                {
                    return Err("invalid function container return span".into());
                }
                return_end = returned_span.1;
            }
        }
        let valid = valid_type_name(name);
        let status = text(outcome, "status")?;
        if status == "emitted" {
            if !valid || !emitted.insert(name) {
                return Err("invalid emitted function container".into());
            }
            let expected = format!("---@class {name}\nlocal {name} = {{}}");
            if mapped.remove(&(path, header)) != Some(expected.as_str()) {
                return Err("missing or changed function container header".into());
            }
            let mut names = BTreeSet::new();
            for (method, method_span) in methods.iter().zip(&method_spans) {
                let method_name = text(method, "name")?;
                if !valid_identifier(method_name) || !names.insert(method_name) {
                    return Err("invalid emitted function container method".into());
                }
                let mut expected = String::new();
                for returned in list(method, "returns")? {
                    expected.push_str("---@return ");
                    expected.push_str(&lower_terms(list(returned, "terms")?)?);
                    expected.push('\n');
                    emitted_returns.insert((path, span_key(&returned["span"])?));
                }
                expected.push_str("function ");
                expected.push_str(name);
                expected.push(':');
                expected.push_str(method_name);
                expected.push_str("() end");
                if mapped.remove(&(path, *method_span)) != Some(expected.as_str()) {
                    return Err("missing or changed function container method".into());
                }
            }
        } else {
            if !matches!(
                status,
                "invalid_function_container_name"
                    | "duplicate_function_container"
                    | "source_name_conflict"
                    | "unsupported_function_container"
            ) || (status == "invalid_function_container_name" && valid)
                || mapped.contains_key(&(path, header))
                || method_spans
                    .iter()
                    .any(|method_span| mapped.contains_key(&(path, *method_span)))
            {
                return Err("invalid blocked function container".into());
            }
            require_issue(library, status, resource, &container["span"])?;
            blocked = true;
        }
    }
    let mut unresolved = BTreeSet::new();
    if report
        .get("unresolved_function_container_returns")
        .is_some()
    {
        let returns = list(report, "unresolved_function_container_returns")?;
        if returns.is_empty() || returns.len() > 65_536 {
            return Err("invalid unresolved function container return inventory".into());
        }
        for returned in returns {
            let path = text(returned, "path")?;
            let key = (path, span_key(&returned["span"])?);
            let resource = sources
                .get(path)
                .ok_or("unknown function container resource")?;
            if returned["scope"] != "annotation_alias_catalog"
                || returned["sha256"] != resource["sha256"]
                || !emitted_returns.contains(&key)
                || !unresolved.insert(key)
            {
                return Err("invalid unresolved function container return link".into());
            }
            require_issue(
                library,
                "unresolved_function_container_return_type",
                resource,
                &returned["span"],
            )?;
        }
        blocked = true;
    }
    for issue in list(library, "issues")? {
        if issue["code"] == "unresolved_function_container_return_type" {
            let source = &issue["source"];
            let path = text(source, "path")?;
            let resource = sources
                .get(path)
                .ok_or("unknown function container issue resource")?;
            if source["scope"] != "annotation_alias_catalog"
                || source["sha256"] != resource["sha256"]
                || !unresolved.contains(&(path, span_key(&source["span"])?))
            {
                return Err("unreported unresolved function container return".into());
            }
        }
    }
    Ok(blocked)
}

fn valid_type_name(name: &str) -> bool {
    valid_identifier(name)
        && !matches!(
            name,
            "bool"
                | "cstring"
                | "luaIndex"
                | "any"
                | "unknown"
                | "never"
                | "nil"
                | "boolean"
                | "number"
                | "integer"
                | "string"
                | "table"
                | "function"
                | "userdata"
                | "lightuserdata"
                | "thread"
        )
}

fn valid_identifier(name: &str) -> bool {
    if name.is_empty() || name.len() > 1024 || keyword(name) {
        return false;
    }
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn keyword(name: &str) -> bool {
    matches!(
        name,
        "and"
            | "break"
            | "do"
            | "else"
            | "elseif"
            | "end"
            | "false"
            | "for"
            | "function"
            | "goto"
            | "if"
            | "in"
            | "local"
            | "nil"
            | "not"
            | "or"
            | "repeat"
            | "return"
            | "then"
            | "true"
            | "until"
            | "while"
    )
}

fn require_issue(library: &Value, code: &str, resource: &Value, span: &Value) -> Result<()> {
    if !list(library, "issues")?.iter().any(|issue| {
        issue["code"] == code
            && issue["source"]["scope"] == "annotation_alias_catalog"
            && issue["source"]["path"] == resource["path"]
            && issue["source"]["sha256"] == resource["sha256"]
            && issue["source"]["span"] == *span
    }) {
        return Err("missing external function container issue".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        let raw = "---@class FunctionContainer\nlocal FunctionContainer = {}\n\n---@return boolean\nfunction FunctionContainer:IsCancelled() end\n";
        let header_end = raw.find("\n\n").unwrap_or(raw.len());
        let method_start = header_end + 2;
        let return_end = raw[method_start..]
            .find('\n')
            .map_or(raw.len(), |end| method_start + end);
        let output = raw.trim_end();
        let hash = format!("sha256:{}", crate::manifest::digest(raw.as_bytes()));
        let container_span = json!({"start":0,"end":raw.len()-1});
        let header_span = json!({"start":0,"end":header_end});
        let method_span = json!({"start":method_start,"end":raw.len()-1});
        let return_span = json!({"start":method_start,"end":return_end});
        let header_link = json!({"scope":"annotation_alias_catalog","path":"FunctionContainer.lua",
            "sha256":hash.clone(),"span":header_span.clone()});
        let method_link = json!({"scope":"annotation_alias_catalog","path":"FunctionContainer.lua",
            "sha256":hash.clone(),"span":method_span.clone()});
        json!({
            "schema":"wow-native-annotation-library/5",
            "issues":[],
            "aliases":{
                "schema":"wow-native-alias-projection/8",
                "authority":"external_annotation_overlay",
                "source":{
                    "schema":"wow-native-alias-resource/6",
                    "revision":"a".repeat(40),
                    "path":"FunctionContainer.lua",
                    "sha256":hash,
                    "source_bytes":raw.len(),
                    "text":raw,
                    "aliases":[],
                    "function_containers":[{
                        "name":"FunctionContainer",
                        "methods":[{
                            "name":"IsCancelled",
                            "returns":[{"terms":["boolean"],"span":return_span}],
                            "span":method_span.clone()
                        }],
                        "header_span":header_span.clone(),
                        "span":container_span
                    }]
                },
                "outcomes":[],
                "function_container_outcomes":[{
                    "ordinal":0,"name":"FunctionContainer","status":"emitted"
                }]
            },
            "files":[{"text":output,"mappings":[
                {"granularity":"declaration","generated":header_span,"source":header_link},
                {"granularity":"declaration","generated":method_span,"source":method_link}
            ]}]
        })
    }

    #[test]
    fn exact_static_class_and_method_bytes_are_required() -> Result<()> {
        let value = fixture();
        assert!(!super::super::verify(&value)?.blocked);
        for (pointer, replacement) in [
            ("/aliases/schema", json!("wow-native-alias-projection/7")),
            (
                "/aliases/source/schema",
                json!("wow-native-alias-resource/5"),
            ),
            ("/aliases/source/function_containers", json!([])),
            ("/aliases/function_container_outcomes", json!([])),
            (
                "/aliases/source/function_containers/0/methods/0/returns/0/terms",
                json!(["number"]),
            ),
            ("/files/0/text", json!("---@class Other")),
            ("/files/0/mappings", json!([])),
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(pointer).ok_or("mutation target")? = replacement;
            assert!(super::super::verify(&changed).is_err(), "{pointer}");
        }
        Ok(())
    }

    #[test]
    fn unresolved_return_requires_inventory_and_matching_issue() -> Result<()> {
        let mut value = fixture();
        value["aliases"]["source"]["function_containers"][0]["methods"][0]["returns"][0]["terms"] =
            json!(["Missing"]);
        value["files"][0]["text"] = json!(
            "---@class FunctionContainer\nlocal FunctionContainer = {}\n\n---@return Missing\nfunction FunctionContainer:IsCancelled() end"
        );
        let link = json!({
            "scope":"annotation_alias_catalog",
            "path":"FunctionContainer.lua",
            "sha256":value["aliases"]["source"]["sha256"],
            "span":value["aliases"]["source"]["function_containers"][0]["methods"][0]
                ["returns"][0]["span"]
        });
        value["aliases"]["unresolved_function_container_returns"] = json!([link.clone()]);
        value["issues"] = json!([{
            "code":"unresolved_function_container_return_type","source":link
        }]);
        assert!(super::super::verify(&value)?.blocked);
        value["issues"] = json!([]);
        assert!(super::super::verify(&value).is_err());
        Ok(())
    }
}
