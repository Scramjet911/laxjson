use laxjson::{from_reader, from_slice, from_str, json, Deserializer, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn err(s: &str) -> String {
    from_str::<Value>(s).unwrap_err().to_string()
}

#[test]
fn object() {
    let s = r#"
    {
        "key": "value",
    }"#;
    assert_eq!(from_str::<Value>(s).unwrap(), json!({"key": "value"}));
}

#[test]
fn array() {
    let s = r#"
    {
        "key": [
            "one",
            "two",
            "three",
        ]
    }"#;
    assert_eq!(
        from_str::<Value>(s).unwrap(),
        json!({"key": ["one", "two", "three"]})
    );
}

#[test]
fn nested_and_whitespace() {
    let s = "[ [1, 2 , ] , {\"a\" : [ ] , } , ]";
    assert_eq!(from_str::<Value>(s).unwrap(), json!([[1, 2], {"a": []}]));
    assert_eq!(
        from_str::<Value>("[1 /* c */ , // d\n ]").unwrap(),
        json!([1])
    );
}

#[test]
fn double_comma_object() {
    let s = r#"
    {
        "key1": "value1",,
        "key2": "value2"
    }"#;
    assert_eq!(err(s), "key must be a string at line 3 column 26");
}

#[test]
fn double_trailing_comma_object() {
    let s = r#"
    {
        "key1": "value1",
        "key2": "value2",,
    }"#;
    assert_eq!(err(s), "key must be a string at line 4 column 26");
}

#[test]
fn double_comma_array() {
    let s = r#"
    {
        "key": [
            "one",,
            "two",
        ]
    }"#;
    assert_eq!(err(s), "expected value at line 4 column 19");
}

#[test]
fn double_trailing_comma_array() {
    let s = r#"
    {
        "key": [
            "one",
            "two",,
        ]
    }"#;
    assert_eq!(err(s), "expected value at line 5 column 19");
}

#[test]
fn lone_comma_still_errors() {
    assert_eq!(err("[,]"), "expected value at line 1 column 2");
    assert_eq!(err("{,}"), "key must be a string at line 1 column 2");
}

#[test]
fn comma_then_eof_still_errors() {
    assert!(from_str::<Value>("[1,").unwrap_err().is_eof());
    assert!(from_str::<Value>("{\"a\":1,").unwrap_err().is_eof());
}

#[test]
fn fixed_size_targets() {
    // Visitors that stop early leave `,]` for end_seq.
    assert_eq!(from_str::<(i32, i32)>("[1, 2,]").unwrap(), (1, 2));
    assert_eq!(from_str::<[i32; 2]>("[1, 2, ]").unwrap(), [1, 2]);
    assert!(from_str::<(i32, i32)>("[1, 2, 3]").is_err());
    assert!(from_str::<(i32, i32)>("[1, 2,,]").is_err());
}

#[test]
fn collections() {
    assert_eq!(from_str::<Vec<u8>>("[1, 2,]").unwrap(), [1, 2]);
    let m: HashMap<String, u8> = from_str(r#"{"a": 1, "b": 2,}"#).unwrap();
    assert_eq!(m.len(), 2);
}

#[derive(Debug, PartialEq, Deserialize)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn struct_and_unknown_fields() {
    let p: Point = from_str(r#"{"x": 1, "y": 2,}"#).unwrap();
    assert_eq!(p, Point { x: 1, y: 2 });
    // Skipped values go through ignore_value.
    let p: Point =
        from_str(r#"{"skip": [1, {"z": [2,],},], "x": 1, "y": 2, "tail": [],}"#).unwrap();
    assert_eq!(p, Point { x: 1, y: 2 });
}

#[test]
fn struct_from_seq() {
    let p: Point = from_str("[1, 2,]").unwrap();
    assert_eq!(p, Point { x: 1, y: 2 });
}

#[test]
fn ignored_value_errors() {
    for s in [
        r#"{"skip": [1,,], "x": 1, "y": 2}"#,
        r#"{"skip": {"a": 1,,}, "x": 1, "y": 2}"#,
        r#"{"skip": [,], "x": 1, "y": 2}"#,
        r#"{"skip": {,}, "x": 1, "y": 2}"#,
    ] {
        assert!(from_str::<Point>(s).is_err(), "{s}");
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },
    AntHive(Vec<String>),
}

#[test]
fn enum_as_array_with_deny_unknown_fields() {
    let animal: Animal = from_str(r#"{"Cat":[0, "Kate",]}"#).unwrap();
    assert_eq!(
        animal,
        Animal::Cat {
            age: 0,
            name: "Kate".to_string()
        }
    );
}

#[test]
fn enum_as_object_with_deny_unknown_fields() {
    let animal: Animal = from_str(r#"{"Cat":{"age": 2, "name": "Kate",}}"#).unwrap();
    assert_eq!(
        animal,
        Animal::Cat {
            age: 2,
            name: "Kate".to_string()
        }
    );
}

#[test]
fn enum_trailing_comma_after_variant() {
    let a: Animal = from_str(r#"{"Cat":{"age": 2, "name": "Kate"},}"#).unwrap();
    assert_eq!(
        a,
        Animal::Cat {
            age: 2,
            name: "Kate".to_string()
        }
    );
    let a: Animal = from_str(r#"{"Frog":["x",[1,],], }"#).unwrap();
    assert_eq!(a, Animal::Frog("x".to_string(), vec![1]));
    let a: Animal = from_str(r#"{"AntHive":["a",],}"#).unwrap();
    assert_eq!(a, Animal::AntHive(vec!["a".to_string()]));
}

#[test]
fn enum_comma_errors() {
    assert!(from_str::<Animal>(r#"{"Cat":[0,"K"],,}"#).is_err());
    assert!(from_str::<Animal>(r#"{"AntHive":[],"Dog":null}"#).is_err());
    assert!(from_str::<Animal>(r#"{"AntHive":[],"#).is_err());
}

#[test]
fn other_readers() {
    let s = r#"{"a": [1, 2,],}"#;
    let want = json!({"a": [1, 2]});
    assert_eq!(from_slice::<Value>(s.as_bytes()).unwrap(), want);
    assert_eq!(from_reader::<_, Value>(s.as_bytes()).unwrap(), want);
}

#[test]
fn stream() {
    let got: Vec<Value> = Deserializer::from_str("[1,] {\"a\":2,}[3,]")
        .into_iter::<Value>()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(got, [json!([1]), json!({"a": 2}), json!([3])]);
}

#[cfg(feature = "raw_value")]
#[test]
fn raw_value_keeps_input() {
    #[derive(Deserialize)]
    struct W {
        raw: Box<laxjson::value::RawValue>,
    }
    let w: W = from_str(r#"{"raw": [1, {"a": 2,},],}"#).unwrap();
    assert_eq!(w.raw.get(), r#"[1, {"a": 2,},]"#);
}
