pub use crate::{
    find_all_matches, AllMatcher, AnyMatcher, CapturedNode, CompositeMatcher,
    ConflictResolution, FindAllMatches, MatchEnvironment, MatchOutcome, MatchStrictness,
    Matcher, NotMatcher, PatternMatcher,
};

pub type MatchResult = MatchOutcome;
pub type OverlapMatchResult = crate::MatchResult;
