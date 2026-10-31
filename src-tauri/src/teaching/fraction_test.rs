use super::*;

#[test]
fn test_fraction_basic_ops() {
    let f1 = ExactFraction::new(1, 2);
    let f2 = ExactFraction::new(1, 3);
    let sum = f1 + f2;
    assert_eq!(sum, ExactFraction::new(5, 6));

    let diff = f1 - f2;
    assert_eq!(diff, ExactFraction::new(1, 6));

    let prod = f1 * f2;
    assert_eq!(prod, ExactFraction::new(1, 6));

    let quot = f1 / f2;
    assert_eq!(quot, ExactFraction::new(3, 2));
}

#[test]
fn test_fraction_simplification_and_latex() {
    let f = ExactFraction::new(6, 8);
    assert_eq!(f.num, 3);
    assert_eq!(f.den, 4);
    assert_eq!(f.to_latex(), "\\frac{3}{4}");

    let neg_f = ExactFraction::new(-5, 10);
    assert_eq!(neg_f.to_latex(), "-\\frac{1}{2}");

    let int_f = ExactFraction::new(12, 4);
    assert_eq!(int_f.to_latex(), "3");

    let zero_f = ExactFraction::new(0, 5);
    assert_eq!(zero_f.to_latex(), "0");
}

#[test]
fn test_fraction_f64_approx() {
    let f = ExactFraction::from_f64_approx(0.75);
    assert_eq!(f, ExactFraction::new(3, 4));

    let f2 = ExactFraction::from_f64_approx(0.33333333);
    assert_eq!(f2, ExactFraction::new(1, 3));

    let f3 = ExactFraction::from_f64_approx(12.5);
    assert_eq!(f3, ExactFraction::new(25, 2));
}
