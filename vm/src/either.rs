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

    /// Applies a function to the left side
    pub fn map_left<U, F: FnOnce(L) -> U>(self, f: F) -> Either<U, R> {
        match self {
            Self::Right(r) => Either::Right(r),
            Self::Left(l) => Either::Left(f(l)),
        }
    }

    /// Applies a function to the right side
    pub fn map_right<U, F: FnOnce(R) -> U>(self, f: F) -> Either<L, U> {
        match self {
            Self::Right(r) => Either::Right(f(r)),
            Self::Left(l) => Either::Left(l),
        }
    }

    /// Checks if `self` is of the left variant
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left(_))
    }

    /// Checks if `self` is of the right variant
    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right(_))
    }
}
