use crate::Dim;

/// A maze rendering target
#[derive(Debug, Clone, Copy)]
pub enum Display {
    Alternate(Dim),
    Standard,
}
