use std::{fmt::Display, iter::Peekable, ops::Deref};

pub(crate) trait NotEmpty {
    type Ok;
    fn not_empty(self) -> Option<Self::Ok>;

    fn empty(self) -> Result<(), Self::Ok>
    where
        Self: Sized,
    {
        self.not_empty().ok_or(()).flipped()
    }
}

impl<T> NotEmpty for T
where
    T: Iterator,
{
    type Ok = Peekable<T>;

    fn not_empty(self) -> Option<Self::Ok> {
        let mut peekable = self.peekable();
        let ret = peekable.peek().is_some().then_some(peekable);
        ret
    }
}

pub(crate) trait Flip {
    type Flipped;
    fn flipped(self) -> Self::Flipped;
}

impl<T, E> Flip for Result<T, E> {
    type Flipped = Result<E, T>;
    fn flipped(self) -> Self::Flipped {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e),
        }
    }
}

pub(crate) struct Formatted<'a, T: ?Sized> {
    fmt: &'a dyn Fn(&T) -> String,
    value: T,
}

impl<'a, T> Formatted<'a, T> {
    pub(crate) fn new(value: T, fmt: &'a impl Fn(&T) -> String) -> Self {
        Self { fmt, value }
    }
}

impl<T: ?Sized> Display for Formatted<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.fmt)(&self.value).fmt(f)
    }
}

impl<T> Deref for Formatted<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
