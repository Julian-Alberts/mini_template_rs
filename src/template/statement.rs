use super::storage_method::StorageMethod;

#[derive(Debug)]
pub enum Statement {
    Literal(*const str),
    Calculated {
        value: StorageMethod,
        modifiers: Vec<(*const str, Vec<StorageMethod>)>,
    },
}

impl PartialEq for Statement {
    fn eq(&self, other: &Statement) -> bool {
        match (self, other) {
            (
                Statement::Calculated {
                    value: s_value,
                    modifiers: s_modifiers,
                },
                Statement::Calculated {
                    value: o_value,
                    modifiers: o_modifiers,
                },
            ) => {
                if s_value != o_value {
                    return false;
                }

                s_modifiers.iter().zip(o_modifiers).all(|(s, o)|
                    // Safety: Both modifier names point to positions in the original template string.
                    unsafe { s.0.as_ref() == o.0.as_ref() && s.1 == o.1 })
            }
            (Statement::Literal(s), &Statement::Literal(o)) =>
            // Safety: Both literals point to positions in the original template string.
            unsafe { s.as_ref() == o.as_ref() },
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::template::StorageMethod;

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
        let calculated1 = Statement::Calculated {
            value: StorageMethod::Variable(&str1[3..6]),
            modifiers: vec![],
        };
        let calculated2 = Statement::Calculated {
            value: StorageMethod::Variable(&str2[5..8]),
            modifiers: vec![],
        };
        assert_eq!(calculated1, calculated2);
    }

    #[test]
    fn two_calclated_values_not_eq() {
        let str1 = "my var in a text";
        let str2 = "other VAR in an other text";
        let calculated1 = Statement::Calculated {
            value: StorageMethod::Variable(&str1[3..6]),
            modifiers: vec![],
        };
        let calculated2 = Statement::Calculated {
            value: StorageMethod::Variable(&str2[5..8]),
            modifiers: vec![],
        };
        assert_ne!(calculated1, calculated2);
    }
}
