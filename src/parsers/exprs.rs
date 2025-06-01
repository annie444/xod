use std::{collections::VecDeque, fmt};

use super::{
    ExprError, Expression, PartialEvalError, Span, VARIABLES,
    ast::{
        BitExpr, BoolFunc, Compare, CompareOp, Funcs, Iter, Line, Loop, Loops, Number, Range,
        SepBitExpr, VarNum, VarOrVal, Variable,
    },
};
use crate::bitops::BitOps;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumOrList {
    Num(usize),
    List(VecDeque<usize>),
}

impl fmt::Display for NumOrList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumOrList::Num(n) => write!(f, "{}", n),
            NumOrList::List(list) => write!(
                f,
                "[{}]",
                list.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumOrListNoOp {
    Num(usize),
    List(VecDeque<usize>),
    NoOp,
}

impl fmt::Display for NumOrListNoOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumOrListNoOp::Num(n) => write!(f, "{}", n),
            NumOrListNoOp::List(list) => write!(
                f,
                "[{}]",
                list.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            NumOrListNoOp::NoOp => write!(f, "<Invalid Operation>"),
        }
    }
}

impl From<NumOrList> for NumOrListNoOp {
    fn from(value: NumOrList) -> Self {
        match value {
            NumOrList::Num(n) => NumOrListNoOp::Num(n),
            NumOrList::List(list) => NumOrListNoOp::List(list),
        }
    }
}

impl TryFrom<NumOrListNoOp> for NumOrList {
    type Error = ();

    fn try_from(value: NumOrListNoOp) -> Result<Self, Self::Error> {
        match value {
            NumOrListNoOp::Num(n) => Ok(NumOrList::Num(n)),
            NumOrListNoOp::List(list) => Ok(NumOrList::List(list)),
            NumOrListNoOp::NoOp => Err(()),
        }
    }
}

fn get_num(num: NumOrList, span: Span) -> Result<usize, ExprError> {
    match num {
        NumOrList::Num(n) => Ok(n),
        NumOrList::List(_) => Err(ExprError::Partial(PartialEvalError {
            loc: span.to_owned(),
            msg: "Expected a number, but got a list.".to_owned(),
            fix: "Use a list operation to access elements.".to_owned(),
        })),
    }
}

fn get_var(var: Span) -> Result<NumOrList, ExprError> {
    if VARIABLES.is_poisoned() {
        VARIABLES.clear_poison();
    }
    if let Ok(vars) = VARIABLES.try_lock().as_mut() {
        if let Some(value) = (*vars).get(*var.fragment()) {
            Ok(value.clone())
        } else {
            Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg: "Variable not defined.".to_owned(),
                fix: format!("{} = 0x42", var.fragment()),
            }))
        }
    } else {
        Err(ExprError::Partial(PartialEvalError {
            loc: var.to_owned(),
            msg: "Unable to access variable.".to_owned(),
            fix: "The program seems to be corrupted. Please exit and restart.".to_owned(),
        }))
    }
}

