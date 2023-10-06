use rust_decimal_macros::dec;
use rust_ob::OrderMatch;
use rust_pie_ob::PieOrderBook;

#[test]
fn process_limit_order1() {
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
