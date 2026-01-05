mod keyed;

pub use keyed::{KeyCalculationError, Keyed};

impl Keyed for () {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for bool {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for u8 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for i8 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for u16 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for i16 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for u32 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for i32 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for u64 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for i64 {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for usize {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for isize {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl Keyed for String {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl<T: Send + Sync> Keyed for Vec<T> {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}

impl<T: Send + Sync> Keyed for [T] {
    fn key(&self) -> Result<[u8; 16], KeyCalculationError> {
        Ok([0; 16])
    }
}
