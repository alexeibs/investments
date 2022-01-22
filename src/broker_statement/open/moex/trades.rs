use std::collections::HashMap;

use log::warn;
use serde::Deserialize;

use crate::broker_statement::open::common::{deserialize_date, parse_quantity};
use crate::broker_statement::partial::PartialBrokerStatement;
use crate::broker_statement::trades::{StockBuy, StockSell};
use crate::core::{EmptyResult, GenericResult};
use crate::currency::Cash;
use crate::types::{Date, DateTime};
use crate::types::Decimal;
use crate::util::{self, DecimalRestrictions};

use super::common::{deserialize_date_time, get_symbol};

#[derive(Deserialize)]
pub struct ConcludedTrades {
    #[serde(rename = "item")]
    trades: Vec<ConcludedTrade>,
}

#[derive(Deserialize)]
struct ConcludedTrade {
    #[serde(rename = "deal_no")]
    id: u64,

    security_name: String,

    #[serde(deserialize_with = "deserialize_date_time")]
    conclusion_time: DateTime,

    #[serde(deserialize_with = "deserialize_date")]
    execution_date: Date,

    #[serde(rename = "buy_qnty")]
    buy_quantity: Option<Decimal>,

    #[serde(rename = "sell_qnty")]
    sell_quantity: Option<Decimal>,

    price: Decimal,

    #[serde(rename = "price_currency_code")]
    price_currency: String,

    // volume - for repo trades
    // volume_currency - for ordinary trades
    #[serde(alias="volume_currency")]
    volume: Decimal,

    #[serde(rename = "accounting_currency_code")]
    accounting_currency: String,

    #[serde(rename = "broker_commission")]
    commission: Decimal,

    #[serde(rename = "broker_commission_currency_code")]
    commission_currency: Option<String>,
}

impl ConcludedTrades {
    pub fn parse(
        &self, statement: &mut PartialBrokerStatement, securities: &HashMap<String, String>,
        trades_with_shifted_execution_date: &mut HashMap<u64, Date>, repo: bool,
    ) -> EmptyResult {
        for trade in &self.trades {
            let symbol = get_symbol(securities, &trade.security_name)?;

            // Just don't know which one exactly is
            if trade.price_currency != trade.accounting_currency {
                return Err!(
                    "Trade currency for {} is not equal to accounting currency which is not supported yet",
                     symbol);
            }

            let price = util::validate_named_cash(
                "price", &trade.price_currency, trade.price,
                DecimalRestrictions::StrictlyPositive)?.normalize();

            let volume = util::validate_named_cash(
                "trade volume", &trade.price_currency, trade.volume,
                DecimalRestrictions::StrictlyPositive)?.normalize();

            let commission = util::validate_named_decimal(
                "commission", trade.commission, DecimalRestrictions::PositiveOrZero)?;

            let commission_currency = match trade.commission_currency.as_ref() {
                Some(currency) => currency,
                None if commission.is_zero() => &trade.price_currency,
                None => return Err!("Missing commission currency for {:?} trade", trade.id),
            };
            let commission = Cash::new(commission_currency, commission);

            let execution_date = match trades_with_shifted_execution_date.remove(&trade.id) {
                Some(execution_date) => {
                    warn!(concat!(
                        "Actual execution date of {:?} trade differs from the planned one. ",
                        "Fix execution date for this trade."
                    ), trade.id);

                    execution_date
                },
                None => trade.execution_date,
            };

            match (trade.buy_quantity, trade.sell_quantity) {
                (Some(quantity), None) => {
                    let quantity = util::validate_decimal(
                        parse_quantity(quantity), DecimalRestrictions::StrictlyPositive)?;
                    debug_assert_eq!(volume, price * quantity);

                    statement.stock_buys.push(StockBuy::new_trade(
                        symbol, quantity, price, volume, commission,
                        trade.conclusion_time.into(), execution_date, repo));
                },
                (None, Some(quantity)) => {
                    let quantity = util::validate_decimal(
                        parse_quantity(quantity), DecimalRestrictions::StrictlyPositive)?;
                    debug_assert_eq!(volume, price * quantity);

                    statement.stock_sells.push(StockSell::new_trade(
                        symbol, quantity, price, volume, commission,
                        trade.conclusion_time.into(), execution_date, repo, false));
                },
                _ => return Err!("Got an unexpected trade: Can't match it as buy or sell trade")
            };
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ExecutedTrades {
    #[serde(rename = "item")]
    trades: Vec<ExecutedTrade>,
}

#[derive(Deserialize)]
struct ExecutedTrade {
    #[serde(rename = "deal_no")]
    id: u64,

    #[serde(deserialize_with = "deserialize_date")]
    plan_execution_date: Date,

    #[serde(deserialize_with = "deserialize_date")]
    fact_execution_date: Date,
}

impl ExecutedTrades {
    pub fn parse(&self) -> GenericResult<HashMap<u64, Date>> {
        let mut trades_with_shifted_execution_date = HashMap::new();

        for trade in &self.trades {
            if trade.fact_execution_date != trade.plan_execution_date {
                if trades_with_shifted_execution_date.insert(trade.id, trade.fact_execution_date).is_some() {
                    return Err!("Got a duplicated {:?} trade", trade.id);
                }
            }
        }

        Ok(trades_with_shifted_execution_date)
    }
}

