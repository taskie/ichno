#[macro_use]
extern crate optional_derive;

#[derive(Debug, PartialEq, Optional)]
struct S1 {
    pub x: i32,
    pub y: Option<i32>,
}

#[derive(Default, Debug, PartialEq, Optional)]
#[optional(name = "OptionalS2", derive = "Default, Debug, PartialEq")]
struct S2 {
    #[optional(name = "z")]
    pub x: i32,
    #[optional(skip = true)]
    pub y: Option<i32>,
    #[optional(name = "x", required = true)]
    pub z: bool,
}

#[test]
fn test_s1() {
    let _s1 = S1 { x: 42, y: Some(42) };
    let _s1_opt = S1Opt { x: Some(42), y: Some(Some(42)) };
}

#[test]
fn test_s2() {
    let _s2 = S2 { x: 42, y: Some(42), z: true };
    let _s2_opt = OptionalS2 { x: true, z: Some(42) };
    format!("{:?}", _s2_opt);
    assert_eq!(OptionalS2 { x: true, z: Some(42) }, _s2_opt);
}

#[test]
fn test_s2_default() {
    assert_eq!(S2 { x: 0, y: None, z: false }, Default::default());
    assert_eq!(OptionalS2 { x: false, z: None }, Default::default());
}
