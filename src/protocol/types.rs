#[derive(Debug, PartialEq)]
pub enum TypeInfo {
    Int {
        offset: usize,
        length: usize,
    },
    Bool,
    Blob {
        offset: usize,
        length: usize,
    },
    Array {
        offset: usize,
        length: usize,
        type_index: i16,
    },
    Optional {
        type_index: i16,
    },
}
