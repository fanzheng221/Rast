use crate::{AstNode, MatchEnvironment, Matcher, NodeSpan, NodeTrait, PatternNode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConflictResolution {
    #[default]
    PreferOuter,
    PreferInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchResult {
    pub span: NodeSpan,
    pub environment: MatchEnvironment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatchCandidate {
    result: MatchResult,
    depth: usize,
    visit_index: usize,
}

fn spans_overlap(left: NodeSpan, right: NodeSpan) -> bool {
    left.start < right.end && right.start < left.end
}

fn span_len(span: NodeSpan) -> u32 {
    span.end.saturating_sub(span.start)
}

fn sort_candidates(candidates: &mut [MatchCandidate], conflict_resolution: ConflictResolution) {
    candidates.sort_by(|left, right| {
        let left_len = span_len(left.result.span);
        let right_len = span_len(right.result.span);

        let length_order = match conflict_resolution {
            ConflictResolution::PreferOuter => right_len.cmp(&left_len),
            ConflictResolution::PreferInner => left_len.cmp(&right_len),
        };

        length_order
            .then_with(|| left.result.span.start.cmp(&right.result.span.start))
            .then_with(|| left.result.span.end.cmp(&right.result.span.end))
            .then_with(|| left.depth.cmp(&right.depth))
            .then_with(|| left.visit_index.cmp(&right.visit_index))
    });
}

pub fn find_all_matches<'a, M: Matcher + ?Sized>(
    matcher: &M,
    target: AstNode<'a>,
    pattern: &PatternNode,
    conflict_resolution: ConflictResolution,
) -> Vec<MatchResult> {
    fn collect_matches<'a, M: Matcher + ?Sized>(
        matcher: &M,
        node: AstNode<'a>,
        pattern: &PatternNode,
        depth: usize,
        visit_index: &mut usize,
        candidates: &mut Vec<MatchCandidate>,
    ) {
        if let Some(environment) = matcher.match_node(node, pattern) {
            candidates.push(MatchCandidate {
                result: MatchResult {
                    span: node.span(),
                    environment,
                },
                depth,
                visit_index: *visit_index,
            });
            *visit_index += 1;
        }

        for child in node.children() {
            collect_matches(matcher, child, pattern, depth + 1, visit_index, candidates);
        }
    }

    let mut candidates = Vec::new();
    let mut visit_index = 0;
    collect_matches(
        matcher,
        target,
        pattern,
        0,
        &mut visit_index,
        &mut candidates,
    );

    sort_candidates(&mut candidates, conflict_resolution);

    let mut selected = Vec::new();
    for candidate in candidates {
        if selected.iter().all(|existing: &MatchCandidate| {
            !spans_overlap(existing.result.span, candidate.result.span)
        }) {
            selected.push(candidate);
        }
    }

    selected.sort_by(|left, right| {
        left.result
            .span
            .start
            .cmp(&right.result.span.start)
            .then_with(|| left.result.span.end.cmp(&right.result.span.end))
            .then_with(|| left.depth.cmp(&right.depth))
            .then_with(|| left.visit_index.cmp(&right.visit_index))
    });
    selected.dedup_by(|left, right| left.result == right.result);

    selected
        .into_iter()
        .map(|candidate| candidate.result)
        .collect()
}

pub trait FindAllMatches {
    fn find_all_matches<'a>(
        &self,
        target: AstNode<'a>,
        pattern: &PatternNode,
        conflict_resolution: ConflictResolution,
    ) -> Vec<MatchResult>;
}

impl<T: Matcher + ?Sized> FindAllMatches for T {
    fn find_all_matches<'a>(
        &self,
        target: AstNode<'a>,
        pattern: &PatternNode,
        conflict_resolution: ConflictResolution,
    ) -> Vec<MatchResult> {
        find_all_matches(self, target, pattern, conflict_resolution)
    }
}

#[allow(non_snake_case)]
pub fn FindAllMatches<'a, M: Matcher + ?Sized>(
    matcher: &M,
    target: AstNode<'a>,
    pattern: &PatternNode,
    conflict_resolution: ConflictResolution,
) -> Vec<MatchResult> {
    find_all_matches(matcher, target, pattern, conflict_resolution)
}
