//! The properties of an embed come from the fragment of its URL (`#key=value&flag`). These
//! tests check what the parser does with the unusual forms.

use flatmarkdown::parse_body;
use serde_json::{Value, json};

/// The `embed` node of a body that holds only one image.
fn embed(md: &str) -> Value {
    let root: Value = serde_json::from_str(&parse_body(md).to_json()).unwrap();
    let node = root["children"][0]["children"][0].clone();
    assert_eq!(node["type"], "embed", "{md:?}");
    node
}

// キーが空のプロパティ（=3）は捨てる。名前がないと、レンダラーが読めないからである。
#[test]
fn a_property_with_an_empty_key_is_dropped() {
    let node = embed("![a](u.png#w=1&=3)");
    assert_eq!(node["props"], json!({"w": "1"}));
}

// キーが空のプロパティしかなければ、propsは省かれる。#だけのURLと同じである。
#[test]
fn props_are_left_out_when_only_properties_with_an_empty_key_are_written() {
    for md in ["![a](u.png#=3)", "![a](u.png#=)", "![a](u.png#)"] {
        let node = embed(md);
        assert_eq!(node["url"], "u.png", "{md:?}");
        assert!(node.get("props").is_none(), "{md:?}");
    }
}

// 同じキーを2回書くと、後の値だけが残る。値のないフラグが後なら、trueになる。
#[test]
fn a_key_written_twice_keeps_its_last_value() {
    assert_eq!(embed("![a](u.png#w=1&w=2)")["props"], json!({"w": "2"}));
    assert_eq!(embed("![a](u.png#w=1&w)")["props"], json!({"w": true}));
}

// =で終わるプロパティは、値が空文字列になる。=のないフラグのtrueとは別である。
#[test]
fn a_property_with_an_empty_value_is_an_empty_string() {
    assert_eq!(
        embed("![a](u.png#k=&flag)")["props"],
        json!({"k": "", "flag": true})
    );
}

// &が続いても、間の何もない区切りはプロパティにならない。
#[test]
fn an_empty_part_between_two_ampersands_is_no_property() {
    assert_eq!(
        embed("![a](u.png#a=1&&b=2&)")["props"],
        json!({"a": "1", "b": "2"})
    );
}
