#[derive(PartialEq, Debug, Clone)]
/// Either left or right value
pub enum Either<L, R> {
    /// The left value
    Left(L),
    /// The right value
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Returns self by reference
    pub fn as_ref(&self) -> Either<&L, &R> {
        match *self {
            Self::Left(ref inner) => Either::Left(inner),
            Self::Right(ref inner) => Either::Right(inner),
        }
    }
}
