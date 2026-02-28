use ast_engine::{
    apply_span_replacements,
    matcher::{MatchStrictness, PatternMatcher},
    overlap_resolution::{ConflictResolution, FindAllMatches},
    wildcard_parsing::to_pattern_ast,
    IntoAstNode, NodeSpan, SpanReplacement, VueSfcExtractor,
};
use oxc::{allocator::Allocator, parser::Parser, span::SourceType};

/// Helper function to run a pattern match and replacement on a Vue SFC.
/// Returns the modified Vue SFC source code.
fn run_vue_sfc_replacement(
    vue_source: &str,
    pattern_source: &str,
    replacement_text: &str,
) -> String {
    let extractor = VueSfcExtractor::new(vue_source);
    let block = extractor
        .extract_script_block()
        .expect("Failed to extract script block from Vue SFC");

    let target_allocator = Allocator::default();
    let pattern_allocator = Allocator::default();

    // Use TypeScript source type for script setup
    let source_type = SourceType::default()
        .with_typescript(true)
        .with_module(true);

    let target_parsed = Parser::new(&target_allocator, block.content, source_type).parse();
    let pattern_parsed = Parser::new(&pattern_allocator, pattern_source, source_type).parse();

    let target_node = target_parsed.program.as_node(block.content);
    let pattern_node = to_pattern_ast(pattern_parsed.program.as_node(pattern_source));

    let matcher = PatternMatcher::new(MatchStrictness::Template);
    let matches =
        matcher.find_all_matches(target_node, &pattern_node, ConflictResolution::PreferOuter);

    let mut replacements = Vec::new();
    for m in matches {
        // Map the relative span in the script block back to the absolute span in the Vue SFC
        let abs_span = block
            .offset_map
            .relative_to_absolute_span(m.span)
            .expect("Failed to map relative span to absolute span");

        replacements.push(SpanReplacement::new(abs_span, replacement_text));
    }

    apply_span_replacements(vue_source, &replacements).expect("Failed to apply span replacements")
}

#[test]
fn test_vue_sfc_complex_structure_replacement() {
    let source = r#"<template>
  <div class="container">
    <h1>{{ title }}</h1>
    <button @click="handleClick">Click me</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const title = ref('Hello Vue');

function handleClick() {
  console.log('Button clicked');
}
</script>

<style scoped>
.container {
  padding: 20px;
}
</style>
"#;

    let pattern = "console.log('Button clicked');";
    let replacement = "console.info('Button was clicked!');";

    let result = run_vue_sfc_replacement(source, pattern, replacement);

    let expected = r#"<template>
  <div class="container">
    <h1>{{ title }}</h1>
    <button @click="handleClick">Click me</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const title = ref('Hello Vue');

function handleClick() {
  console.info('Button was clicked!');
}
</script>

<style scoped>
.container {
  padding: 20px;
}
</style>
"#;

    assert_eq!(
        result, expected,
        "Failed to replace console.log in complex Vue SFC"
    );
}

#[test]
fn test_vue_sfc_multiline_script_replacement() {
    let source = r#"<script setup>
const a = 1;
const b = 2;
const c = a + b;
</script>
<template>
  <div>{{ c }}</div>
</template>
"#;

    let pattern = "const b = 2;";
    let replacement = "const b = 42;";

    let result = run_vue_sfc_replacement(source, pattern, replacement);

    let expected = r#"<script setup>
const a = 1;
const b = 42;
const c = a + b;
</script>
<template>
  <div>{{ c }}</div>
</template>
"#;

    assert_eq!(
        result, expected,
        "Failed to replace multiline script content"
    );
}

#[test]
fn test_vue_sfc_special_characters_replacement() {
    let source = r#"<template>
  <div>Test</div>
</template>
<script>
export default {
  data() {
    return {
      msg: "Hello \"World\"",
      regex: /test\/ing/g
    };
  }
}
</script>
"#;

    let pattern = "msg: \"Hello \\\"World\\\"\"";
    let replacement = "msg: 'Hello Vue'";

    let result = run_vue_sfc_replacement(source, pattern, replacement);

    let expected = r#"<template>
  <div>Test</div>
</template>
<script>
export default {
  data() {
    return {
      msg: 'Hello Vue',
      regex: /test\/ing/g
    };
  }
}
</script>
"#;

    assert_eq!(
        result, expected,
        "Failed to replace content with special characters"
    );
}

#[test]
fn test_vue_sfc_multiple_matches_replacement() {
    let source = r#"<script setup lang="ts">
function foo() {
  console.log("foo");
}

function bar() {
  console.log("bar");
}
</script>
"#;

    // We want to replace all console.log calls
    let pattern = "console.log($$$ARGS);";
    let replacement = "logger.info($$$ARGS);";

    // Note: Our helper function doesn't support metavariable replacement yet,
    // so we'll just replace the exact match for now.
    // Let's write a custom test for this to handle metavariables if needed,
    // or just replace a fixed string.

    let pattern2 = "console.log";
    // Wait, pattern must be a valid AST node. "console.log" is an expression.
    // Let's use a full statement pattern.
}
