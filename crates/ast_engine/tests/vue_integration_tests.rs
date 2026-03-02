use ast_engine::{
    apply_span_replacements, default_source_type,
    matcher::PatternMatcher,
    overlap_resolution::{ConflictResolution, FindAllMatches},
    parse_pattern_ast, IntoAstNode, SpanReplacement, VueSfcExtractor,
};
use oxc::{allocator::Allocator, parser::Parser};

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

    let allocator = Allocator::default();
    let source_type = default_source_type();
    let parsed_source = Parser::new(&allocator, block.content, source_type).parse();
    assert!(
        parsed_source.errors.is_empty(),
        "Target script parse failed: {:?}",
        parsed_source.errors
    );

    let pattern_node =
        parse_pattern_ast(pattern_source, source_type).expect("Failed to parse pattern AST");
    let matcher = PatternMatcher::default();
    let matches = matcher.find_all_matches(
        parsed_source.program.as_node(block.content),
        &pattern_node,
        ConflictResolution::PreferOuter,
    );

    let mut replacements = Vec::new();
    for matched in matches {
        let abs_span = block
            .offset_map
            .relative_to_absolute_span(matched.span)
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

    let pattern = "const title = ref('Hello Vue');";
    let replacement = "const title = ref('Hello Rast');";

    let result = run_vue_sfc_replacement(source, pattern, replacement);

    let expected = r#"<template>
  <div class="container">
    <h1>{{ title }}</h1>
    <button @click="handleClick">Click me</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const title = ref('Hello Rast');

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
const msg = "Hello \"World\"";
const regex = /test\/ing/g;
</script>
"#;

    let pattern = r#"const msg = "Hello \"World\"";"#;
    let replacement = "const msg = 'Hello Vue';";

    let result = run_vue_sfc_replacement(source, pattern, replacement);

    let expected = r#"<template>
  <div>Test</div>
</template>
<script>
const msg = 'Hello Vue';
const regex = /test\/ing/g;
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
console.log("foo");
console.log("bar");
</script>
"#;

    let pattern = "console.log(\"foo\");";
    let replacement = "logger.info(\"foo\");";

    let result = run_vue_sfc_replacement(source, pattern, replacement);
    assert!(
        result.contains("logger.info(\"foo\");"),
        "Expected first console.log call to be replaced"
    );
}
