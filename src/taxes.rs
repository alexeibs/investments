use std::collections::{HashMap, HashSet};
use std::default::Default;

use crate::localities::Country;
use crate::types::{Date, Decimal};
use chrono::Datelike;

#[derive(Debug, Clone, Copy)]
pub enum TaxPaymentDay {
    Day {month: u32, day: u32},
    OnClose,
}

impl Default for TaxPaymentDay {
    fn default() -> TaxPaymentDay {
        TaxPaymentDay::Day {
            month: 3,
            day: 15,
        }
    }
}

impl TaxPaymentDay {
    /// Returns an approximate date when tax is going to be paid for the specified income
    /// FIXME: Implement
    pub fn get(&self, income_date: Date) -> Date {
        Date::from_ymd(income_date.year() + 1, 3, 15)
    }
}

pub struct NetTaxCalculator {
    country: Country,
    tax_payment_day: TaxPaymentDay,
    profit: HashMap<Date, Decimal>,
}

impl NetTaxCalculator {
    pub fn new(country: Country, tax_payment_day: TaxPaymentDay) -> NetTaxCalculator {
        NetTaxCalculator {
            country,
            tax_payment_day,
            profit: HashMap::new(),
        }
    }

    pub fn add_profit(&mut self, date: Date, amount: Decimal) {
        self.profit.entry(self.tax_payment_day.get(date))
            .and_modify(|profit| *profit += amount)
            .or_insert(amount);
    }

    pub fn get_taxes(&self) -> HashMap<Date, Decimal> {
        let mut taxes = HashMap::new();
        let mut years = HashSet::new();

        for (&tax_payment_date, &profit) in self.profit.iter() {
            let year = tax_payment_date.year();
            assert!(years.insert(year)); // Ensure that we have only one tax payment date per year

            let tax_to_pay = self.country.tax_to_pay(profit, None);
            assert_eq!(taxes.insert(tax_payment_date, tax_to_pay), None);
        }

        taxes
    }
}
