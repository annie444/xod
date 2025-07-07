use super::{RefSpan, Span};
use crate::bitops::BitOps;
use std::{collections::VecDeque, fmt};

pub struct Ast<'a> {
    pub body: &'static str,
    pub lines: VecDeque<Line<'a>>,
}

impl<'a> Ast<'a> {
    pub fn new(body: &'static str, lines: VecDeque<Line<'a>>) -> Self {
        Self { body, lines }
    }

    pub fn add_line(&mut self, line: Line<'a>) {
        self.lines.push_back(line);
    }

    pub fn get_line(&mut self) -> Option<Line<'a>> {
        self.lines.pop_front()
    }
}

// (Variabl, Method, Opt<VarNum>)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method<'a> {
    Append(Span<'a>, Span<'a>, VarNum<'a>),
    Prepend(Span<'a>, Span<'a>, VarNum<'a>),
    Front(Span<'a>, Span<'a>),
    Back(Span<'a>, Span<'a>),
    Index(Span<'a>, Span<'a>, VarNum<'a>),
}

impl<'a> RefSpan<'a> for Method<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Append(var, _, _) => *var,
            Self::Prepend(var, _, _) => *var,
            Self::Front(var, _) => *var,
            Self::Back(var, _) => *var,
            Self::Index(var, _, _) => *var,
        }
    }
}

impl fmt::Display for Method<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Append(var, _, num) => write!(f, "{}.append({num})", var.fragment()),
            Self::Prepend(var, _, num) => write!(f, "{}.prepend({num})", var.fragment()),
            Self::Front(var, _) => write!(f, "{}.front()", var.fragment()),
            Self::Back(var, _) => write!(f, "{}.back()", var.fragment()),
            Self::Index(var, _, num) => write!(f, "{var}.index({num})"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Number<'a>(pub usize, pub Span<'a>, pub Option<Span<'a>>);

impl<'a> RefSpan<'a> for Number<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.1
    }
}

impl std::fmt::Display for Number<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.2 {
            write!(f, "{}{}", span.fragment(), self.1.fragment())
        } else {
            write!(f, "{}", self.1.fragment())
        }
    }
}

impl<'a> Number<'a> {
    pub fn new(number: usize, span: Span<'a>, tag: Option<Span<'a>>) -> Self {
        Self(number, span, tag)
    }
}

impl<'a> From<Number<'a>> for usize {
    fn from(value: Number<'a>) -> Self {
        value.0
    }
}

impl<'a> From<(usize, Span<'a>, Option<Span<'a>>)> for Number<'a> {
    fn from(value: (usize, Span<'a>, Option<Span<'a>>)) -> Self {
        Self(value.0, value.1, value.2)
    }
}

impl<'a> From<Number<'a>> for (usize, Span<'a>, Option<Span<'a>>) {
    fn from(value: Number<'a>) -> Self {
        (value.0, value.1, value.2)
    }
}

impl<'a> From<(usize, Span<'a>)> for Number<'a> {
    fn from(value: (usize, Span<'a>)) -> Self {
        Self(value.0, value.1, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range<'a> {
    pub fragment: Span<'a>,
    pub start: VarNum<'a>,
    pub end: VarNum<'a>,
}

impl<'a> RefSpan<'a> for Range<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.fragment
    }
}

impl std::fmt::Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl<'a> Range<'a> {
    pub fn new(fragment: Span<'a>, start: VarNum<'a>, end: VarNum<'a>) -> Self {
        Self {
            fragment,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Iter<'a> {
    List(VecDeque<VarNum<'a>>),
    Range(Range<'a>),
    Var(Span<'a>),
}

impl<'a> RefSpan<'a> for Iter<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::List(l) => l.front().map_or(Span::new(""), |v| v.get_span()),
            Self::Range(r) => r.fragment,
            Self::Var(v) => *v,
        }
    }
}

impl fmt::Display for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List(l) => {
                write!(f, "[")?;
                let len = l.len();
                for (i, j) in l.iter().enumerate() {
                    write!(f, "{j}")?;
                    if i < len - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Self::Range(r) => write!(f, "{r}"),
            Self::Var(v) => write!(f, "{}", v.fragment()),
        }
    }
}

impl<'a> From<VecDeque<VarNum<'a>>> for Iter<'a> {
    fn from(value: VecDeque<VarNum<'a>>) -> Self {
        Self::List(value)
    }
}

