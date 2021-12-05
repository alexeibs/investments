use crate::core::GenericResult;
use crate::formatting;

use super::Date;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Period {
    first: Date,
    last: Date,
}

impl Period {
    pub fn new(first: Date, last: Date) -> GenericResult<Period> {
        let period = Period {first, last};

        if period.first > period.last {
            return Err!("Invalid period: {}", period.format());
        }

        Ok(period)
    }

    pub fn prev_date(&self) -> Date {
        self.first.pred()
    }

    pub fn first_date(&self) -> Date {
        self.first
    }

    pub fn last_date(&self) -> Date {
        self.last
    }

    pub fn next_date(&self) -> Date {
        self.last.succ()
    }

    pub fn contains(&self, date: Date) -> bool {
        self.first <= date && date <= self.last
    }

    pub fn days(&self) -> i64 {
        (self.last - self.first).num_days() + 1
    }

    pub fn format(&self) -> String {
        format!("{} - {}", formatting::format_date(self.first), formatting::format_date(self.last))
    }
}