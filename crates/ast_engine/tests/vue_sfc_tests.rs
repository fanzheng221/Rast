use ast_engine::{
    compute_line_starts, offset_to_line_col, NodeSpan, VueSfcExtractor, VueSfcScriptKind,
};

#[test]
fn test_vue_sfc_extract_script_setup_and_offset_map() {
    let source = r#"<template>
  <div>{{ msg }}</div>
</template>
<script setup lang="ts">
const answer = 42;
</script>
<style scoped>
.app { color: red; }
</style>
"#;

    let extractor = VueSfcExtractor::new(source);
    let block = extractor.extract_script_block().unwrap();
    assert_eq!(block.kind, VueSfcScriptKind::ScriptSetup);
    assert!(block.content.contains("const answer = 42;"));
    assert!(block.block_presence.has_template);
    assert!(block.block_presence.has_style);
    assert!(block.block_presence.has_script_setup);

    let rel = block.content.find("answer").unwrap() as u32;
    let abs = block.offset_map.relative_to_absolute_offset(rel).unwrap();
    assert_eq!(&source[abs as usize..abs as usize + 6], "answer");
    assert_eq!(block.offset_map.absolute_to_relative_offset(abs), Some(rel));

    let rel_span = NodeSpan {
        start: rel,
        end: rel + 6,
    };
    let abs_span = block
        .offset_map
        .relative_to_absolute_span(rel_span)
        .unwrap();
    assert_eq!(
        &source[abs_span.start as usize..abs_span.end as usize],
        "answer"
    );
    assert_eq!(
        block.offset_map.absolute_to_relative_span(abs_span),
        Some(rel_span)
    );

    let expected = offset_to_line_col(abs as usize, &compute_line_starts(source));
    assert_eq!(
        block.offset_map.relative_offset_to_line_col(rel),
        Some(expected)
    );
}

#[test]
fn test_vue_sfc_extract_normal_script_and_presence() {
    let source = r#"<script lang="ts">
export const value = 1;
</script>
"#;

    let extractor = VueSfcExtractor::new(source);
    let block = extractor.extract_script_block().unwrap();
    assert_eq!(block.kind, VueSfcScriptKind::Script);
    assert!(block.content.contains("export const value = 1;"));

    let presence = extractor.block_presence();
    assert!(presence.has_script);
    assert!(!presence.has_script_setup);
    assert!(!presence.has_template);
    assert!(!presence.has_style);
}

#[test]
fn test_vue_sfc_identify_template_style_without_script() {
    let source = r#"<template><div>Only template</div></template>
<style>.only { color: blue; }</style>
"#;

    let extractor = VueSfcExtractor::new(source);
    assert!(extractor.extract_script_block().is_none());

    let presence = extractor.block_presence();
    assert!(!presence.has_script);
    assert!(!presence.has_script_setup);
    assert!(presence.has_template);
    assert!(presence.has_style);
}