impl<'a> From<Range<'a>> for Iter<'a> {
    fn from(value: Range<'a>) -> Self {
        Self::Range(value)
    }
}

impl<'a> From<Span<'a>> for Iter<'a> {
    fn from(value: Span<'a>) -> Self {
        Self::Var(value)
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Loop<'a> {
    pub kind: Loops<'a>,
    pub body: VecDeque<Line<'a>>,
    pub open: Span<'a>,
    pub close: Span<'a>,
}

impl<'a> RefSpan<'a> for Loop<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.open
    }
}

impl fmt::Display for Loop<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        for line in &self.body {
            write!(f, "    {line}")?;
        }
        writeln!(f, "}}")
    }
}

impl<'a> Loop<'a> {
    pub fn new(kind: Loops<'a>, open: Span<'a>, close: Span<'a>) -> Self {
        let body = VecDeque::new();
        Self {
            kind,
            body,
            open,
            close,
        }
    }

    pub fn new_with_body(
        kind: Loops<'a>,
        body: VecDeque<Line<'a>>,
        open: Span<'a>,
        close: Span<'a>,
    ) -> Self {
        Self {
            kind,
            body,
            open,
            close,
        }
    }

    pub fn add_line(&mut self, line: Line<'a>) {
        self.body.push_back(line)
    }

    pub fn get_line(&mut self) -> Option<Line<'a>> {
        self.body.pop_front()
    }
}

impl<'a> From<(Loops<'a>, Span<'a>, Span<'a>)> for Loop<'a> {
    fn from(value: (Loops<'a>, Span<'a>, Span<'a>)) -> Self {
        Loop::new(value.0, value.1, value.2)
    }
}

impl<'a> From<(Loops<'a>, VecDeque<Line<'a>>, Span<'a>, Span<'a>)> for Loop<'a> {
    fn from(value: (Loops<'a>, VecDeque<Line<'a>>, Span<'a>, Span<'a>)) -> Self {
        Loop::new_with_body(value.0, value.1, value.2, value.3)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loops<'a> {
    For(Span<'a>, Span<'a>, Iter<'a>),
    While(Span<'a>, CompareOp<'a>),
    If(Span<'a>, CompareOp<'a>),
}

impl<'a> RefSpan<'a> for Loops<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::For(span, _, _) | Self::While(span, _) | Self::If(span, _) => *span,
        }
    }
}

impl fmt::Display for Loops<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::For(_, span, iter) => writeln!(f, "for ({} in {iter}) {{", span.fragment()),
            Self::While(_, cond) => writeln!(f, "while ({cond}) {{"),
            Self::If(_, cond) => writeln!(f, "if ({cond}) {{"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable<'a> {
    pub name: Span<'a>,
    pub value: VarOrVal<'a>,
}

impl<'a> RefSpan<'a> for Variable<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.name
    }
}

impl fmt::Display for Variable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name.fragment(), self.value)
    }
}

