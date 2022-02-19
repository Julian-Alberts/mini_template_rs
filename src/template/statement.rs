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

                s_modifiers
                    .iter()
                    .zip(o_modifiers)
                    .all(|(s, o)| 
                    // Safety: Both modifier names point to positions in the original template string.
                    unsafe { s.0.as_ref() == o.0.as_ref() && s.1 == o.1 }
                )
            }
            (Statement::Literal(s), &Statement::Literal(o)) => 
                // Safety: Both literals point to positions in the original template string.
                unsafe { s.as_ref() == o.as_ref() 
            },
            _ => false,
        }
    }
}
