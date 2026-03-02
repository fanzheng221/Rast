use ast_engine::{MatchEnvironment, NodeSpan};
use serde::Serialize;
use serde_json::{Map, Value};

pub fn to_json_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|err| format!("{{\"error\":\"Failed to serialize result: {err}\"}}"))
}

pub fn to_napi_error(context: &str, err: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(format!("{context}: {err}"))
}

pub fn slice_span_text(source: &str, span: NodeSpan) -> napi::Result<String> {
    let start = span.start as usize;
    let end = span.end as usize;

    if end > source.len() || start > end {
        return Err(napi::Error::from_reason("Invalid match span"));
    }
    if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        return Err(napi::Error::from_reason("Match span is not UTF-8 boundary"));
    }

    Ok(source[start..end].to_string())
}

pub fn env_to_metavariables(env: &MatchEnvironment) -> Map<String, Value> {
    let mut vars = Map::new();

    for (name, captured) in &env.single_captures {
        vars.insert(name.clone(), Value::String(captured.text.clone()));
    }

    for (name, captured_nodes) in &env.multi_captures {
        if vars.contains_key(name) {
            continue;
        }
        let combined = captured_nodes
            .iter()
            .map(|node| node.text.as_str())
            .collect::<String>();
        vars.insert(name.clone(), Value::String(combined));
    }

    vars
}

#[derive(Serialize)]
pub struct SerializableSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Serialize)]
pub struct PatternMatchPayload {
    pub span: SerializableSpan,
    pub text: String,
    pub metavariables: Map<String, Value>,
}

#[derive(Serialize)]
pub struct SerializableLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Serialize)]
pub struct VueSfcScriptPayload {
    pub span: SerializableSpan,
    pub kind: String,
}

#[derive(Serialize)]
pub struct VueSfcPatternMatchPayload {
    pub relative_span: SerializableSpan,
    pub absolute_span: SerializableSpan,
    pub text: String,
    pub metavariables: Map<String, Value>,
    pub location: SerializableLocation,
}

#[derive(Serialize)]
pub struct VueSfcPatternSearchPayload {
    pub script: Option<VueSfcScriptPayload>,
    pub matches: Vec<VueSfcPatternMatchPayload>,
}

#[derive(Serialize)]
pub struct ScanResult {
    pub path: String,
    pub matches: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifications: Option<usize>,
}