impl<'a> Variable<'a> {
    pub fn new(name: Span<'a>, value: VarOrVal<'a>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarOrVal<'a> {
    Var(Span<'a>),
    Num(Number<'a>),
    List(VecDeque<VarNum<'a>>),
    Range(Range<'a>),
    Expr(BitExpr<'a>),
    SepExpr(SepBitExpr<'a>),
    Func(Funcs<'a>),
    Method(Method<'a>),
}

impl<'a> RefSpan<'a> for VarOrVal<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Var(v) => *v,
            Self::Num(n) => n.get_span(),
            Self::Expr(e) => e.get_span(),
            Self::Method(m) => m.get_span(),
            Self::List(l) => l.front().map_or(Span::new(""), |v| v.get_span()),
            Self::Range(r) => r.fragment,
            Self::Func(u) => u.get_span(),
            Self::SepExpr(se) => se.get_span(),
        }
    }
}

impl fmt::Display for VarOrVal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{}", v.fragment()),
            Self::Num(x) => write!(f, "{x}"),
            Self::Expr(x) => {
                write!(f, "{x}")
            }
            Self::Method(m) => {
                write!(f, "{m}")
            }
            Self::List(l) => {
                write!(f, "[")?;
                let len = l.len();
                for (i, j) in l.iter().enumerate() {
                    write!(f, "{j}")?;
                    if i < len - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Self::Range(r) => {
                write!(f, "{r}")
            }
            Self::Func(u) => {
                write!(f, "{u}")
            }
            Self::SepExpr(se) => {
                write!(f, "{se}")
            }
        }
    }
}

impl<'a> From<Method<'a>> for VarOrVal<'a> {
    fn from(value: Method<'a>) -> Self {
        Self::Method(value)
    }
}

impl<'a> From<Range<'a>> for VarOrVal<'a> {
    fn from(value: Range<'a>) -> Self {
        Self::Range(value)
    }
}

impl<'a> From<Span<'a>> for VarOrVal<'a> {
    fn from(value: Span<'a>) -> Self {
        Self::Var(value)
    }
}

impl<'a> From<(usize, Span<'a>, Option<Span<'a>>)> for VarOrVal<'a> {
    fn from(value: (usize, Span<'a>, Option<Span<'a>>)) -> Self {
        Self::Num(Number::new(value.0, value.1, value.2))
    }
}

impl<'a> From<Number<'a>> for VarOrVal<'a> {
    fn from(value: Number<'a>) -> Self {
        Self::Num(value)
    }
}

impl<'a> From<BitExpr<'a>> for VarOrVal<'a> {
    fn from(value: BitExpr<'a>) -> Self {
        Self::Expr(value)
    }
}

impl<'a> From<SepBitExpr<'a>> for VarOrVal<'a> {
    fn from(value: SepBitExpr<'a>) -> Self {
        Self::SepExpr(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarNum<'a> {
    Var(Span<'a>),
    Num(Number<'a>),
    Expr(Box<SepBitExpr<'a>>),
    Func(Box<Funcs<'a>>),
    Method(Box<Method<'a>>),
}

impl<'a> RefSpan<'a> for VarNum<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Var(v) => *v,
            Self::Num(n) => n.get_span(),
            Self::Expr(e) => e.get_span(),
            Self::Func(f) => f.get_span(),
            Self::Method(m) => m.get_span(),
        }
    }
}

impl VarNum<'_> {
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    pub fn is_num(&self) -> bool {
        matches!(self, Self::Num(_))
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }
}

impl fmt::Display for VarNum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{}", v.fragment()),
            Self::Num(x) => write!(f, "{x}"),
            Self::Expr(x) => {
                write!(f, "{x}")
            }
            Self::Func(u) => write!(f, "{u}"),
            Self::Method(m) => write!(f, "{m}"),
        }
    }
}

impl<'a> From<Method<'a>> for VarNum<'a> {
    fn from(value: Method<'a>) -> Self {
        Self::Method(Box::new(value))
    }
}

impl<'a> From<Funcs<'a>> for VarNum<'a> {
    fn from(value: Funcs<'a>) -> Self {
        Self::Func(Box::new(value))
    }
}

impl<'a> From<VecDeque<VarNum<'a>>> for VarOrVal<'a> {
    fn from(value: VecDeque<VarNum<'a>>) -> Self {
        Self::List(value)
    }
}

impl<'a> From<Span<'a>> for VarNum<'a> {
    fn from(value: Span<'a>) -> Self {
        Self::Var(value)
    }
}

