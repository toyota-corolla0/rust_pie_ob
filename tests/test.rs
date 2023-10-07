use rust_decimal_macros::dec;
use rust_ob::OrderMatch;
use rust_pie_ob::PieOrderBook;

#[test]
fn process_limit_order1() {
    let mut pie_ob = PieOrderBook::new(dec!(10), 4);

    use rust_pie_ob::errors::ProcessLimitOrder as E;
    assert_eq!(
        pie_ob.process_limit_order(0, 4, rust_ob::Side::Buy, dec!(3), dec!(5)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(1, 6, rust_ob::Side::Buy, dec!(3), dec!(5)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(2, 1, rust_ob::Side::Buy, dec!(15), dec!(5)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(3, 2, rust_ob::Side::Buy, dec!(10), dec!(10)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(4, 3, rust_ob::Side::Buy, dec!(-1), dec!(15)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(5, 1, rust_ob::Side::Buy, dec!(0), dec!(12)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(6, 0, rust_ob::Side::Buy, dec!(3), dec!(0)),
        Err(E::OrderValidationFailed)
    );
    assert_eq!(
        pie_ob.process_limit_order(7, 2, rust_ob::Side::Buy, dec!(3), dec!(-1)),
        Err(E::OrderValidationFailed)
    );
}

#[test]
fn process_limit_order2() {
    let mut pie_ob = PieOrderBook::new(dec!(10), 2);

    let res = pie_ob
        .process_limit_order(1, 0, rust_ob::Side::Buy, dec!(3), dec!(5))
        .unwrap();
    assert_eq!(res.len(), 0);

    let res = pie_ob
        .process_limit_order(2, 0, rust_ob::Side::Buy, dec!(5), dec!(1))
        .unwrap();
    assert_eq!(res.len(), 0);

    let mut res = pie_ob
        .process_limit_order(3, 1, rust_ob::Side::Buy, dec!(8), dec!(3))
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
        .process_limit_order(1, 0, rust_ob::Side::Buy, dec!(3), dec!(5))
        .unwrap();
    assert_eq!(res.len(), 0);

    let res = pie_ob
        .process_limit_order(2, 0, rust_ob::Side::Buy, dec!(5), dec!(1))
        .unwrap();
    assert_eq!(res.len(), 0);

    let mut res = pie_ob
        .process_limit_order(3, 1, rust_ob::Side::Buy, dec!(8), dec!(3))
        .unwrap();
    res.sort_by(|v1, v2| v1.order.cmp(&v2.order));

    let mut res = pie_ob
        .process_limit_order(4, 2, rust_ob::Side::Buy, dec!(3), dec!(4))
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
            .process_limit_order(1, 1, rust_ob::Side::Buy, dec!(5), dec!(1))
            .unwrap()
            .len(),
        0
    );

    pie_ob.cancel_order(1, 1);
}

#[test]
fn general1() {
    // let mut pie_ob = PieOrderBook::new(dec!(100), 4);
}
