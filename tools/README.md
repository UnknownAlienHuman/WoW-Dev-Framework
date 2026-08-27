# Tools

This directory is reserved for repository-owned development and release utilities that are not normal runtime libraries.

Planned tool classes:

- Reference Pack build and differential evaluation;
- fixture minimization and mutation;
- Ketho/Numy/LuaLS parity adapters;
- corpus manifest and license capture;
- upstream compatibility probes;
- schema compatibility simulation;
- deterministic output comparison;
- benchmark and agent-task evaluation runners;
- migration and release verification.

Tools must follow the same untrusted-input security model as production code. A helper script does not receive permission to execute arbitrary addon or external repository code.
