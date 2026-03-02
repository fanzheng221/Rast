# @rust_ast/bindings

NAPI bindings for the Rast AST engine.

## Install

```bash
npm install @rust_ast/bindings
```

## Usage

```js
const rast = require("@rust_ast/bindings");

const rule = {
  query: "call_expression",
  action: {
    type: "replace",
    target: "callee",
    with: "bar",
  },
};

const output = rast.apply_rule('foo("hello")', rule);
console.log(output);
```

## Links

- Repository: <https://github.com/fanzheng221/Rast>
- Issues: <https://github.com/fanzheng221/Rast/issues>