fn set_var(var: Span, value: NumOrList) -> Result<(), ExprError> {
    if VARIABLES.is_poisoned() {
        VARIABLES.clear_poison();
    }
    if let Ok(vars) = VARIABLES.try_lock().as_mut() {
        let _ = (*vars).insert((**(var.fragment())).to_owned(), value);
        Ok(())
    } else {
        Err(ExprError::Partial(PartialEvalError {
            loc: var.to_owned(),
            msg: "Unable to access variable.".to_owned(),
            fix: "The program seems to be corrupted. Please exit and restart.".to_owned(),
        }))
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for Number<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        Ok(self.0)
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, std::ops::Range<usize>> for Range<'a> {
    fn eval(&'b mut self) -> Result<std::ops::Range<usize>, ExprError<'a>> {
        let start_span = self.start.fragment().to_owned();
        let start = get_num(self.start.eval()?, start_span)?;
        let end_span = self.end.fragment().to_owned();
        let end = get_num(self.end.eval()?, end_span)?;
        match start.cmp(&end) {
            std::cmp::Ordering::Less => Err(PartialEvalError {
                loc: start_span,
                msg: "Start of range is greater than end.".to_owned(),
                fix: format!("{}..{}", end, start),
            }
            .into()),
            std::cmp::Ordering::Equal => Err(PartialEvalError {
                loc: start_span,
                msg: "Start of range is equal to end.".to_owned(),
                fix: format!("{}..{}", start, start + 1),
            }
            .into()),
            _ => Ok(start..end),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyIterator<'a> {
    Range(Option<Span<'a>>, std::ops::Range<usize>),
    List(Option<Span<'a>>, VecDeque<usize>),
    Expr(Option<Span<'a>>, bool, CompareOp<'a>),
}

impl<'a> AnyIterator<'a> {
    pub fn set_var(&mut self, var: Span<'a>) {
        match self {
            AnyIterator::Range(var_ref, _) => *var_ref = Some(var),
            AnyIterator::List(var_ref, _) => *var_ref = Some(var),
            AnyIterator::Expr(var_ref, _, _) => *var_ref = Some(var),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntoIter<'a> {
    pub vec: Option<VecDeque<usize>>,
    pub range: Option<std::ops::Range<usize>>,
    pub var: Option<Span<'a>>,
    pub expr: Option<CompareOp<'a>>,
    pub single: bool,
    pub done: bool,
}

impl Iterator for IntoIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let val = if let Some(vec) = &mut self.vec {
            vec.pop_front()
        } else if let Some(range) = &mut self.range {
            range.next()
        } else if let Some(expr) = &mut self.expr {
            if self.single {
                if self.done {
                    return None;
                }
                self.done = true;
            }
            if let Ok(val) = expr.eval() {
                if val == 0 {
                    return None;
                } else {
                    Some(val)
                }
            } else {
                return None;
            }
        } else {
            None
        };
        if let Some(var) = &self.var {
            if let Some(value) = val {
                set_var(*var, NumOrList::Num(value)).ok()?;
            }
        }
        val
    }
}

impl<'a> IntoIterator for AnyIterator<'a> {
    type Item = usize;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Range(var, range) => IntoIter {
                vec: None,
                range: Some(range),
                var,
                expr: None,
                single: false,
                done: false,
            },
            Self::List(var, list) => IntoIter {
                vec: Some(list),
                range: None,
                var,
                expr: None,
                single: false,
                done: false,
            },
            Self::Expr(var, single, expr) => IntoIter {
                vec: None,
                range: None,
                var,
                expr: Some(expr),
                single,
                done: false,
            },
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, AnyIterator<'a>> for Iter<'a> {
    fn eval(&'b mut self) -> Result<AnyIterator<'a>, ExprError<'a>> {
        match self {
            Iter::List(list) => {
                let mut deque: VecDeque<usize> = VecDeque::new();
                for item in list.iter_mut() {
                    match item {
                        VarNum::Var(var) => {
                            if let Ok(value) = get_var(*var) {
                                match value {
                                    NumOrList::Num(num) => deque.push_back(num),
                                    NumOrList::List(list) => {
                                        deque.extend(list);
                                    }
                                }
                            }
                        }
                        VarNum::Num(num) => deque.push_back(num.eval()?),
                        VarNum::Expr(expr) => deque.push_back(expr.eval()?),
                    }
                }
                Ok(AnyIterator::List(None, deque))
            }
            Iter::Var(var) => {
                let mut deque = VecDeque::new();
                if let Ok(value) = get_var(*var) {
                    match value {
                        NumOrList::Num(num) => deque.push_back(num),
                        NumOrList::List(list) => {
                            deque.extend(list);
                        }
                    }
                }
                Ok(AnyIterator::List(None, deque))
            }
            Iter::Range(range) => Ok(AnyIterator::Range(None, range.eval()?)),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, ()> for Loop<'a> {
    fn eval(&'b mut self) -> Result<(), ExprError<'a>> {
        let iter = self.kind.eval()?;
        for _ in iter {
            for ln in &mut self.body {
                ln.eval()?;
            }
        }
        Ok(())
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, AnyIterator<'a>> for Loops<'a> {
    fn eval(&'b mut self) -> Result<AnyIterator<'a>, ExprError<'a>> {
        match self {
            Loops::For(_, var, iter) => {
                let mut iter = iter.eval()?;
                iter.set_var(*var);
                Ok(iter)
            }
            Loops::While(_, op) => Ok(AnyIterator::Expr(None, false, op.clone())),
            Loops::If(_, op) => Ok(AnyIterator::Expr(None, true, op.clone())),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, ()> for Variable<'a> {
    fn eval(&'b mut self) -> Result<(), ExprError<'a>> {
        let val = self.value.eval()?;
        set_var(self.name, val)?;
        Ok(())
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrList> for VarOrVal<'a> {
    fn eval(&'b mut self) -> Result<NumOrList, ExprError<'a>> {
        match self {
            VarOrVal::Var(var) => get_var(*var),
            VarOrVal::Num(num) => Ok(NumOrList::Num(num.eval()?)),
            VarOrVal::List(list) => {
                let mut deque = VecDeque::new();
                for item in list.iter_mut() {
                    match item {
                        VarNum::Var(var) => {
                            if let Ok(value) = get_var(*var) {
                                match value {
                                    NumOrList::Num(num) => deque.push_back(num),
                                    NumOrList::List(list) => {
                                        deque.extend(list);
                                    }
                                }
                            }
                        }
                        VarNum::Num(num) => deque.push_back(num.eval()?),
                        VarNum::Expr(expr) => deque.push_back(expr.eval()?),
                    }
                }
                Ok(NumOrList::List(deque))
            }
            VarOrVal::Range(range) => Ok(NumOrList::List(range.eval()?.collect())),
            VarOrVal::Expr(expr) => expr.eval().map(NumOrList::Num),
            VarOrVal::SepExpr(sep_expr) => sep_expr.eval().map(NumOrList::Num),
            VarOrVal::Func(func) => func.eval(),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrList> for VarNum<'a> {
    fn eval(&'b mut self) -> Result<NumOrList, ExprError<'a>> {
        match self {
            VarNum::Var(var) => get_var(*var),
            VarNum::Num(num) => Ok(NumOrList::Num(num.eval()?)),
            VarNum::Expr(expr) => expr.eval().map(NumOrList::Num),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrListNoOp> for Line<'a> {
    fn eval(&'b mut self) -> Result<NumOrListNoOp, ExprError<'a>> {
        match self {
            Line::Variable(var) => var.eval().map(|_| NumOrListNoOp::NoOp),
            Line::Empty => Ok(NumOrListNoOp::NoOp),
            Line::Loop(loop_) => loop_.eval().map(|_| NumOrListNoOp::NoOp),
            Line::Expr(expr) => expr.eval().map(NumOrListNoOp::Num),
            Line::Comp(op) => op.eval().map(NumOrListNoOp::Num),
            Line::Func(func) => func.eval().map(NumOrListNoOp::from),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrList> for Funcs<'a> {
    fn eval(&'b mut self) -> Result<NumOrList, ExprError<'a>> {
        match self {
            Self::Quit(_) => Err(ExprError::Quit),
            Self::Bool(_, op) => op.eval().map(NumOrList::Num),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for BoolFunc<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        match self {
            Self::Compare(op) => op.eval(),
            Self::VarNum(var) => {
                let num_fragment = var.fragment().to_owned();
                let num = get_num(var.eval()?, num_fragment)?;
                if num >= 1 { Ok(1) } else { Ok(0) }
            }
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for CompareOp<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        let left = get_num(self.left.eval()?, self.op_span)?;
        let right = get_num(self.right.eval()?, self.op_span)?;
        match self.op {
            Compare::Equal => Ok((left == right) as usize),
            Compare::NotEqual => Ok((left != right) as usize),
            Compare::Greater => Ok((left > right) as usize),
            Compare::Less => Ok((left < right) as usize),
            Compare::GreaterEqual => Ok((left >= right) as usize),
            Compare::LessEqual => Ok((left <= right) as usize),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for BitExpr<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        let left = get_num(self.left.eval()?, self.op_span)?;
        match self.op {
            BitOps::Not => Ok(!left),
            _ => {
                if let Some(ref mut right) = self.right {
                    let right = get_num(right.eval()?, self.op_span)?;
                    match self.op {
                        BitOps::LeftShift => Ok(left << right),
                        BitOps::And => Ok(left & right),
                        BitOps::Xor => Ok(left ^ right),
                        BitOps::RightShift => Ok(left >> right),
                        BitOps::Or => Ok(left | right),
                        _ => unreachable!(),
                    }
                } else {
                    Err(ExprError::Partial(PartialEvalError {
                        msg: format!("Missing right operand for bitwise operation: {}", self.op),
                        fix: format!("{} {} 0x800", left, self.op),
                        loc: self.op_span.to_owned(),
                    }))
                }
            }
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for SepBitExpr<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        self.expr.eval()
    }
}
