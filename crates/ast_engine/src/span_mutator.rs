use crate::NodeSpan;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanReplacement {
    pub span: NodeSpan,
    pub replacement: String,
}

impl SpanReplacement {
    pub fn new(span: NodeSpan, replacement: impl Into<String>) -> Self {
        Self {
            span,
            replacement: replacement.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDiff {
    pub span: NodeSpan,
    pub original: String,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanMutatorError {
    InvalidSpan { start: u32, end: u32 },
    OutOfBounds { span: NodeSpan, text_len: usize },
    InvalidCharBoundary { span: NodeSpan },
    OverlappingSpans { left: NodeSpan, right: NodeSpan },
    SourceMismatch { span: NodeSpan },
}

impl fmt::Display for SpanMutatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpan { start, end } => {
                write!(f, "Invalid span range: start ({start}) > end ({end})")
            }
            Self::OutOfBounds { span, text_len } => {
                write!(
                    f,
                    "Span [{}, {}) is out of bounds for source length {}",
                    span.start, span.end, text_len
                )
            }
            Self::InvalidCharBoundary { span } => {
                write!(
                    f,
                    "Span [{}, {}) does not align to UTF-8 char boundary",
                    span.start, span.end
                )
            }
            Self::OverlappingSpans { left, right } => {
                write!(
                    f,
                    "Overlapping spans detected: [{}, {}) and [{}, {})",
                    left.start, left.end, right.start, right.end
                )
            }
            Self::SourceMismatch { span } => {
                write!(
                    f,
                    "Source text does not match expected original content at span [{}, {})",
                    span.start, span.end
                )
            }
        }
    }
}

impl Error for SpanMutatorError {}

pub fn generate_text_diffs(
    source: &str,
    replacements: &[SpanReplacement],
) -> Result<Vec<TextDiff>, SpanMutatorError> {
    validate_replacements(source, replacements)?;

    replacements
        .iter()
        .map(|replacement| {
            let span = replacement.span;
            let start = span.start as usize;
            let end = span.end as usize;
            Ok(TextDiff {
                span,
                original: source[start..end].to_string(),
                replacement: replacement.replacement.clone(),
            })
        })
        .collect()
}

pub fn apply_span_replacements(
    source: &str,
    replacements: &[SpanReplacement],
) -> Result<String, SpanMutatorError> {
    let diffs = generate_text_diffs(source, replacements)?;
    apply_text_diffs(source, &diffs)
}

pub fn apply_text_diffs(source: &str, diffs: &[TextDiff]) -> Result<String, SpanMutatorError> {
    validate_diffs(source, diffs)?;

    let mut result = source.to_string();
    let mut ordered: Vec<&TextDiff> = diffs.iter().collect();
    ordered.sort_by(|left, right| {
        right
            .span
            .end
            .cmp(&left.span.end)
            .then_with(|| right.span.start.cmp(&left.span.start))
    });

    for diff in ordered {
        let start = diff.span.start as usize;
        let end = diff.span.end as usize;
        result.replace_range(start..end, &diff.replacement);
    }

    Ok(result)
}

fn validate_diffs(source: &str, diffs: &[TextDiff]) -> Result<(), SpanMutatorError> {
    let spans = diffs.iter().map(|diff| diff.span).collect::<Vec<_>>();
    validate_spans(source, &spans)?;

    for diff in diffs {
        let start = diff.span.start as usize;
        let end = diff.span.end as usize;
        if source[start..end] != diff.original {
            return Err(SpanMutatorError::SourceMismatch { span: diff.span });
        }
    }

    Ok(())
}

fn validate_replacements(source: &str, replacements: &[SpanReplacement]) -> Result<(), SpanMutatorError> {
    let spans = replacements
        .iter()
        .map(|replacement| replacement.span)
        .collect::<Vec<_>>();
    validate_spans(source, &spans)
}

fn validate_spans(source: &str, spans: &[NodeSpan]) -> Result<(), SpanMutatorError> {
    let text_len = source.len();

    for span in spans {
        if span.start > span.end {
            return Err(SpanMutatorError::InvalidSpan {
                start: span.start,
                end: span.end,
            });
        }

        let start = span.start as usize;
        let end = span.end as usize;

        if end > text_len {
            return Err(SpanMutatorError::OutOfBounds {
                span: *span,
                text_len,
            });
        }

        if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
            return Err(SpanMutatorError::InvalidCharBoundary { span: *span });
        }
    }

    let mut ordered = spans.to_vec();
    ordered.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.end.cmp(&right.end))
    });

    for window in ordered.windows(2) {
        let left = window[0];
        let right = window[1];
        if spans_overlap(left, right) {
            return Err(SpanMutatorError::OverlappingSpans { left, right });
        }
    }

    Ok(())
}

fn spans_overlap(left: NodeSpan, right: NodeSpan) -> bool {
    left.start < right.end && right.start < left.end
}
