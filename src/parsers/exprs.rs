use super::{
    Expression, PartialEvalError, Span, VARIABLES,
    ast::{BitExpr, Number, SepBitExpr, VarNum},
};
use crate::bitops::BitOps;

fn get_var<'a>(var: &'a Span<'a>) -> Result<usize, PartialEvalError<'a>> {
    if VARIABLES.is_poisoned() {
        VARIABLES.clear_poison();
    }
    if let Ok(vars) = VARIABLES.try_lock().as_mut() {
        if let Some(value) = (*vars).get(var.fragment()) {
            Ok(*value)
        } else {
            Err(PartialEvalError {
                loc: var.to_owned(),
                msg: "Variable not defined.".to_owned(),
                fix: format!("{} = 0x42", var.fragment()),
            })
        }
    } else {
        Err(PartialEvalError {
            loc: var.to_owned(),
            msg: "Unable to access variable.".to_owned(),
            fix: "The program seems to be corrupted. Please exit and restart.".to_owned(),
        })
    }
}

impl Expression<usize> for Number<'_> {
    fn eval(&mut self) -> Result<usize, super::PartialEvalError> {
        Ok(self.0)
    }
}

impl Expression<usize> for BitExpr<'_> {
    fn eval(&mut self) -> Result<usize, super::PartialEvalError> {
        let left = self.left.eval()?;
        match self.op {
            BitOps::Not => Ok(!left),
            _ => {
                if let Some(ref mut right) = self.right {
                    let right = right.eval()?;
                    match self.op {
                        BitOps::LeftShift => Ok(left << right),
                        BitOps::And => Ok(left & right),
                        BitOps::Xor => Ok(left ^ right),
                        BitOps::RightShift => Ok(left >> right),
                        BitOps::Or => Ok(left | right),
                        _ => unreachable!(),
                    }
                } else {
                    Err(super::PartialEvalError {
                        msg: format!("Missing right operand for bitwise operation: {}", self.op),
                        fix: format!("{} {} 0x800", left, self.op),
                        loc: self.op_span.to_owned(),
                    })
                }
            }
        }
    }
}

impl Expression<usize> for VarNum<'_> {
    fn eval(&mut self) -> Result<usize, super::PartialEvalError> {
        match self {
            Self::Var(var) => get_var(var),
            Self::Num(val) => Ok((*val).eval()?),
            Self::Expr(e) => (*e).eval(),
        }
    }
}

impl Expression<usize> for SepBitExpr<'_> {
    fn eval(&mut self) -> Result<usize, super::PartialEvalError> {
        self.expr.eval()
    }
}
