use super::{CastOpType, Expression, Variable};
use crate::prelude::*;

/// A side-effectful operation.
/// Can be a register assignment or a memory load/store operation.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Def {
    /// A memory load into the register given by `var`.
    Load {
        /// The target register of the memory load.
        /// The size of `var` also determines the number of bytes read from memory.
        var: Variable,
        /// The expression computing the address from which to read from.
        /// The size of `address` is required to match the pointer size of the corresponding CPU architecture.
        address: Expression,
    },
    /// A memory store operation.
    Store {
        /// The expression computing the address that is written to.
        /// The size of `address` is required to match the pointer size of the corresponding CPU architecture.
        address: Expression,
        /// The expression computing the value that is written to memory.
        /// The size of `value` also determines the number of bytes written.
        value: Expression,
    },
    /// A register assignment, assigning the result of the expression `value` to the register `var`.
    Assign {
        /// The register that is written to.
        var: Variable,
        /// The expression computing the value that is assigned to the register.
        value: Expression,
    },
}

impl Term<Def> {
    /// This function checks whether the instruction
    /// is a zero extension of the overwritten sub register of the previous instruction.
    /// If so, returns its TID
    pub fn check_for_zero_extension(
        &self,
        output_name: String,
        output_sub_register: String,
    ) -> Option<Tid> {
        match &self.term {
            Def::Assign {
                var,
                value:
                    Expression::Cast {
                        op: CastOpType::IntZExt,
                        arg,
                        ..
                    },
            } if output_name == var.name => {
                let argument: &Expression = arg;
                match argument {
                    Expression::Var(var) if var.name == output_sub_register => {
                        Some(self.tid.clone())
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Substitute every occurence of `input_var` in the address and value expressions
    /// with `replace_with_expression`.
    /// Does not change the target variable of assignment- and load-instructions.
    pub fn substitute_input_var(
        &mut self,
        input_var: &Variable,
        replace_with_expression: &Expression,
    ) {
        match &mut self.term {
            Def::Assign { var: _, value } => {
                value.substitute_input_var(input_var, replace_with_expression)
            }
            Def::Load { var: _, address } => {
                address.substitute_input_var(input_var, replace_with_expression)
            }
            Def::Store { address, value } => {
                address.substitute_input_var(input_var, replace_with_expression);
                value.substitute_input_var(input_var, replace_with_expression);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intermediate_representation::BinOpType;

    #[test]
    fn zero_extension_check() {
        let eax_variable = Expression::Var(Variable {
            name: String::from("EAX"),
            size: ByteSize::new(4),
            is_temp: false,
        });
        let int_sub_expr = Expression::BinOp {
            op: BinOpType::IntSub,
            lhs: Box::new(Expression::Var(Variable {
                name: String::from("EAX"),
                size: ByteSize::new(4),
                is_temp: false,
            })),
            rhs: Box::new(Expression::Var(Variable {
                name: String::from("ECX"),
                size: ByteSize::new(4),
                is_temp: false,
            })),
        };

        let zero_extend_def = Term {
            tid: Tid::new("zero_tid"),
            term: Def::Assign {
                var: Variable {
                    name: String::from("RAX"),
                    size: ByteSize::new(8),
                    is_temp: false,
                },
                value: Expression::Cast {
                    op: CastOpType::IntZExt,
                    size: ByteSize::new(8),
                    arg: Box::new(eax_variable.clone()),
                },
            },
        };
        // An expression that is a zero extension but does not directly contain a variable
        let zero_extend_but_no_var_def = Term {
            tid: Tid::new("zero_tid"),
            term: Def::Assign {
                var: Variable {
                    name: String::from("RAX"),
                    size: ByteSize::new(8),
                    is_temp: false,
                },
                value: Expression::Cast {
                    op: CastOpType::IntZExt,
                    size: ByteSize::new(8),
                    arg: Box::new(int_sub_expr.clone()),
                },
            },
        };

        let non_zero_extend_def = Term {
            tid: Tid::new("zero_tid"),
            term: Def::Assign {
                var: Variable {
                    name: String::from("RAX"),
                    size: ByteSize::new(8),
                    is_temp: false,
                },
                value: Expression::Cast {
                    op: CastOpType::IntSExt,
                    size: ByteSize::new(8),
                    arg: Box::new(eax_variable.clone()),
                },
            },
        };

        assert_eq!(
            zero_extend_def.check_for_zero_extension(String::from("RAX"), String::from("EAX")),
            Some(Tid::new("zero_tid"))
        );
        assert_eq!(
            zero_extend_but_no_var_def
                .check_for_zero_extension(String::from("RAX"), String::from("EAX")),
            None
        );
        assert_eq!(
            non_zero_extend_def.check_for_zero_extension(String::from("RAX"), String::from("EAX")),
            None
        );
    }
}
