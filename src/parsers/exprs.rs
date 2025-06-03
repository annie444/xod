use std::{collections::VecDeque, fmt};

use super::{
    ExprError, Expression, PartialEvalError, RefSpan, Span, VARIABLES,
    ast::{
        BitExpr, BoolFunc, Compare, CompareOp, Funcs, Iter, Line, Loop, Loops, Method, Number,
        Range, SepBitExpr, VarNum, VarOrVal, Variable,
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

fn get_num(
    num: NumOrList,
    var: Span,
    msg: Option<String>,
    fix: Option<String>,
) -> Result<usize, ExprError> {
    match num {
        NumOrList::Num(n) => Ok(n),
        NumOrList::List(_) => match (msg, fix) {
            (Some(msg), Some(fix)) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg,
                fix,
            })),
            (Some(msg), None) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg,
                fix: "Use a list operation to access elements.".to_owned(),
            })),
            (None, Some(fix)) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg: "Expected a number, but got a list.".to_owned(),
                fix,
            })),
            (None, None) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg: "Expected a number, but got a list.".to_owned(),
                fix: "Use a list operation to access elements.".to_owned(),
            })),
        },
    }
}

fn get_list(
    list: NumOrList,
    var: Span,
    msg: Option<String>,
    fix: Option<String>,
) -> Result<VecDeque<usize>, ExprError> {
    match list {
        NumOrList::List(list) => Ok(list),
        NumOrList::Num(_) => match (msg, fix) {
            (Some(msg), Some(fix)) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg,
                fix,
            })),
            (Some(msg), None) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg,
                fix: format!("Try wrapping the number in brackets `[{}]`", var),
            })),
            (None, Some(fix)) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg: "Expected a list, but got a number.".to_owned(),
                fix,
            })),
            (None, None) => Err(ExprError::Partial(PartialEvalError {
                loc: var.to_owned(),
                msg: "Expected a list, but got a number.".to_owned(),
                fix: format!("Try wrapping the number in brackets `[{}]`", var),
            })),
        },
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
    if var.fragment().is_empty() {
        return Err(ExprError::Partial(PartialEvalError {
            loc: var.to_owned(),
            msg: "Variable name cannot be empty.".to_owned(),
            fix: "Please provide a valid variable name.".to_owned(),
        }));
    }
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

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrList> for Method<'a> {
    fn eval(&'b mut self) -> Result<NumOrList, ExprError<'a>> {
        match self {
            Self::Append(var, _, value) => {
                let list = get_var(*var)?;
                let mut list = get_list(list, *var, None, None)?;
                let value = get_num(value.eval()?, value.get_span(), None, None)?;
                list.push_back(value);
                set_var(*var, NumOrList::List(list.clone()))?;
                Ok(NumOrList::List(list))
            }
            Self::Prepend(var, _, value) => {
                let list = get_var(*var)?;
                let mut list = get_list(list, *var, None, None)?;
                let value = get_num(value.eval()?, value.get_span(), None, None)?;
                list.push_front(value);
                set_var(*var, NumOrList::List(list.clone()))?;
                Ok(NumOrList::List(list))
            }
            Self::Front(var, _) => {
                let list = get_var(*var)?;
                let mut list = get_list(list, *var, None, None)?;
                let num = list.pop_front().ok_or_else(|| {
                    ExprError::Partial(PartialEvalError {
                        loc: *var,
                        msg: "List is empty, cannot get front element.".to_owned(),
                        fix: "Ensure the list is not empty before calling front.".to_owned(),
                    })
                })?;
                set_var(*var, NumOrList::List(list))?;
                Ok(NumOrList::Num(num))
            }
            Self::Back(var, _) => {
                let list = get_var(*var)?;
                let mut list = get_list(list, *var, None, None)?;
                let num = list.pop_back().ok_or_else(|| {
                    ExprError::Partial(PartialEvalError {
                        loc: *var,
                        msg: "List is empty, cannot get front element.".to_owned(),
                        fix: "Ensure the list is not empty before calling front.".to_owned(),
                    })
                })?;
                set_var(*var, NumOrList::List(list))?;
                Ok(NumOrList::Num(num))
            }
            Self::Index(var, _, value) => {
                let list = get_var(*var)?;
                let list = get_list(list, *var, None, None)?;
                let index = get_num(value.eval()?, value.get_span(), None, None)?;
                if index >= list.len() {
                    return Err(ExprError::Partial(PartialEvalError {
                        loc: value.get_span().to_owned(),
                        msg: "Index out of bounds.".to_owned(),
                        fix: format!("{}.index({})", var.fragment(), list.len() - 1),
                    }));
                }
                let num = list[index];
                Ok(NumOrList::Num(num))
            }
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for Number<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        Ok(self.0)
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, std::ops::Range<usize>> for Range<'a> {
    fn eval(&'b mut self) -> Result<std::ops::Range<usize>, ExprError<'a>> {
        let start_span = self.start.get_span().to_owned();
        let start = get_num(self.start.eval()?, start_span, None, None)?;
        let end_span = self.end.get_span().to_owned();
        let end = get_num(self.end.eval()?, end_span, None, None)?;
        match start.cmp(&end) {
            std::cmp::Ordering::Greater => Err(PartialEvalError {
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
                        VarNum::Func(b) => deque.push_back(get_num(
                            b.eval()?,
                            b.get_span(),
                            Some(format!(
                                "`{}` returns a list, not a number.",
                                b.get_span().fragment()
                            )),
                            None,
                        )?),
                        VarNum::Method(b) => deque.push_back(get_num(
                            b.eval()?,
                            b.get_span(),
                            Some(format!(
                                "`{}` returns a list, not a number.",
                                b.get_span().fragment()
                            )),
                            Some(
                                "Try using `.front()` or `.back()` to get a single element."
                                    .to_owned(),
                            ),
                        )?),
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
            VarOrVal::Method(m) => m.eval(),
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
                        VarNum::Func(b) => {
                            deque.push_back(get_num(b.eval()?, b.get_span(), None, None)?)
                        }
                        VarNum::Method(b) => {
                            deque.push_back(get_num(
                                b.eval()?,
                                b.get_span(),
                                Some(format!(
                                    "`{}` returns a list, not a number.",
                                    b.get_span().fragment()
                                )),
                                Some(
                                    "Try using `.front()` or `.back()` to get a single element."
                                        .to_owned(),
                                ),
                            )?);
                        }
                    }
                }
                Ok(NumOrList::List(deque))
            }
            VarOrVal::Range(range) => Ok(NumOrList::List(range.eval()?.collect())),
            VarOrVal::Expr(expr) => expr.eval().map(NumOrList::Num),
            VarOrVal::SepExpr(sep_expr) => sep_expr.eval().map(NumOrList::Num),
            VarOrVal::Func(func) => match func.eval() {
                Ok(num_or_list) => Ok(num_or_list),
                Err(_) => Err(ExprError::Partial(PartialEvalError {
                    loc: func.get_span(),
                    msg: "Function did not return a number or list.".to_owned(),
                    fix: "Ensure the function returns a valid number or list.".to_owned(),
                })),
            },
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrList> for VarNum<'a> {
    fn eval(&'b mut self) -> Result<NumOrList, ExprError<'a>> {
        match self {
            VarNum::Var(var) => get_var(*var),
            VarNum::Num(num) => Ok(NumOrList::Num(num.eval()?)),
            VarNum::Expr(expr) => expr.eval().map(NumOrList::Num),
            VarNum::Func(b) => b.eval(),
            VarNum::Method(b) => b.eval(),
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
            Line::Method(m) => m.eval().map(NumOrListNoOp::from),
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, NumOrList> for Funcs<'a> {
    fn eval(&'b mut self) -> Result<NumOrList, ExprError<'a>> {
        match self {
            Self::Quit(_) => Err(ExprError::Quit),
            Self::Help(_) => Err(ExprError::Help),
            Self::History(_) => Err(ExprError::History),
            Self::Clear(_) => Err(ExprError::Clear),
            Self::Bool(_, op) => op.eval().map(NumOrList::Num),
            Self::Dec(_, var) => {
                let var = var.eval()?;
                match var {
                    NumOrList::Num(num) => {
                        println!("{}", num);
                        Ok(NumOrList::Num(num))
                    }
                    NumOrList::List(list) => {
                        let len = list.len() - 1;
                        let mut p = "[".to_string();
                        for (i, num) in list.iter().enumerate() {
                            if i < len {
                                p.push_str(&format!("{}, ", num));
                            } else {
                                p.push_str(&format!("{}", num));
                            }
                        }
                        p.push(']');
                        println!("{}", p);
                        Ok(NumOrList::List(list))
                    }
                }
            }
            Self::Hex(_, var) => {
                let var = var.eval()?;
                match var {
                    NumOrList::Num(num) => {
                        println!("0x{:x}", num);
                        Ok(NumOrList::Num(num))
                    }
                    NumOrList::List(list) => {
                        let len = list.len() - 1;
                        let mut p = "[".to_string();
                        for (i, num) in list.iter().enumerate() {
                            if i < len {
                                p.push_str(&format!("0x{:x}, ", num));
                            } else {
                                p.push_str(&format!("0x{:x}", num));
                            }
                        }
                        p.push(']');
                        println!("{}", p);
                        Ok(NumOrList::List(list))
                    }
                }
            }
            Self::Oct(_, var) => {
                let var = var.eval()?;
                match var {
                    NumOrList::Num(num) => {
                        println!("0o{:o}", num);
                        Ok(NumOrList::Num(num))
                    }
                    NumOrList::List(list) => {
                        let len = list.len() - 1;
                        let mut p = "[".to_string();
                        for (i, num) in list.iter().enumerate() {
                            if i < len {
                                p.push_str(&format!("0o{:o}, ", num));
                            } else {
                                p.push_str(&format!("0o{:o}", num));
                            }
                        }
                        p.push(']');
                        println!("{}", p);
                        Ok(NumOrList::List(list))
                    }
                }
            }
            Self::Bin(_, var) => {
                let var = var.eval()?;
                match var {
                    NumOrList::Num(num) => {
                        println!("0b{:b}", num);
                        Ok(NumOrList::Num(num))
                    }
                    NumOrList::List(list) => {
                        let len = list.len() - 1;
                        let mut p = "[".to_string();
                        for (i, num) in list.iter().enumerate() {
                            if i < len {
                                p.push_str(&format!("0b{:b}, ", num));
                            } else {
                                p.push_str(&format!("0b{:b}", num));
                            }
                        }
                        p.push(']');
                        println!("{}", p);
                        Ok(NumOrList::List(list))
                    }
                }
            }
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for BoolFunc<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        match self {
            Self::Compare(op) => op.eval(),
            Self::VarNum(var) => {
                let num_fragment = var.get_span().to_owned();
                let num = get_num(var.eval()?, num_fragment, None, None)?;
                if num >= 1 { Ok(1) } else { Ok(0) }
            }
        }
    }
}

impl<'b, 'a: 'b> Expression<'a, 'b, usize> for CompareOp<'a> {
    fn eval(&'b mut self) -> Result<usize, ExprError<'a>> {
        let left = get_num(self.left.eval()?, self.op_span, None, None)?;
        let right = get_num(self.right.eval()?, self.op_span, None, None)?;
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
        let left = get_num(self.left.eval()?, self.op_span, None, None)?;
        match self.op {
            BitOps::Not => Ok(!left),
            _ => {
                if let Some(ref mut right) = self.right {
                    let right = get_num(right.eval()?, self.op_span, None, None)?;
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

#[cfg(test)]
mod test {
    use crate::parsers::general::lines;

    use super::*;

    #[test]
    fn evaluate_simple_variable_expression() {
        let lns = lines(Span::new(
            r#"
x = 42
"#,
        ));
        assert!(lns.is_ok());
        let mut lns = lns.unwrap();
        for line in &mut lns.1 {
            let val = line.eval();
            assert!(val.is_ok());
            let val = val.unwrap();
            assert_eq!(val, NumOrListNoOp::NoOp);
        }
        let vars = VARIABLES.lock().unwrap();
        assert!(vars.contains_key("x"));
        let x = vars.get("x").unwrap();
        assert_eq!(x, &NumOrList::Num(42));
    }
}
