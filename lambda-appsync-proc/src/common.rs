use proc_macro2::Span;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CaseType {
    Camel,  // Lowercase separated by Uppercase letters, first letter lower
    Pascal, // Lowercase separated by Uppercase letters, first letter uppercase
    Snake,  // Lowercase separated by _
    Upper,  // All uppercase separated by _
}
impl CaseType {
    fn case(s: &str) -> Self {
        let mut chars = s.chars();
        if let Some(c) = chars.next() {
            if c.is_uppercase() {
                // Can only be Pascal or all upper
                for c in chars {
                    if !(c.is_uppercase() || c == '_') {
                        return Self::Pascal;
                    }
                }
                return Self::Upper;
            } else {
                // Can only be Camel or all lower
                for c in chars {
                    if !(c.is_lowercase() || c == '_') {
                        return Self::Camel;
                    }
                }
                return Self::Snake;
            }
        }
        panic!("Empty string has no case");
    }
}

// Word is always stored as lowercase
#[derive(Debug)]
pub(crate) struct Word(String);
impl Word {
    pub(crate) fn capitalize(&self) -> String {
        let mut chars = self.0.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        }
    }
    pub(crate) fn to_uppercase(&self) -> String {
        self.0.to_uppercase()
    }
}
impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::borrow::Borrow<str> for Word {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug)]
pub(crate) struct Name {
    orig: String,
    span: Span,
    words: Vec<Word>,
    name_override: Option<String>,
}
impl From<String> for Name {
    fn from(value: String) -> Self {
        Self::from((value, Span::call_site()))
    }
}
impl From<(String, Span)> for Name {
    fn from((orig, span): (String, Span)) -> Self {
        let mut words = vec![];
        match CaseType::case(orig.as_str()) {
            CaseType::Camel => {
                let mut slice_start = 0;
                for (i, c) in orig.char_indices() {
                    if c.is_uppercase() {
                        words.push(Word(orig[slice_start..i].to_lowercase()));
                        slice_start = i;
                    }
                }
                words.push(Word(orig[slice_start..].to_lowercase()));
            }
            CaseType::Pascal => {
                let mut slice_start = 0;
                for (i, c) in orig.char_indices() {
                    if c.is_uppercase() && i != 0 {
                        words.push(Word(orig[slice_start..i].to_lowercase()));
                        slice_start = i;
                    }
                }
                words.push(Word(orig[slice_start..].to_lowercase()));
            }
            CaseType::Snake => {
                words.extend(orig.split('_').map(|w| Word(w.to_owned())));
            }
            CaseType::Upper => {
                words.extend(orig.split('_').map(|w| Word(w.to_lowercase())));
            }
        };
        Name {
            orig,
            span,
            words,
            name_override: None,
        }
    }
}
impl Name {
    pub(crate) fn override_name(&mut self, name: String) {
        self.name_override.replace(name);
    }
    pub(crate) fn orig(&self) -> &str {
        &self.orig
    }
    // pub(crate) fn set_span(&mut self, span: Span) {
    //     self.span = Some(span);
    // }
    fn to_case(&self, case: CaseType) -> String {
        if let Some(ref name_override) = self.name_override {
            name_override.clone()
        } else {
            match case {
                CaseType::Camel => {
                    let mut word_iter = self.words.iter();
                    let first = word_iter.next().expect("non empty name");
                    Some(first)
                        .into_iter()
                        .chain(word_iter)
                        .map(|w| w.capitalize())
                        .collect()
                }
                CaseType::Pascal => self.words.iter().map(|w| w.capitalize()).collect(),
                CaseType::Snake => self.words.join("_"),
                CaseType::Upper => self
                    .words
                    .iter()
                    .map(|w| w.to_uppercase())
                    .collect::<Vec<_>>()
                    .join("_"),
            }
        }
    }
    pub(crate) fn to_type_ident(&self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(&self.to_case(CaseType::Pascal), self.span)
    }
    /// Converts the name to a valid Rust identifier in snake case, automatically escaping Rust keywords.
    ///
    /// # Note
    ///
    /// Most Rust keywords can be escaped using the `r#` prefix to make them valid identifiers.
    /// However, some keywords like `crate`, `self`, `super` cannot be escaped this way
    /// and will instead be prefixed with `r_` (e.g. `r_self`).
    pub(crate) fn to_var_ident(&self) -> proc_macro2::Ident {
        // List of Rust keywords that need escaping
        const RUST_KEYWORDS: &[&str] = &[
            // Keywords used in current Rust
            "as",
            "async",
            "await",
            "break",
            "const",
            "continue",
            "dyn",
            "else",
            "enum",
            "extern",
            "false",
            "fn",
            "for",
            "if",
            "impl",
            "in",
            "let",
            "loop",
            "match",
            "mod",
            "move",
            "mut",
            "pub",
            "ref",
            "return",
            "static",
            "struct",
            "trait",
            "true",
            "type",
            "unsafe",
            "use",
            "where",
            "while",
            // Reserved keywords
            "abstract",
            "become",
            "box",
            "do",
            "final",
            "macro",
            "override",
            "priv",
            "try",
            "gen",
            "typeof",
            "unsized",
            "virtual",
            "yield",
            // Weak keywords
            "macro_rules",
            "union",
            "'static",
            "safe",
            "raw",
        ];
        const RUST_INESCAPABLE_KEYWORDS: &[&str] = &[
            // Keywords that cannot be escaped with r#
            "crate", "self", "super",
        ];
        let ident_str = self.to_case(CaseType::Snake);

        if self.words.len() == 1 && RUST_INESCAPABLE_KEYWORDS.contains(&ident_str.as_str()) {
            proc_macro2::Ident::new(&format!("r_{}", ident_str), self.span)
        } else if self.words.len() == 1 && RUST_KEYWORDS.contains(&ident_str.as_str()) {
            proc_macro2::Ident::new_raw(&ident_str, self.span)
        } else {
            proc_macro2::Ident::new(&ident_str, self.span)
        }
    }
    pub(crate) fn to_prefixed_fct_ident(&self, prefix: &str) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &format!("{prefix}_{}", self.to_case(CaseType::Snake)),
            self.span,
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum OperationKind {
    Query,
    Mutation,
    Subscription,
}
impl core::fmt::Display for OperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationKind::Query => write!(f, "Query"),
            OperationKind::Mutation => write!(f, "Mutation"),
            OperationKind::Subscription => write!(f, "Subscription"),
        }
    }
}
impl OperationKind {
    pub(crate) fn fct_prefix(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        }
    }
    pub(crate) fn module_name(self) -> &'static str {
        match self {
            Self::Query => "queries",
            Self::Mutation => "mutations",
            Self::Subscription => "subscriptions",
        }
    }
    pub(crate) fn operation_enum_name(self, span: Span) -> proc_macro2::Ident {
        match self {
            Self::Query => proc_macro2::Ident::new("QueryField", span),
            Self::Mutation => proc_macro2::Ident::new("MutationField", span),
            Self::Subscription => proc_macro2::Ident::new("SubscriptionField", span),
        }
    }
}
