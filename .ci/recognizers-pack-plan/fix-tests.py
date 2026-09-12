from pathlib import Path

path = Path("crates/wow-recognizers/tests/pack_plan.rs")
text = path.read_text(encoding="utf-8")
replacements = {
    '    assert_eq!(rule.capture_names(), ["member"]);\n': (
        '    assert_eq!(rule.capture_names().len(), 1);\n'
        '    assert_eq!(rule.capture_names()[0].as_ref(), "member");\n'
    ),
    '    assert_eq!(rule.output_ids(), ["called_member"]);\n': (
        '    assert_eq!(rule.output_ids().len(), 1);\n'
        '    assert_eq!(rule.output_ids()[0].as_ref(), "called_member");\n'
    ),
    '        .find(|step| step.required_capabilities() == ["reference.api.complete"])\n': (
        '        .find(|step| {\n'
        '            step.required_capabilities().len() == 1\n'
        '                && step.required_capabilities()[0].as_ref() == "reference.api.complete"\n'
        '        })\n'
    ),
}
for old, new in replacements.items():
    if text.count(old) != 1:
        raise SystemExit(f"expected one replacement target: {old!r}")
    text = text.replace(old, new, 1)
path.write_text(text, encoding="utf-8")
