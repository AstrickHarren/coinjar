use crate::{
    accn::{Accn, ContactId},
    journal::{Booking, Posting},
    valuable::{Money, Valuable},
};

use super::BuildBook;

enum Party {
    Me,
    Contact(ContactId),
}

pub(crate) struct Split<B: BuildBook> {
    inner: B,

    payer: Option<Party>,
    debtor: Vec<Party>,
    postings: Vec<Posting>,
}

impl<B: BuildBook> BuildBook for Split<B> {
    fn from_booking(booking: Booking) -> Self {
        Self {
            inner: B::from_booking(booking),
            payer: None,
            debtor: Vec::new(),
            postings: Vec::new(),
        }
    }

    fn with_posting(&mut self, _accn: Accn, _money: Option<Money>) -> &mut Self {
        todo!()
    }

    fn with_tag<'a>(&mut self, _tag_name: &str, _args: impl Iterator<Item = &'a str>) -> &mut Self {
        todo!()
    }

    fn into_booking(self) -> Booking {
        todo!()
    }
}

impl<B: BuildBook> Split<B> {
    fn split_inbalance(&mut self) -> Valuable {
        let _ret = self.postings.iter_mut().map(|_p| {});

        todo!()
    }
}
