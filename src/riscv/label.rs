use std::fmt;
#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    pub name: String,
}

impl Label {
    pub fn new(name: String) -> Self {
        Label { name }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
