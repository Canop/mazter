use crate::Dim;

#[derive(Debug, Clone, Copy)]
pub enum Display {
    Alternate(Dim),
    Standard,
}

