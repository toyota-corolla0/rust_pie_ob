use std::{collections::HashMap, fmt::Display, hash::Hash};

use rust_decimal::Decimal;
use rust_ob::{OrderBook, OrderMatch, Side};

use crate::errors;

#[derive(Debug)]
pub struct PieOrderBook<OrderID>
where
    OrderID: Copy + PartialEq + Eq + Hash,
{
    contract_price: Decimal,
    order_books: Vec<OrderBook<OrderID>>,
}

impl<OrderID> PieOrderBook<OrderID>
where
    OrderID: Copy + PartialEq + Eq + Hash,
{
    /// Create new `PieOrderBook`
    ///
    /// IMPORTANT: This function panics if outcomes is less than 2
    pub fn new(contract_price: Decimal, outcomes: usize) -> Self {
        if outcomes < 2 {
            panic!("PieOrderBook: new: outcomes must always be 2 or greater")
        }

        let mut order_books = Vec::new();
        for _ in 0..outcomes {
            order_books.push(OrderBook::new())
        }

        PieOrderBook {
            contract_price,
            order_books,
        }
    }

    /// Process a new limit order
    ///
    /// IMPORTANT: PieOrderBook will possibly panic if another order
    /// with the same id exists already inside PieOrderBook
    pub fn process_limit_order(
        &mut self,
        id: OrderID,
        outcome: usize,
        side: Side,
        price: Decimal,
        mut quantity: Decimal,
    ) -> Result<Vec<OrderMatch<OrderID>>, errors::ProcessLimitOrder> {
        // order parameter validation
        let failed_validation = outcome >= self.order_books.len()
            || price <= Decimal::ZERO
            || price >= self.contract_price
            || quantity <= Decimal::ZERO;

        if failed_validation {
            return Err(errors::ProcessLimitOrder::OrderValidationFailed);
        }

        // process order
        let mut order_match_map: HashMap<OrderID, OrderMatch<OrderID>> = HashMap::new();

        while quantity > Decimal::ZERO {
            let (own_price, own_quantity) =
                self.get_order_book_best_price_quantity(outcome, side.opposite());
            let (others_price, others_quantity) =
                self.get_other_order_books_best_price_quantity(outcome, side.opposite());

            if match side {
                // buys match to own order book at same price
                Side::Buy => {
                    price >= own_price && !(own_price > others_price && !others_quantity.is_zero())
                }
                // sells match to other order books at same price
                Side::Sell => {
                    price <= own_price && !(own_price <= others_price && !others_quantity.is_zero())
                }
            } {
                // match in own outcome order book
                let satisfied_quantity = own_quantity.min(quantity);

                let order_match_vec = self.order_books[outcome]
                            .process_market_order(id, side, satisfied_quantity)
                            .expect("PieOrderBook::process_limit_order: order with id already exists in own outcome OrderBook");

                for order_match in order_match_vec {
                    Self::add_order_match_to_map(&mut order_match_map, &order_match)
                }

                quantity = quantity
                    .checked_sub(satisfied_quantity)
                    .expect("PieOrderBook: subtraction overflow");
            } else if match side {
                Side::Buy => price >= others_price && !others_quantity.is_zero(),
                Side::Sell => price <= others_price && !others_quantity.is_zero(),
            } {
                // match in other outcome order books
                let satisfied_quantity = others_quantity.min(quantity);

                for i in 0..self.order_books.len() {
                    if i == outcome {
                        continue;
                    }

                    let order_match_vec = self.order_books[i]
                                .process_market_order(id, side.opposite(), satisfied_quantity)
                                .expect("PieOrderBook::process_limit_order: order with id already exists in other outcome OrderBook");

                    assert_ne!(order_match_vec.len(), 0);

                    for order_match in order_match_vec.iter().rev().skip(1) {
                        Self::add_order_match_to_map(&mut order_match_map, order_match)
                    }
                }

                let mut cost = others_price
                    .checked_mul(satisfied_quantity)
                    .expect("PieOrderBook: multiplication overflow");
                if let Side::Sell = side {
                    cost.set_sign_negative(true);
                }

                Self::add_order_match_to_map(
                    &mut order_match_map,
                    &OrderMatch {
                        order: id,
                        quantity: satisfied_quantity,
                        cost,
                    },
                );

                quantity = quantity
                    .checked_sub(satisfied_quantity)
                    .expect("PieOrderBook: subtraction overflow");
            } else {
                // nothing satisfies
                break;
            }
        }

        // add remaining to outcome orderbook if not empty
        if !quantity.is_zero() {
            assert_eq!(
                self.order_books[outcome]
                    .process_limit_order(id, side, price, quantity)
                    .expect("PieOrderBook::process_limit_order: should never panic")
                    .len(),
                0
            );
        }

        Ok(order_match_map.into_values().collect())
    }

    /// Cancel an order
    ///
    /// IMPORTANT: PieOrderBook will panic if you try to cancel an order that
    /// does not exist within PieOrderBook
    ///
    /// outcome of order is required for finding the order book where the
    /// order exists. If the outcome is incorrect, this function will panic.
    pub fn cancel_order(&mut self, outcome: usize, id: OrderID) {
        self.order_books
            .get_mut(outcome)
            .expect("PieOrderBook::cancel_order: given outcome did not yield order_book")
            .cancel_order(id)
            .expect("PieOrderBook::cancel_order: error on cancel_order");
    }

    fn get_order_book_best_price_quantity(&self, outcome: usize, side: Side) -> (Decimal, Decimal) {
        let res = self.order_books[outcome].get_highest_priority_price_quantity(side);

        match side {
            Side::Buy => res.unwrap_or((Decimal::ZERO, Decimal::ZERO)),
            Side::Sell => res.unwrap_or((self.contract_price, Decimal::ZERO)),
        }
    }

    fn get_other_order_books_best_price_quantity(
        &self,
        outcome: usize,
        side: Side,
    ) -> (Decimal, Decimal) {
        let mut price = self.contract_price;
        let mut quantity = Decimal::MAX;

        for i in 0..self.order_books.len() {
            if i == outcome {
                continue;
            }

            let (highest_priority_price, highest_priority_quantity) =
                self.get_order_book_best_price_quantity(i, side.opposite());

            price = price
                .checked_sub(highest_priority_price)
                .expect("PieOrderBook: subtraction overflow");

            quantity = quantity.min(highest_priority_quantity);
        }

        (price, quantity)
    }

    fn add_order_match_to_map(
        map: &mut HashMap<OrderID, OrderMatch<OrderID>>,
        order_match: &OrderMatch<OrderID>,
    ) {
        match map.get_mut(&order_match.order) {
            Some(map_entry) => {
                map_entry.quantity = map_entry
                    .quantity
                    .checked_add(order_match.quantity)
                    .expect("PieOrderBook: addition overflow");

                map_entry.cost = map_entry
                    .cost
                    .checked_add(order_match.cost)
                    .expect("PieOrderBook: addition overflow");
            }

            None => {
                map.insert(order_match.order, order_match.clone());
            }
        }
    }
}

impl<OrderID> Display for PieOrderBook<OrderID>
where
    OrderID: Copy + PartialEq + Eq + Hash + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Contract Price: {}", self.contract_price)?;

        for i in 0..self.order_books.len() {
            writeln!(f, "\n-----OUTCOME ORDERBOOK {}-----", i)?;
            write!(f, "{}", self.order_books[i])?;
        }

        write!(f, "")
    }
}
