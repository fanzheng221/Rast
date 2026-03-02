---
"@rust_ast/bindings": patch
"@rust_ast/unplugin": patch
"@rust_ast/mcp-server": patch
"@rust_ast/cli": patch
---

Improve npm package metadata and package pages:

- add per-package README files so npm pages render documentation
- add `license`, `repository`, `homepage`, `bugs`, and `keywords`
- set `publishConfig.access` to `public` for each package
