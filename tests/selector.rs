extern crate jsonpath_lib as jsonpath;
#[macro_use]
extern crate serde_json;

use common::{read_json, setup};
use jsonpath::{JsonPathError, Parser, Selector, SelectorMut};
use serde_json::Value;

mod common;

#[test]
fn selector_mut() {
    setup();

    let mut selector_mut = SelectorMut::default();

    let mut nums = Vec::new();
    let result = selector_mut
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(read_json("./benchmark/example.json"))
        .replace_with(&mut |v| {
            if let Value::Number(n) = v {
                nums.push(n.as_f64().unwrap());
            }
            Value::String("a".to_string())
        })
        .unwrap()
        .take()
        .unwrap();

    assert_eq!(
        nums,
        vec![8.95_f64, 12.99_f64, 8.99_f64, 22.99_f64, 19.95_f64]
    );

    let mut selector = Selector::default();
    let result = selector
        .str_path(r#"$.store..price"#)
        .unwrap()
        .value(&result)
        .select()
        .unwrap();

    assert_eq!(
        vec![
            &json!("a"),
            &json!("a"),
            &json!("a"),
            &json!("a"),
            &json!("a")
        ],
        result
    );
}

#[test]
fn selector_mut_try_replace_with_err() {
    setup();

    let mut selector = SelectorMut::default();
    let err_msg = "unexpected value";
    let input = json!({
            "school": {
                "friends": [
                    {"name": "친구1", "age": 20},
                    {"name": "친구2", "age": 20},
                ],
            },
            "friends": [
                {"name": "친구3", "age": 30},
                {"name": "친구4"},
            ],
        });
    let e = selector
        .str_path("$..friends.*")
        .unwrap()
        .value(input.clone())
        .try_replace_with(&mut |v| {
            match v.get("age") {
                Some(Value::Number(age)) if age.is_u64() => {
                    let mut v = v.clone();
                    if let Value::Object(map) = &mut v {
                        map.insert("age".to_string(), json!(age.as_u64()));
                    }
                    return Some(Ok(v));
                }
                _ => Some(Err(JsonPathError::Unexpected(err_msg.to_string())))
            }
        });

    if let Err(JsonPathError::Unexpected(s)) = e {
        assert_eq!(s, err_msg);
        assert_eq!(selector.take().unwrap(), input);
    } else {
        panic!(false);
    }
}

#[test]
fn selector_mut_try_replace_with_skip() {
    setup();

    let mut selector = SelectorMut::default();
    match selector
        .str_path("$..a")
        .unwrap()
        .value(json!({
            "a": "maybe_changed",
            "b": {
                "a": "keep"
            }
        }))
        .try_replace_with(&mut |v| {
            if v.eq(&json!("keep")) {
                None
            } else {
                Some(Ok(json!("changed")))
            }
        }) {
        Ok(selector) => {
            let ret = selector.take().unwrap();
            assert_eq!(
                ret,
                json!({
                    "a": "changed",
                    "b": {
                        "a": "keep"
                    }
                })
            );
        }
        _ => panic!(false),
    };
}

#[test]
fn selector_node_ref() {
    let node = Parser::compile("$.*").unwrap();
    let mut selector = Selector::default();
    selector.compiled_path(&node);
    assert!(std::ptr::eq(selector.node_ref().unwrap(), &node));
}
