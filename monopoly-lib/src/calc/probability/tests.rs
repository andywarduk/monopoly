use super::*;

#[test]
fn test_add1() {
    let a = p!(1 / 2);
    let b = p!(1 / 3);

    let c = a + b;

    assert_eq!(c, p!(5 / 6));
}

#[test]
fn test_add2() {
    let a = p!(1 / 4);
    let b = p!(1 / 4);

    let c = a + b;

    assert_eq!(c, p!(1 / 2));
}

#[test]
fn test_addref1() {
    let a = p!(1 / 2);
    let b = p!(1 / 3);

    let add_by_ref = |a: Probability, b: &Probability| a + b;
    let c = add_by_ref(a, &b);

    assert_eq!(c, p!(5 / 6));
}

#[test]
fn test_addref2() {
    let a = p!(1 / 4);
    let b = p!(1 / 4);

    let add_by_ref = |a: Probability, b: &Probability| a + b;
    let c = add_by_ref(a, &b);

    assert_eq!(c, p!(1 / 2));
}

#[test]
fn test_addassign1() {
    let mut a = p!(1 / 2);
    let b = p!(1 / 3);

    a += b;

    assert_eq!(a, p!(5 / 6));
}

#[test]
fn test_addassign2() {
    let mut a = p!(1 / 4);
    let b = p!(1 / 4);

    a += b;

    assert_eq!(a, p!(1 / 2));
}

#[test]
fn test_sum1() {
    let sum = [p!(1 / 2), p!(1 / 4)].iter().copied().sum::<Probability>();

    assert_eq!(sum, p!(3 / 4));
}

#[test]
fn test_sub1() {
    let a = p!(1 / 2);
    let b = p!(1 / 3);

    let c = a - b;

    assert_eq!(c, p!(1 / 6));
}

#[test]
fn test_sub2() {
    let a = p!(1 / 4);
    let b = p!(1 / 2);

    let c = a - b;

    assert_eq!(c, p!(-1 / 4));
}

#[test]
fn test_subref1() {
    let a = p!(1 / 2);
    let b = p!(1 / 3);

    let sub_by_ref = |a: Probability, b: &Probability| a - b;
    let c = sub_by_ref(a, &b);

    assert_eq!(c, p!(1 / 6));
}

#[test]
fn test_subref2() {
    let a = p!(1 / 4);
    let b = p!(1 / 2);

    let sub_by_ref = |a: Probability, b: &Probability| a - b;
    let c = sub_by_ref(a, &b);

    assert_eq!(c, p!(-1 / 4));
}

#[test]
fn test_subassign1() {
    let mut a = p!(1 / 2);
    let b = p!(1 / 3);

    a -= b;

    assert_eq!(a, p!(1 / 6));
}

#[test]
fn test_subassign2() {
    let mut a = p!(1 / 4);
    let b = p!(1 / 2);

    a -= b;

    assert_eq!(a, p!(-1 / 4));
}

#[test]
fn test_mul1() {
    let a = p!(1 / 2);
    let b = p!(1 / 3);

    let c = a * b;

    assert_eq!(c, p!(1 / 6));
}

#[test]
fn test_mul2() {
    let a = p!(2 / 3);
    let b = p!(3 / 6);

    let c = a * b;

    assert_eq!(c, p!(1 / 3));
}

#[test]
fn test_mulassign1() {
    let mut a = p!(1 / 2);
    let b = p!(1 / 3);

    a *= b;

    assert_eq!(a, p!(1 / 6));
}

#[test]
fn test_mulassign2() {
    let mut a = p!(2 / 3);
    let b = p!(3 / 6);

    a *= b;

    assert_eq!(a, p!(1 / 3));
}

#[test]
fn test_div1() {
    let a = p!(2 / 3);

    let c = a / 2;

    assert_eq!(c, p!(1 / 3));
}

#[test]
fn test_div2() {
    let a = p!(2 / 3);

    let c = a / 3;

    assert_eq!(c, p!(2 / 9));
}

#[test]
fn test_display1() {
    let a = p!(2 / 3);

    assert_eq!(a.to_string(), "2/3");
}

#[test]
fn test_display2() {
    let a = Probability::NEVER;

    assert_eq!(a.to_string(), "0");
}
