# @rust_ast/unplugin

Universal bundler plugin for Rast AST analysis (Vite/Webpack/Rollup via `unplugin`).

## Install

```bash
npm install -D @rust_ast/unplugin
```

## Vite Example

```ts
import { defineConfig } from "vite";
import rast from "@rust_ast/unplugin";

export default defineConfig({
  plugins: [
    rast.vite({
      query: "call_expression",
      action: {
        type: "replace",
        target: "callee",
        with: "bar",
      },
    }),
  ],
});
```

## Links

- Repository: <https://github.com/fanzheng221/Rast>
- Issues: <https://github.com/fanzheng221/Rast/issues>
