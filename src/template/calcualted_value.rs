use super::StorageMethod;

#[derive(Debug)]
pub struct CalcualtedValue {
    pub value: StorageMethod,
    pub modifiers: Vec<(*const str, Vec<StorageMethod>)>,
}

impl PartialEq for CalcualtedValue {

    fn eq(&self, other: &Self) -> bool {
        if self.value != other.value {
            return false;
        }

        self.modifiers.iter().zip(&other.modifiers).all(|(s, o)|
            // Safety: Both modifier names point to positions in the original template string.
            unsafe { 
                println!("CalcualtedValue: {:#?} == {:#?} = {}", s.0.as_ref(), o.0.as_ref() ,s.0.as_ref() == o.0.as_ref() );
                s.0.as_ref() == o.0.as_ref() && s.1 == o.1 
            })
    }

}
