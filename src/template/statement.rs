#[cfg(not(feature = "disable_assign"))]
use super::assign::Assign;
use super::{CalculatedValue, Conditional};

#[derive(Debug)]
pub enum Statement {
    Literal(*const str),
    Calculated(CalculatedValue),
    Condition(Conditional),
    #[cfg(not(feature = "disable_assign"))]
    Assign(Assign),
}

impl PartialEq for Statement {
    fn eq(&self, other: &Statement) -> bool {
        match (self, other) {
            (Statement::Calculated(s), Statement::Calculated(o)) => s == o,
            (Statement::Literal(s), &Statement::Literal(o)) =>
            // Safety: Both literals point to positions in the original template string.
            unsafe { s.as_ref() == o.as_ref() },
            (Statement::Condition(s), Statement::Condition(o)) => s == o,
            #[cfg(not(feature = "disable_assign"))]
            (Statement::Assign(s), Statement::Assign(o)) => s == o,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::template::{CalculatedValue, StorageMethod};

    use super::Statement;

    #[test]
    fn two_literals_eq() {
        let str1 = "my literal value in a text";
        let str2 = "same literal value in an other text";
        let literal1 = Statement::Literal(&str1[3..16]);
        let literal2 = Statement::Literal(&str2[5..18]);
        assert_eq!(literal1, literal2);
    }

    #[test]
    fn two_literals_not_eq() {
        let str1 = "my literal value in a text";
        let str2 = "other LITERAL value in an other text";
        let literal1 = Statement::Literal(&str1[3..16]);
        let literal2 = Statement::Literal(&str2[5..18]);
        assert_ne!(literal1, literal2);
    }

    #[test]
    fn two_calclated_values_eq() {
        let str1 = "my var in a text";
        let str2 = "same var in an other text";
        let calculated1 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(&str1[3..6]),
            vec![],
        ));
        let calculated2 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(&str2[5..8]),
            vec![],
        ));
        assert_eq!(calculated1, calculated2);
    }

    #[test]
    fn two_calclated_values_not_eq() {
        let str1 = "my var in a text";
        let str2 = "other VAR in an other text";
        let calculated1 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(&str1[3..6]),
            vec![],
        ));
        let calculated2 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(&str2[5..8]),
            vec![],
        ));
        assert_ne!(calculated1, calculated2);
    }
}
