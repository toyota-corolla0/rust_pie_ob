use std::{collections::HashMap, hash::Hash};

use rust_decimal::Decimal;
use rust_ob::{OrderBook, OrderMatch, Side};

use crate::errors;

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

        match side {
            Side::Buy => {
                let mut order_match_map: HashMap<OrderID, OrderMatch<OrderID>> = HashMap::new();

                while quantity > Decimal::ZERO {
                    let (own_price, own_quantity) = self.get_own_order_book_price_quantity(outcome);
                    let (others_price, others_quantity) =
                        self.get_other_order_books_price_quantity(outcome);

                    if price < own_price.min(others_price) {
                        break;
                    }

                    if own_price <= others_price {
                        let satisfied_quantity = own_quantity.min(quantity);

                        let order_match_vec = self.order_books[outcome]
                            .process_market_order(id, Side::Buy, satisfied_quantity)
                            .expect("PieOrderBook::process_limit_order: order with id already exists in own outcome OrderBook");

                        for order_match in order_match_vec {
                            Self::add_order_match_to_map(&mut order_match_map, &order_match)
                        }

                        quantity = quantity
                            .checked_sub(satisfied_quantity)
                            .expect("PieOrderBook: subtraction overflow");
                    } else {
                        let satisfied_quantity = others_quantity.min(quantity);

                        for (i, other_outcome_ob) in self.order_books.iter_mut().enumerate() {
                            if i == outcome {
                                continue;
                            }

                            let order_match_vec = other_outcome_ob
                                .process_market_order(id, Side::Sell, satisfied_quantity)
                                .expect("PieOrderBook::process_limit_order: order with id already exists in other outcome OrderBook");

                            for order_match in order_match_vec.iter().rev().skip(1) {
                                Self::add_order_match_to_map(&mut order_match_map, order_match)
                            }
                        }

                        Self::add_order_match_to_map(
                            &mut order_match_map,
                            &OrderMatch {
                                order: id,
                                quantity: satisfied_quantity,
                                cost: others_price
                                    .checked_mul(satisfied_quantity)
                                    .expect("PieOrderBook: multiplication overflow"),
                            },
                        );

                        quantity = quantity
                            .checked_sub(satisfied_quantity)
                            .expect("PieOrderBook: subtraction overflow");
                    }
                }

                if !quantity.is_zero() {
                    assert_eq!(
                        self.order_books[outcome]
                            .process_limit_order(id, Side::Buy, price, quantity)
                            .expect("PieOrderBook::process_limit_order: should never panic")
                            .len(),
                        0
                    );
                }

                Ok(order_match_map.into_values().collect())
            }

            Side::Sell => {
                Ok(self.order_books[outcome].process_limit_order(id, Side::Sell, price, quantity)
                    .expect("PieOrderBook::process_limit_order: order with id already exists in own outcome OrderBook"))
            }
        }
    }

    pub fn cancel_order(&mut self, outcome: usize, id: OrderID) {
        self.order_books[outcome].cancel_order(id).expect("PieOrderBook::cancel_order: error on cancel_order");
    }

    fn get_own_order_book_price_quantity(&self, outcome: usize) -> (Decimal, Decimal) {
        self.order_books[outcome]
            .get_highest_priority_price_quantity(Side::Sell)
            .unwrap_or((self.contract_price, Decimal::ZERO))
    }

    fn get_other_order_books_price_quantity(&self, outcome: usize) -> (Decimal, Decimal) {
        let mut price = self.contract_price;
        let mut quantity = Decimal::MAX;

        for (i, ob) in self.order_books.iter().enumerate() {
            if i == outcome {
                continue;
            }

            let highest_priority_price_quantity_buy = ob
                .get_highest_priority_price_quantity(Side::Buy)
                .unwrap_or((Decimal::ZERO, Decimal::MAX));

            price = price
                .checked_sub(highest_priority_price_quantity_buy.0)
                .expect("PieOrderBook: subtraction overflow");

            quantity = quantity.min(highest_priority_price_quantity_buy.1);
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
