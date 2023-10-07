use rust_decimal_macros::dec;
use rust_ob::{OrderMatch, Side};
use rust_pie_ob::PieOrderBook;

#[test]
fn process_limit_order1() {
    let mut pie_ob = PieOrderBook::new(dec!(10), 4);

    use rust_pie_ob::errors::ProcessLimitOrder as E;
    assert_eq!(
        pie_ob.process_limit_order(0, 4, Side::Buy, dec!(3), dec!(5)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(1, 6, Side::Buy, dec!(3), dec!(5)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(2, 1, Side::Buy, dec!(15), dec!(5)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(3, 2, Side::Buy, dec!(10), dec!(10)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(4, 3, Side::Buy, dec!(-1), dec!(15)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(5, 1, Side::Buy, dec!(0), dec!(12)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(6, 0, Side::Buy, dec!(3), dec!(0)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(7, 2, Side::Buy, dec!(3), dec!(-1)),
        Err(E::OrderValidationFailed)
    );
}

#[test]
fn process_limit_order2() {
    let mut pie_ob = PieOrderBook::new(dec!(10), 2);

    let res = pie_ob
        .process_limit_order(1, 0, Side::Buy, dec!(3), dec!(5))
        .unwrap();
    assert_eq!(res.len(), 0);

    let res = pie_ob
        .process_limit_order(2, 0, Side::Buy, dec!(5), dec!(1))
        .unwrap();
    assert_eq!(res.len(), 0);

    let mut res = pie_ob
        .process_limit_order(3, 1, Side::Buy, dec!(8), dec!(3))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));
    assert_eq!(
        res,
        vec![
            OrderMatch {
                order: 1,
                quantity: dec!(2),
                cost: dec!(6)
            },
            OrderMatch {
                order: 2,
                quantity: dec!(1),
                cost: dec!(5)
            },
            OrderMatch {
                order: 3,
                quantity: dec!(3),
                cost: dec!(19)
            }
        ]
    );
}

#[test]
fn process_limit_order3() {
    let mut pie_ob = PieOrderBook::new(dec!(10), 3);

    let res = pie_ob
        .process_limit_order(1, 0, Side::Buy, dec!(3), dec!(5))
        .unwrap();
    assert_eq!(res.len(), 0);

    let res = pie_ob
        .process_limit_order(2, 0, Side::Buy, dec!(5), dec!(1))
        .unwrap();
    assert_eq!(res.len(), 0);

    let mut res = pie_ob
        .process_limit_order(3, 1, Side::Buy, dec!(8), dec!(3))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));

    let mut res = pie_ob
        .process_limit_order(4, 2, Side::Buy, dec!(3), dec!(4))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));
    assert_eq!(
        res,
        vec![
            OrderMatch {
                order: 1,
                quantity: dec!(2),
                cost: dec!(6)
            },
            OrderMatch {
                order: 2,
                quantity: dec!(1),
                cost: dec!(5)
            },
            OrderMatch {
                order: 3,
                quantity: dec!(3),
                cost: dec!(24)
            },
            OrderMatch {
                order: 4,
                quantity: dec!(3),
                cost: dec!(-5)
            }
        ]
    );
}

#[test]
fn cancel_order1() {
    let mut pie_ob = PieOrderBook::new(dec!(10), 2);

    assert_eq!(
        pie_ob
            .process_limit_order(1, 1, Side::Buy, dec!(5), dec!(1))
            .unwrap()
            .len(),
        0
    );

    pie_ob.cancel_order(1, 1);
}

#[test]
fn general1() {
    let mut pie_ob = PieOrderBook::new(dec!(100), 4);

    assert_eq!(
        pie_ob
            .process_limit_order(1, 0, Side::Buy, dec!(20), dec!(2))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(2, 0, Side::Buy, dec!(20), dec!(2))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(3, 0, Side::Buy, dec!(20), dec!(2))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(4, 1, Side::Buy, dec!(50), dec!(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(5, 2, Side::Buy, dec!(40), dec!(5))
            .unwrap()
            .len(),
        0
    );

    let mut res = pie_ob
        .process_limit_order(6, 3, Side::Buy, dec!(1), dec!(8))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));
    assert_eq!(
        res,
        vec![
            OrderMatch {
                order: 1,
                quantity: dec!(2),
                cost: dec!(40)
            },
            OrderMatch {
                order: 2,
                quantity: dec!(2),
                cost: dec!(40)
            },
            OrderMatch {
                order: 3,
                quantity: dec!(1),
                cost: dec!(20)
            },
            OrderMatch {
                order: 4,
                quantity: dec!(5),
                cost: dec!(250)
            },
            OrderMatch {
                order: 5,
                quantity: dec!(5),
                cost: dec!(200)
            },
            OrderMatch {
                order: 6,
                quantity: dec!(5),
                cost: dec!(-50)
            }
        ]
    );

    pie_ob.cancel_order(0, 3);
    pie_ob.cancel_order(3, 6);
}

#[test]
fn general2() {
    let mut pie_ob = PieOrderBook::new(dec!(100), 3);

    assert_eq!(
        pie_ob
            .process_limit_order(1, 0, Side::Buy, dec!(8), dec!(14))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(2, 0, Side::Buy, dec!(9), dec!(8))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(3, 0, Side::Buy, dec!(7), dec!(12))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(4, 0, Side::Sell, dec!(15), dec!(8))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(5, 0, Side::Sell, dec!(23), dec!(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(6, 0, Side::Sell, dec!(11), dec!(6))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(7, 0, Side::Sell, dec!(12), dec!(11))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(8, 1, Side::Buy, dec!(25), dec!(4))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(9, 1, Side::Buy, dec!(29), dec!(11))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(10, 1, Side::Buy, dec!(20), dec!(7))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(11, 1, Side::Buy, dec!(15), dec!(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(12, 1, Side::Sell, dec!(33), dec!(10))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(13, 1, Side::Sell, dec!(31), dec!(11))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(14, 1, Side::Sell, dec!(32), dec!(3))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(15, 2, Side::Buy, dec!(53), dec!(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(16, 2, Side::Buy, dec!(57), dec!(20))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(17, 2, Side::Buy, dec!(59), dec!(5))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(18, 2, Side::Buy, dec!(51), dec!(14))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(19, 2, Side::Sell, dec!(62), dec!(7))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
            .process_limit_order(20, 2, Side::Sell, dec!(66), dec!(12))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        pie_ob
        .process_limit_order(21, 2, Side::Sell, dec!(63), dec!(5))
        .unwrap()
        .len(),
        0
    );

    let mut res = pie_ob
        .process_limit_order(22, 2, Side::Buy, dec!(64), dec!(17))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));
    assert_eq!(
        res,
        vec![
            OrderMatch {
                order: 2,
                quantity: dec!(8),
                cost: dec!(72)
            },
            OrderMatch {
                order: 9,
                quantity: dec!(8),
                cost: dec!(232)
            },
            OrderMatch {
                order: 19,
                quantity: dec!(7),
                cost: dec!(-434)
            },
            OrderMatch {
                order: 21,
                quantity: dec!(2),
                cost: dec!(-126)
            },
            OrderMatch {
                order: 22,
                quantity: dec!(17),
                cost: dec!(1056)
            },
        ]
    );

    assert_eq!(
        pie_ob
        .process_limit_order(23, 1, Side::Sell, dec!(30), dec!(2))
        .unwrap()
        .len(),
        0
    );

    let mut res = pie_ob
        .process_limit_order(24, 2, Side::Sell, dec!(50), dec!(3))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));
    assert_eq!(
        res,
        vec![
            OrderMatch {
                order: 6,
                quantity: dec!(2),
                cost: dec!(-22)
            },
            OrderMatch {
                order: 17,
                quantity: dec!(1),
                cost: dec!(59)
            },
            OrderMatch {
                order: 23,
                quantity: dec!(2),
                cost: dec!(-60)
            },
            OrderMatch {
                order: 24,
                quantity: dec!(3),
                cost: dec!(-177)
            },
        ]
    );

    println!("{pie_ob}");
}
