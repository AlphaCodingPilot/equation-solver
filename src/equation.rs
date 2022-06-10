use std::collections::VecDeque;

use crate::equation_element::{
    EquationElement::{self, *},
    MultiplicativeOperationType::Multiplication,
    OperationType::MultiplicativeOperation,
    ValueType::Variable,
};
use crate::equation_error::EquationError::{self, *};
use crate::equation_side::{EquationSide, EquationSideType::*};
use crate::nested_term::NestedTerm;
use crate::term::Term;

pub struct Equation {
    left_hand_side: Term,
    right_hand_side: Term,
}

impl Equation {
    pub fn new(tokens: Vec<EquationElement>) -> Result<Self, EquationError> {
        if tokens
            .iter()
            .filter(|element| **element == Value(Variable))
            .count()
            == 0
        {
            return Err(NoOccurrencesOfVariable);
        }

        let mut nested_terms = VecDeque::new();
        let mut left_hand_side = EquationSide::new(LeftHandSide);
        let mut right_hand_side = EquationSide::new(RightHandSide);
        let mut current_equation_side = &mut left_hand_side;
        let mut other_equation_side = &mut right_hand_side;
        let mut current_nested_term = NestedTerm::new();
        let mut previous_element = Separator;

        for element in tokens {
            match element.clone() {
                Value(value) => {
                    current_nested_term.multiply_value(&value, &previous_element)?;
                }
                Operation(operation) => {
                    current_nested_term.set_operation(operation, &previous_element)?
                }
                Separator => {
                    if let Value(_) | ClosingParenthesis = previous_element {
                        if let RightHandSide = current_equation_side.side {
                            return Err(InvalidSeparatorAmount);
                        }
                        if !nested_terms.is_empty() {
                            return Err(ParenthesisError);
                        }

                        current_nested_term.push_multiplier();
                        current_equation_side.term = current_nested_term.term;
                        current_equation_side = &mut right_hand_side;
                        other_equation_side = &mut left_hand_side;
                        current_nested_term = NestedTerm::new();
                    } else {
                        return Err(InvalidSeparator);
                    }
                }
                OpeningParenthesis => {
                    if let Value(_) | ClosingParenthesis = previous_element {
                        current_nested_term.set_operation(
                            MultiplicativeOperation(Multiplication),
                            &previous_element,
                        )?;
                    }
                    nested_terms.push_back(current_nested_term);
                    current_nested_term = NestedTerm::new();
                }
                ClosingParenthesis => {
                    if let Value(_) | ClosingParenthesis = previous_element {
                        current_nested_term.push_multiplier();

                        let mut nested_term = nested_terms.pop_back().ok_or(ParenthesisError)?;

                        nested_term.merge_term(&current_nested_term.term, other_equation_side)?;

                        current_nested_term = nested_term;
                    } else {
                        return Err(ParenthesisError);
                    }
                }
            }
            previous_element = element;
        }

        if let LeftHandSide = current_equation_side.side {
            return Err(InvalidSeparatorAmount);
        }
        if !nested_terms.is_empty() {
            return Err(ParenthesisError);
        }

        current_nested_term.push_multiplier();
        current_equation_side.term = current_nested_term.term;

        if left_hand_side.multiplier == right_hand_side.multiplier {
            left_hand_side.multiplier = Term::new_multiplier();
            right_hand_side.multiplier = Term::new_multiplier();
        }

        left_hand_side.push_multiplier();
        right_hand_side.push_multiplier();

        Ok(Self {
            left_hand_side: left_hand_side.term,
            right_hand_side: right_hand_side.term,
        })
    }

    pub fn set_zero(self) -> Term {
        let mut term = self.left_hand_side.clone();
        for (exponent, coefficient) in self.right_hand_side.addends.iter() {
            *term.addends.entry(*exponent).or_insert(0.0) -= coefficient;
        }

        term.exceptions_in_domain
            .extend(&self.right_hand_side.exceptions_in_domain);
        term
    }
}
