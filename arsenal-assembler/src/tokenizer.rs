use regex::Regex;

#[derive(Debug, Clone)]
pub enum ArsenalToken {
    Whitespace(String),
    LineEnd(String),
    Separator(String),
    Identifier(String),
    Hex(String),
    Number(String),
    StringLiteral(String),
    Label(String),
    SpecialIdentifier(String),
    IDGrab(String),
    SizeGrab(String),
    Selection(String),
    Range(String),
    Shift(String),
    VarAssignment(String),
    OpenParen(String),
    ClosedParen(String),
    Comment(String),
    NumericSlice(String),
}

pub trait Token {
    fn next(data: &str) -> Option<(Self, &str)> where Self: Sized;
    fn is_whitespace(&self) -> bool;
}

impl Token for ArsenalToken {
    fn next(data: &str) -> Option<(Self, &str)> {
        use ArsenalToken::*;
        for (pattern, response) in [
            ("\\s+", Whitespace as fn(String) -> Self),
            (";", LineEnd),
            (",", Separator),
            ("->", Range),
            ("\\b0[xX][0-9A-Fa-f]+\\b", Hex),
            (r"0|([1-9]([0-9]+)?)", Number),
            ("label", Label),
            ("^\\.[a-zA-Z_][a-zA-Z0-9_]*", SpecialIdentifier),
            ("^[a-zA-Z_][a-zA-Z0-9_]*", Identifier),
            ("&", IDGrab),
            ("\\$", SizeGrab),
            (":", Selection),
            ("=>", Shift),
            ("=", VarAssignment),
            ("\\(", OpenParen),
            ("\\)", ClosedParen),
            (r#""(?:\\.|[^\\"])*""#, StringLiteral),
            (r"//.[^\n]*\n", Comment),
            (r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", Comment),
            ("#", NumericSlice),
        ] {
            if let Some((pat, length)) = parse_pattern(pattern, data) {
                return Some((response(pat), &data[length..]));
            }
        }
        return None;
    }

    fn is_whitespace(&self) -> bool {
        match self {
            Self::Whitespace(_) => true,
            Self::Comment(_) => true,
            _ => false,
        }
    }
}

pub fn tokenize<T: Token + std::fmt::Debug>(input: &str) -> Vec<T> {
    let mut string = input;
    let mut ret = vec![];
    let mut last = ".".to_string();
    while let Some((next_token, next_string)) = T::next(string) {
        last = string.clone().to_string();
        string = next_string;
        ret.push(next_token);
    }
    let mut return_value = vec![];
    for elm in ret {
        if !elm.is_whitespace() {
            return_value.push(elm);
        }
    }
    return_value
}

fn parse_pattern(pattern: &str, text: &str) -> Option<(String, usize)> {
    let regex = Regex::new(pattern).unwrap();
    if let Some(mat) = regex.find(text) {
        if mat.start() == 0 {
            let matched_text = mat.as_str().to_string();
            let length = mat.end();
            return Some((matched_text, length));
        }
    }
    None
}