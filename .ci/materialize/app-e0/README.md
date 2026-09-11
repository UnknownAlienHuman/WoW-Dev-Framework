# `wow`

`wow` is the bounded one-shot E0 command-line entry point over `wow-service`.

```text
wow operations
wow rules-evaluate --input rule-input.json
cat rule-input.json | wow rules-evaluate --input -
```

`rules-evaluate` accepts one strict JSON `wow_rules::RuleEvaluationInput`, constructs one immutable service snapshot, executes `rules.evaluate@1`, and writes one canonical JSON response to stdout. It reads at most 16 MiB and performs no discovery, network access, source execution, editor mutation, or persistent update.

Exit codes:

- `0`: completed without diagnostics;
- `1`: completed with rule diagnostics;
- `2`: invalid invocation/input or structural service error;
- `3`: registered-operation request rejected.

The CLI does not infer a current WoW client or Reference view. Input identities and evidence must already be explicit and mutually compatible.
