use laxjson::{from_reader, from_slice, from_str, json, Deserializer, Value};

#[test]
fn line_comments() {
    let s = r#"
    {
        // Here is a comment.
        "key": "value"
    }// Another one at the end."#;
    assert_eq!(from_str::<Value>(s).unwrap(), json!({"key": "value"}));
}

#[test]
fn line_comment_ends_at_cr() {
    assert_eq!(from_str::<Value>("// c\r1").unwrap(), json!(1));
    assert_eq!(from_str::<Value>("// c\r\n1").unwrap(), json!(1));
}

#[test]
fn line_comment_at_eof() {
    assert_eq!(from_str::<Value>("1 //").unwrap(), json!(1));
    assert_eq!(from_str::<Value>("1 // c").unwrap(), json!(1));
}

#[test]
fn block_comments() {
    let s = r#"
    /*
     Comment up here.
     */
    {
        /* And one in here. */
        "key": "value"
    }/* end *//* ...back to *** back! */ /* Two trailing asterisks **/"#;
    assert_eq!(from_str::<Value>(s).unwrap(), json!({"key": "value"}));
}

#[test]
fn block_comment_asterisk_edges() {
    for s in ["/**/1", "/***/1", "/* a **/1", "/* a ***/1", "/* * / */1"] {
        assert_eq!(from_str::<Value>(s).unwrap(), json!(1), "{s}");
    }
}

#[test]
fn comments_between_tokens() {
    let s = r#"{ /*a*/ "k" /*b*/ : /*c*/ [ /*d*/ 1 /*e*/ , // f
        2 /*g*/ ] /*h*/ , "z" : { // i
        } }"#;
    assert_eq!(from_str::<Value>(s).unwrap(), json!({"k": [1, 2], "z": {}}));
}

#[test]
fn comments_in_empty_containers() {
    assert_eq!(from_str::<Value>("[ /* x */ ]").unwrap(), json!([]));
    assert_eq!(from_str::<Value>("{ // x\n }").unwrap(), json!({}));
}

#[test]
fn comment_markers_in_strings_untouched() {
    let v: Value = from_str(r#"["// no", "/* no */", "a/b"]"#).unwrap();
    assert_eq!(v, json!(["// no", "/* no */", "a/b"]));
}

#[test]
fn comments_with_utf8() {
    assert_eq!(from_str::<Value>("// héllo ✓\n1").unwrap(), json!(1));
    assert_eq!(from_str::<Value>("/* héllo ✓ */1").unwrap(), json!(1));
}

#[test]
fn comments_in_other_readers() {
    let s = "/* a */ [1, // b\n 2]";
    assert_eq!(from_slice::<Value>(s.as_bytes()).unwrap(), json!([1, 2]));
    assert_eq!(
        from_reader::<_, Value>(s.as_bytes()).unwrap(),
        json!([1, 2])
    );
}

#[test]
fn comments_in_stream() {
    let s = "1/*a*/2 //b\n3";
    let got: Vec<Value> = Deserializer::from_str(s)
        .into_iter::<Value>()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(got, [json!(1), json!(2), json!(3)]);
}

#[test]
fn comment_in_skipped_value() {
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct S {
        a: u8,
    }
    let s: S = from_str(
        r#"{"skip": [1, /* x */ {"y": 2}], "a": 1 // z
    }"#,
    )
    .unwrap();
    assert_eq!(s, S { a: 1 });
}

#[test]
fn unterminated_block_comment() {
    let err = from_str::<Value>("1 /* abc").unwrap_err();
    assert_eq!(
        err.to_string(),
        "EOF while parsing a block comment at line 1 column 8"
    );
    assert!(err.is_eof());
    assert!(from_str::<Value>("/*").unwrap_err().is_eof());
    assert!(from_str::<Value>("/* *").unwrap_err().is_eof());
}

#[test]
fn lone_slash() {
    let err = from_str::<Value>("1 /").unwrap_err();
    assert_eq!(
        err.to_string(),
        "expected `/` or `*` after `/` at line 1 column 3"
    );
    assert!(err.is_syntax());
    let err = from_str::<Value>("[1, /x]").unwrap_err();
    assert_eq!(
        err.to_string(),
        "expected `/` or `*` after `/` at line 1 column 6"
    );
}

#[test]
fn block_comments_do_not_nest() {
    let err = from_str::<Value>("/* /* */ */ 1").unwrap_err();
    assert!(err.is_syntax());
}

#[test]
fn comment_only_input() {
    let err = from_str::<Value>("// nothing").unwrap_err();
    assert!(err.is_eof());
}
