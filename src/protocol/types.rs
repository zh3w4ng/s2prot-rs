#[derive(Debug, PartialEq)]
pub enum TypeInfo {
    Int { offset: usize, length: usize },
    Variant2,
}
