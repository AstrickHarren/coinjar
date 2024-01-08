use itertools::Itertools;

use crate::{accn::AccnStore, journal::Booking};

use super::BuildBook;

pub(crate) struct RelativeDate<B: BuildBook> {
    inner: B,
    diff: Option<i32>,
}

impl<B: BuildBook> BuildBook for RelativeDate<B> {
    fn from_booking(booking: crate::journal::Booking) -> Self {
        Self {
            inner: B::from_booking(booking),
            diff: Default::default(),
        }
    }

    fn with_posting(
        &mut self,
        accn: crate::accn::Accn,
        money: Option<crate::valuable::Money>,
    ) -> &mut Self {
        self.inner.with_posting(accn, money);
        self
    }

    fn with_tag<'a>(
        &mut self,
        _accns: &mut AccnStore,
        tag_name: &str,
        args: impl Iterator<Item = impl AsRef<str>>,
    ) -> &mut Self {
        if tag_name == "date" {
            if self.diff.is_some() {
                panic!("relative date already set");
            }

            let diff = args
                .into_iter()
                .exactly_one()
                .unwrap_or_else(|_| panic!("expected exactly one argument for tag 'date'"));
            let diff = diff
                .as_ref()
                .parse::<i32>()
                .unwrap_or_else(|e| panic!("expected integer argument for tag 'date', got: {}", e));
            self.diff = Some(diff);
        } else {
            self.inner.with_tag(_accns, tag_name, args);
        }

        self
    }

    fn into_booking_with(self, accns: &mut AccnStore) -> Booking
    where
        Self: Sized,
    {
        let mut booking = self.inner.into_booking_with(accns);
        booking.date = booking.date + chrono::Duration::days(self.diff.unwrap_or(0) as i64);
        booking
    }

    fn parse_accn<'a>(
        &mut self,
        accns: &'a mut AccnStore,
        names: impl IntoIterator<Item = impl std::borrow::Borrow<str>>,
    ) -> crate::accn::AccnMut<'a> {
        self.inner.parse_accn(accns, names)
    }
}
