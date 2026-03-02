use serde::{Deserialize, Serialize};

/// `sgconfig.yml` root schema for a single Rast rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCore {
    pub id: String,
    pub language: RuleLanguage,
    pub rule: Rule,
}

/// Alias for the canonical `sgconfig.yml` payload.
pub type SgConfig = RuleCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleLanguage {
    Js,
    Jsx,
    Ts,
    Tsx,
    Javascript,
    Typescript,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rule {
    pub core: RuleKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleKind {
    Pattern(PatternAtomicRule),
    Regex(RegexAtomicRule),
    Kind(KindAtomicRule),
    All(AllCompositeRule),
    Any(AnyCompositeRule),
    Not(NotCompositeRule),
    Inside(InsideRelationalRule),
    Has(HasRelationalRule),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternAtomicRule {
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegexAtomicRule {
    pub regex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindAtomicRule {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllCompositeRule {
    pub all: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnyCompositeRule {
    pub any: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotCompositeRule {
    pub not: Box<Rule>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsideRelationalRule {
    pub inside: Box<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HasRelationalRule {
    pub has: Box<Rule>,
}

impl RuleCore {
    pub fn from_yaml(input: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(input)
    }
}