impl<'a> From<(usize, Span<'a>, Option<Span<'a>>)> for VarNum<'a> {
    fn from(value: (usize, Span<'a>, Option<Span<'a>>)) -> Self {
        Self::Num(Number::new(value.0, value.1, value.2))
    }
}

impl<'a> From<Number<'a>> for VarNum<'a> {
    fn from(value: Number<'a>) -> Self {
        Self::Num(value)
    }
}

impl<'a> From<SepBitExpr<'a>> for VarNum<'a> {
    fn from(value: SepBitExpr<'a>) -> Self {
        Self::Expr(Box::new(value))
    }
}

impl<'a> From<Funcs<'a>> for VarOrVal<'a> {
    fn from(value: Funcs<'a>) -> Self {
        Self::Func(value)
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Line<'a> {
    Empty,
    Variable(Variable<'a>),
    Expr(BitExpr<'a>),
    Comp(CompareOp<'a>),
    Func(Funcs<'a>),
    Loop(Loop<'a>),
    Method(Method<'a>),
}

impl<'a> RefSpan<'a> for Line<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Empty => Span::new(""),
            Self::Variable(v) => v.get_span(),
            Self::Expr(e) => e.get_span(),
            Self::Comp(c) => c.get_span(),
            Self::Func(u) => u.get_span(),
            Self::Loop(l) => l.get_span(),
            Self::Method(m) => m.get_span(),
        }
    }
}

impl Line<'_> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }

    pub fn is_comp(&self) -> bool {
        matches!(self, Self::Comp(_))
    }

    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(_))
    }

    pub fn is_loop(&self) -> bool {
        matches!(self, Self::Loop(_))
    }

    pub fn is_method(&self) -> bool {
        matches!(self, Self::Method(_))
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => writeln!(f),
            Self::Variable(v) => writeln!(f, "{v}"),
            Self::Expr(e) => writeln!(f, "{e}"),
            Self::Comp(c) => writeln!(f, "{c}"),
            Self::Func(u) => writeln!(f, "{u}"),
            Self::Loop(l) => writeln!(f, "{l}"),
            Self::Method(m) => writeln!(f, "{m}"),
        }
    }
}

impl<'a> From<Method<'a>> for Line<'a> {
    fn from(value: Method<'a>) -> Self {
        Self::Method(value)
    }
}

impl<'a> From<Variable<'a>> for Line<'a> {
    fn from(value: Variable<'a>) -> Self {
        Self::Variable(value)
    }
}

impl<'a> From<BitExpr<'a>> for Line<'a> {
    fn from(value: BitExpr<'a>) -> Self {
        Self::Expr(value)
    }
}

impl<'a> From<CompareOp<'a>> for Line<'a> {
    fn from(value: CompareOp<'a>) -> Self {
        Self::Comp(value)
    }
}

impl<'a> From<Funcs<'a>> for Line<'a> {
    fn from(value: Funcs<'a>) -> Self {
        Self::Func(value)
    }
}

impl<'a> From<Loop<'a>> for Line<'a> {
    fn from(value: Loop<'a>) -> Self {
        Self::Loop(value)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Funcs<'a> {
    Bool(Span<'a>, BoolFunc<'a>),
    Quit(Span<'a>),
    Help(Span<'a>),
    History(Span<'a>),
    Clear(Span<'a>),
    Hex(Span<'a>, VarNum<'a>),
    Bin(Span<'a>, VarNum<'a>),
    Oct(Span<'a>, VarNum<'a>),
    Dec(Span<'a>, VarNum<'a>),
    Log(Span<'a>, VarNum<'a>, VarNum<'a>),
}

impl<'a> RefSpan<'a> for Funcs<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Bool(span, _)
            | Self::Log(span, _, _)
            | Self::Quit(span)
            | Self::History(span)
            | Self::Clear(span)
            | Self::Hex(span, _)
            | Self::Bin(span, _)
            | Self::Oct(span, _)
            | Self::Dec(span, _)
            | Self::Help(span) => *span,
        }
    }
}

impl fmt::Display for Funcs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(_, i) => write!(f, "bool({i})"),
            Self::Quit(_) => write!(f, "quit()"),
            Self::Hex(_, v) => write!(f, "hex({v})"),
            Self::Bin(_, v) => write!(f, "bin({v})"),
            Self::Oct(_, v) => write!(f, "oct({v})"),
            Self::Dec(_, v) => write!(f, "dec({v})"),
            Self::Help(_) => write!(f, "help()"),
            Self::History(_) => write!(f, "history()"),
            Self::Clear(_) => write!(f, "clear()"),
            Self::Log(_, left, right) => write!(f, "log({left}, {right})"),
        }
    }
}

