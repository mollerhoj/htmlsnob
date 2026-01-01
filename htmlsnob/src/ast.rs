#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Area {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Either<StringArea, TemplateExpression>,
    pub value: Option<AttributeValue>,
    pub area: Area,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringArea {
    pub content: String,
    pub area: Area,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeValue {
    pub start_quote: Option<char>,
    pub end_quote: Option<char>,
    pub parts: Vec<Either<StringArea, TemplateExpression>>,
    pub area: Area,
}

impl AttributeValue {
    // TODO Remove this helper method and use parts directly
    pub fn string_areas(&self) -> Vec<StringArea> {
        self.parts
            .iter()
            .filter_map(|either| match either {
                Either::Left(string_area) => Some(string_area.clone()),
                Either::Right(_) => None,
            })
            .collect()
    }
}

// TODO: Replace this with AttributeName and AttributeValue
#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn left(&self) -> Option<&L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    pub fn left_mut(&mut self) -> Option<&mut L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct OpenTag {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub area: Area,
    pub self_closed: bool,
    pub is_missing_end_bracket: bool, // Validated with missing_end_bracket_disallowed
    pub close_tag_index: Option<usize>, // TODO: Make rule
    pub index: usize,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct CloseTag {
    pub name: String,
    pub area: Area,
    pub is_missing_end_bracket: bool, // Validated with missing_end_bracket_disallowed
    pub open_tag_index: Option<usize>, // TODO: Make rule
}

#[derive(Debug, PartialEq, Clone)]
pub enum Construct {
    Statement,
    Expression,
    If,
    Else,
    EndIf,
    Loop,
    EndLoop,
    Switch,
    Case,
    EndSwitch,
    Block,
    EndBlock,
    Comment,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TemplateExpression {
    pub content: String,
    pub area: Area,
    pub kind: Construct,
    pub is_missing_end_bracket: bool, // Validated with missing_end_bracket_disallowed
}

#[derive(Debug, PartialEq, Clone)]
pub struct Doctype {
    pub content: String,
    pub area: Area,
    pub is_missing_end_bracket: bool, // Validated with missing_end_bracket_disallowed
}

#[derive(Debug, PartialEq, Clone)]
pub struct Comment {
    pub content: String,
    pub area: Area,
    pub is_missing_end_bracket: bool, // Validated with missing_end_bracket_disallowed
}

#[derive(Debug, PartialEq, Clone)]
pub struct Text {
    pub content: String,
    pub area: Area,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Doctype(Doctype),
    OpenTag(OpenTag),
    CloseTag(CloseTag),
    Text(Text),
    Comment(Comment),
    TemplateExpression(TemplateExpression),
}
