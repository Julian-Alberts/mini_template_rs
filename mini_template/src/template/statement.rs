#[cfg(feature = "assign")]
use super::assign::Assign;
#[cfg(feature = "conditional")]
use super::Conditional;
#[cfg(feature = "loop")]
use super::Loop;
use super::{custom_block::CustomBlock, CalculatedValue};
use crate::template::Include;

#[derive(Debug)]
pub enum Statement {
    Literal(*const str),
    Calculated(CalculatedValue),
    #[cfg(feature = "conditional")]
    Conditional(Conditional),
    CustomBlock(Box<dyn CustomBlock>),
    #[cfg(feature = "assign")]
    Assign(Assign),
    #[cfg(feature = "loop")]
    Loop(Loop),
    Include(Include),
}

impl PartialEq for Statement {
    fn eq(&self, other: &Statement) -> bool {
        match (self, other) {
            (Statement::Calculated(s), Statement::Calculated(o)) => s == o,
            (Statement::Literal(s), &Statement::Literal(o)) =>
            // Safety: Both literals point to positions in the original template string.
            unsafe { s.as_ref() == o.as_ref() },
            #[cfg(feature = "conditional")]
            (Statement::Conditional(s), Statement::Conditional(o)) => s == o,
            #[cfg(feature = "assign")]
            (Statement::Assign(s), Statement::Assign(o)) => s == o,
            #[cfg(feature = "loop")]
            (Statement::Loop(s), Statement::Loop(o)) => s == o,
            (Statement::CustomBlock(_s), Statement::CustomBlock(_o)) => todo!(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::value::ident::Ident;
    use crate::{template::CalculatedValue, value::StorageMethod};

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
            StorageMethod::Variable(Ident::new_static(&str1[3..6])),
            vec![],
        ));
        let calculated2 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static(&str2[5..8])),
            vec![],
        ));
        assert_eq!(calculated1, calculated2);
    }

    #[test]
    fn two_calclated_values_not_eq() {
        let str1 = "my var in a text";
        let str2 = "other VAR in an other text";
        let calculated1 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static(&str1[3..6])),
            vec![],
        ));
        let calculated2 = Statement::Calculated(CalculatedValue::new(
            StorageMethod::Variable(Ident::new_static(&str2[5..8])),
            vec![],
        ));
        assert_ne!(calculated1, calculated2);
    }
}