impl<'a> From<(Span<'a>, BoolFunc<'a>)> for Funcs<'a> {
    fn from(value: (Span<'a>, BoolFunc<'a>)) -> Self {
        Self::Bool(value.0, value.1)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoolFunc<'a> {
    Compare(CompareOp<'a>),
    VarNum(VarNum<'a>),
}

impl<'a> RefSpan<'a> for BoolFunc<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        match self {
            Self::Compare(c) => c.get_span(),
            Self::VarNum(v) => v.get_span(),
        }
    }
}

impl fmt::Display for BoolFunc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compare(c) => write!(f, "{c}"),
            Self::VarNum(v) => write!(f, "{v}"),
        }
    }
}

impl<'a> From<CompareOp<'a>> for BoolFunc<'a> {
    fn from(value: CompareOp<'a>) -> Self {
        Self::Compare(value)
    }
}

impl<'a> From<VarNum<'a>> for BoolFunc<'a> {
    fn from(value: VarNum<'a>) -> Self {
        Self::VarNum(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compare {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

impl fmt::Display for Compare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareOp<'a> {
    pub left: VarNum<'a>,
    pub op: Compare,
    pub op_span: Span<'a>,
    pub right: VarNum<'a>,
}

impl<'a> RefSpan<'a> for CompareOp<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.op_span
    }
}

impl fmt::Display for CompareOp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

impl<'a> CompareOp<'a> {
    pub fn new(left: VarNum<'a>, op: Compare, op_span: Span<'a>, right: VarNum<'a>) -> Self {
        Self {
            left,
            op,
            op_span,
            right,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitExpr<'a> {
    pub left: VarNum<'a>,
    pub op: BitOps,
    pub op_span: Span<'a>,
    pub right: Option<VarNum<'a>>,
}

impl<'a> RefSpan<'a> for BitExpr<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.op_span
    }
}

impl<'a> BitExpr<'a> {
    pub fn new(left: VarNum<'a>, op: BitOps, op_span: Span<'a>, right: Option<VarNum<'a>>) -> Self {
        Self {
            left,
            op,
            op_span,
            right,
        }
    }
}

impl std::fmt::Display for BitExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(right) = &self.right {
            write!(f, "{} {} {right}", self.left, self.op)
        } else {
            write!(f, "{} {}", self.op, self.left)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SepBitExpr<'a> {
    pub open: Span<'a>,
    pub expr: BitExpr<'a>,
    pub close: Span<'a>,
}

impl<'a> RefSpan<'a> for SepBitExpr<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        self.expr.get_span()
    }
}

impl<'a> SepBitExpr<'a> {
    pub fn new(open: Span<'a>, expr: BitExpr<'a>, close: Span<'a>) -> Self {
        Self { open, expr, close }
    }
}

impl fmt::Display for SepBitExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.open.fragment(),
            self.expr,
            self.close.fragment()
        )
    }
}

impl<'a> From<(Span<'a>, BitExpr<'a>, Span<'a>)> for SepBitExpr<'a> {
    fn from(value: (Span<'a>, BitExpr<'a>, Span<'a>)) -> Self {
        Self {
            open: value.0,
            expr: value.1,
            close: value.2,
        }
    }
}
