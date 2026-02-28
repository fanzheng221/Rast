use crate::{compute_line_starts, offset_to_line_col, NodeSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VueSfcScriptKind {
    Script,
    ScriptSetup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VueSfcBlockPresence {
    pub has_script: bool,
    pub has_script_setup: bool,
    pub has_template: bool,
    pub has_style: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueSfcOffsetMap {
    script_span: NodeSpan,
    relative_to_absolute_offsets: Vec<u32>,
    line_starts: Vec<usize>,
}

impl VueSfcOffsetMap {
    pub(crate) fn new(script_span: NodeSpan, source: &str) -> Self {
        let start = script_span.start;
        let end = script_span.end;
        let relative_to_absolute_offsets = (start..=end).collect::<Vec<_>>();
        Self {
            script_span,
            relative_to_absolute_offsets,
            line_starts: compute_line_starts(source),
        }
    }

    pub fn script_span(&self) -> NodeSpan {
        self.script_span
    }

    pub fn relative_to_absolute_offset(&self, relative_offset: u32) -> Option<u32> {
        self.relative_to_absolute_offsets
            .get(relative_offset as usize)
            .copied()
    }

    pub fn absolute_to_relative_offset(&self, absolute_offset: u32) -> Option<u32> {
        if absolute_offset < self.script_span.start || absolute_offset > self.script_span.end {
            return None;
        }
        Some(absolute_offset - self.script_span.start)
    }

    pub fn relative_to_absolute_span(&self, relative_span: NodeSpan) -> Option<NodeSpan> {
        let start = self.relative_to_absolute_offset(relative_span.start)?;
        let end = self.relative_to_absolute_offset(relative_span.end)?;
        Some(NodeSpan { start, end })
    }

    pub fn absolute_to_relative_span(&self, absolute_span: NodeSpan) -> Option<NodeSpan> {
        let start = self.absolute_to_relative_offset(absolute_span.start)?;
        let end = self.absolute_to_relative_offset(absolute_span.end)?;
        Some(NodeSpan { start, end })
    }

    pub fn absolute_offset_to_line_col(&self, absolute_offset: u32) -> (usize, usize) {
        offset_to_line_col(absolute_offset as usize, &self.line_starts)
    }

    pub fn relative_offset_to_line_col(&self, relative_offset: u32) -> Option<(usize, usize)> {
        let absolute = self.relative_to_absolute_offset(relative_offset)?;
        Some(self.absolute_offset_to_line_col(absolute))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedScriptBlock<'a> {
    pub content: &'a str,
    pub kind: VueSfcScriptKind,
    pub offset_map: VueSfcOffsetMap,
    pub block_presence: VueSfcBlockPresence,
}

#[derive(Debug, Clone)]
pub struct VueSfcExtractor<'a> {
    source: &'a str,
    block_presence: VueSfcBlockPresence,
}

impl<'a> VueSfcExtractor<'a> {
    pub fn new(source: &'a str) -> Self {
        let script_blocks = collect_sfc_script_blocks(source);
        let has_script = script_blocks
            .iter()
            .any(|block| block.kind == VueSfcScriptKind::Script);
        let has_script_setup = script_blocks
            .iter()
            .any(|block| block.kind == VueSfcScriptKind::ScriptSetup);
        let has_template = find_sfc_block(source, "template", 0).is_some();
        let has_style = find_sfc_block(source, "style", 0).is_some();

        Self {
            source,
            block_presence: VueSfcBlockPresence {
                has_script,
                has_script_setup,
                has_template,
                has_style,
            },
        }
    }

    pub fn block_presence(&self) -> VueSfcBlockPresence {
        self.block_presence
    }

    pub fn extract_script_block(&self) -> Option<ExtractedScriptBlock<'a>> {
        let block = collect_sfc_script_blocks(self.source).into_iter().next()?;
        let script_span = NodeSpan {
            start: block.content_start as u32,
            end: block.content_end as u32,
        };
        let content = &self.source[block.content_start..block.content_end];

        Some(ExtractedScriptBlock {
            content,
            kind: block.kind,
            offset_map: VueSfcOffsetMap::new(script_span, self.source),
            block_presence: self.block_presence,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SfcBlockMatch {
    content_start: usize,
    content_end: usize,
    kind: VueSfcScriptKind,
}

fn collect_sfc_script_blocks(source: &str) -> Vec<SfcBlockMatch> {
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    while let Some(block) = find_sfc_block(source, "script", cursor) {
        let next_cursor = block.content_end.saturating_add(1);
        blocks.push(block);
        if next_cursor > source.len() {
            break;
        }
        cursor = next_cursor;
    }
    blocks
}

fn find_sfc_block(source: &str, tag_name: &str, from_offset: usize) -> Option<SfcBlockMatch> {
    let lower_source = source.to_ascii_lowercase();
    let lower_tag_name = tag_name.to_ascii_lowercase();
    let open_tag_prefix = format!("<{}", lower_tag_name);
    let close_tag = format!("</{}>", lower_tag_name);

    let mut cursor = from_offset;
    while cursor < lower_source.len() {
        let open_start = lower_source[cursor..].find(&open_tag_prefix)? + cursor;
        let boundary_idx = open_start + open_tag_prefix.len();
        if !is_sfc_tag_boundary(lower_source.as_bytes().get(boundary_idx).copied()) {
            cursor = boundary_idx;
            continue;
        }

        let open_end = lower_source[boundary_idx..].find('>')? + boundary_idx;
        let content_start = open_end + 1;
        let close_start = lower_source[content_start..].find(&close_tag)? + content_start;
        let open_tag_content = &source[open_start..=open_end];

        let kind = if lower_tag_name == "script" && tag_has_attribute(open_tag_content, "setup") {
            VueSfcScriptKind::ScriptSetup
        } else {
            VueSfcScriptKind::Script
        };

        return Some(SfcBlockMatch {
            content_start,
            content_end: close_start,
            kind,
        });
    }
    None
}

fn is_sfc_tag_boundary(ch: Option<u8>) -> bool {
    matches!(
        ch,
        None | Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b'\t')
    )
}

fn tag_has_attribute(open_tag: &str, attr_name: &str) -> bool {
    let lower = open_tag.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut idx = 1usize;

    while idx < bytes.len() && bytes[idx].is_ascii_alphabetic() {
        idx += 1;
    }

    while idx < bytes.len() {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] == b'>' || bytes[idx] == b'/' {
            break;
        }

        let attr_start = idx;
        while idx < bytes.len()
            && !bytes[idx].is_ascii_whitespace()
            && bytes[idx] != b'='
            && bytes[idx] != b'>'
            && bytes[idx] != b'/'
        {
            idx += 1;
        }

        if &lower[attr_start..idx] == attr_name {
            return true;
        }

        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }

        if idx < bytes.len() && bytes[idx] == b'=' {
            idx += 1;
            while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }

            if idx < bytes.len() && (bytes[idx] == b'"' || bytes[idx] == b'\'') {
                let quote = bytes[idx];
                idx += 1;
                while idx < bytes.len() && bytes[idx] != quote {
                    idx += 1;
                }
                if idx < bytes.len() {
                    idx += 1;
                }
            } else {
                while idx < bytes.len() && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b'>' {
                    idx += 1;
                }
            }
        }
    }

    false
}
