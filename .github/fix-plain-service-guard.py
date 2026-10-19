from pathlib import Path

path = Path('.github/plain-service-results.py')
text = path.read_text()
old = '''    if text.count(field) != 2:
        raise SystemExit(f"unexpected Service result field count for {field!r}: {text.count(field)}")
    text = text.replace(field, "")
'''
new = '''    occurrences = text.count(field)
    if occurrences not in (2, 4):
        raise SystemExit(f"unexpected Service result field count for {field!r}: {occurrences}")
    text = text.replace(field, "")
'''
if text.count(old) != 1:
    raise SystemExit('unexpected result-field guard')
path.write_text(text.replace(old, new))
