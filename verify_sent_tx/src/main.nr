fn main(x: Field, y: pub Field) {
    assert(x != y);
}

#[test]
fn test_main(pay_amount: u128, event_hash: Field) {
    main(1, 2); 

    // Uncomment to make test fail
    // main(1, 1);
}
