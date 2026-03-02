use ast_engine::{
    default_source_type, parse_pattern_ast, ConflictResolution, IntoAstNode, Matcher,
    PatternMatcher, VueSfcScriptKind,
};
use napi_derive::napi;
use oxc::{allocator::Allocator, parser::Parser};

use crate::serde_payload::{
    env_to_metavariables, slice_span_text, to_json_string, to_napi_error, PatternMatchPayload,
    SerializableLocation, SerializableSpan, VueSfcPatternMatchPayload, VueSfcPatternSearchPayload,
    VueSfcScriptPayload,
};

#[napi(js_name = "find_pattern")]
pub fn find_pattern(source: String, pattern: String) -> napi::Result<String> {
    let allocator = Allocator::default();
    let source_type = default_source_type();

    let parsed_source = Parser::new(&allocator, &source, source_type).parse();
    if let Some(err) = parsed_source.errors.first() {
        return Err(to_napi_error("Invalid source", err));
    }

    let pattern_ast = parse_pattern_ast(&pattern, source_type)
        .map_err(|err| to_napi_error("Invalid pattern", err))?;
    let matcher = PatternMatcher::default();
    let matches = matcher.find_all_matches(
        parsed_source.program.as_node(&source),
        &pattern_ast,
        ConflictResolution::PreferOuter,
    );

    let mut payload = Vec::with_capacity(matches.len());
    for matched in matches {
        payload.push(PatternMatchPayload {
            span: SerializableSpan {
                start: matched.span.start,
                end: matched.span.end,
            },
            text: slice_span_text(&source, matched.span)?,
            metavariables: env_to_metavariables(&matched.environment),
        });
    }

    Ok(to_json_string(&payload))
}

#[napi(js_name = "find_pattern_in_vue_sfc")]
pub fn find_pattern_in_vue_sfc(source: String, pattern: String) -> napi::Result<String> {
    let extractor = ast_engine::VueSfcExtractor::new(&source);
    let Some(block) = extractor.extract_script_block() else {
        return Ok(to_json_string(&VueSfcPatternSearchPayload {
            script: None,
            matches: Vec::new(),
        }));
    };

    let allocator = Allocator::default();
    let source_type = default_source_type();
    let parsed_source = Parser::new(&allocator, block.content, source_type).parse();
    if let Some(err) = parsed_source.errors.first() {
        return Err(to_napi_error("Invalid Vue SFC script block", err));
    }

    let pattern_ast = parse_pattern_ast(&pattern, source_type)
        .map_err(|err| to_napi_error("Invalid pattern", err))?;
    let matcher = PatternMatcher::default();
    let matches = matcher.find_all_matches(
        parsed_source.program.as_node(block.content),
        &pattern_ast,
        ConflictResolution::PreferOuter,
    );

    let script_span = block.offset_map.script_span();
    let script_payload = VueSfcScriptPayload {
        span: SerializableSpan {
            start: script_span.start,
            end: script_span.end,
        },
        kind: match block.kind {
            VueSfcScriptKind::Script => "script".to_string(),
            VueSfcScriptKind::ScriptSetup => "scriptSetup".to_string(),
        },
    };

    let mut payload = Vec::with_capacity(matches.len());
    for matched in matches {
        let absolute_span = block
            .offset_map
            .relative_to_absolute_span(matched.span)
            .ok_or_else(|| {
                napi::Error::from_reason("Failed to map relative span to absolute Vue SFC offset")
            })?;
        let (line, column) = block
            .offset_map
            .absolute_offset_to_line_col(absolute_span.start);
        payload.push(VueSfcPatternMatchPayload {
            relative_span: SerializableSpan {
                start: matched.span.start,
                end: matched.span.end,
            },
            absolute_span: SerializableSpan {
                start: absolute_span.start,
                end: absolute_span.end,
            },
            text: slice_span_text(&source, absolute_span)?,
            metavariables: env_to_metavariables(&matched.environment),
            location: SerializableLocation { line, column },
        });
    }

    Ok(to_json_string(&VueSfcPatternSearchPayload {
        script: Some(script_payload),
        matches: payload,
    }))
}
