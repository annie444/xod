use super::Span;
use crate::bitops::BitOps;
use std::{collections::VecDeque, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number<'a>(pub usize, pub Span<'a>, pub Option<Span<'a>>);

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
    pub start: VarNum<'a>,
    pub end: VarNum<'a>,
}

impl std::fmt::Display for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl<'a> Range<'a> {
    pub fn new(start: VarNum<'a>, end: VarNum<'a>) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Iter<'a> {
    List(VecDeque<VarNum<'a>>),
    Range(Range<'a>),
    Var(Span<'a>),
}

impl fmt::Display for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::List(l) => {
                write!(f, "[")?;
                let len = l.len();
                for (i, j) in l.iter().enumerate() {
                    write!(f, "{}", j)?;
                    if i < len - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Self::Range(r) => write!(f, "{}", r),
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

impl fmt::Display for Loop<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        for line in &self.body {
            write!(f, "    {}", line)?;
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

impl fmt::Display for Loops<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::For(_, span, iter) => writeln!(f, "for ({} in {}) {{", span.fragment(), iter),
            Self::While(_, cond) => writeln!(f, "while ({}) {{", cond),
            Self::If(_, cond) => writeln!(f, "if ({}) {{", cond),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable<'a> {
    pub name: Span<'a>,
    pub value: VarOrVal<'a>,
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
}

impl fmt::Display for VarOrVal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{}", v.fragment()),
            Self::Num(x) => write!(f, "{}", x),
            Self::Expr(x) => {
                write!(f, "{}", x)
            }
            Self::List(l) => {
                write!(f, "[")?;
                let len = l.len();
                for (i, j) in l.iter().enumerate() {
                    write!(f, "{}", j)?;
                    if i < len - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Self::Range(r) => {
                write!(f, "{}", r)
            }
            Self::Func(u) => {
                write!(f, "{}", u)
            }
            Self::SepExpr(se) => {
                write!(f, "{}", se)
            }
        }
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
}

impl fmt::Display for VarNum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{}", v.fragment()),
            Self::Num(x) => write!(f, "{}", x),
            Self::Expr(x) => {
                write!(f, "{}", x)
            }
        }
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
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => writeln!(f),
            Self::Variable(v) => writeln!(f, "{};", v),
            Self::Expr(e) => writeln!(f, "{};", e),
            Self::Comp(c) => writeln!(f, "{};", c),
            Self::Func(u) => write!(f, "{};", u),
            Self::Loop(l) => write!(f, "{};", l),
        }
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
    Run(Span<'a>),
}

impl fmt::Display for Funcs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(_, i) => write!(f, "bool({})", i),
            Self::Quit(_) => write!(f, "quit()"),
            Self::Run(_) => write!(f, "run()"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoolFunc<'a> {
    Compare(CompareOp<'a>),
    VarNum(VarNum<'a>),
}

impl fmt::Display for BoolFunc<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compare(c) => write!(f, "{}", c),
            Self::VarNum(v) => write!(f, "{}", v),
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
            write!(f, "{} {} {}", self.left, self.op, right)
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
