//! `ix` is the index of a footnote in its body. A footnote without its other side in the same
//! body is completed with a placeholder while the parser works. That placeholder must not
//! take a number.

use flatmarkdown::body_to_ast;
use flatmarkdown_ast::{Node, parse};

/// The name and the `ix` of every footnote reference, in the order of the tree.
fn references(md: &str) -> Vec<(String, u32)> {
    let root = parse(&body_to_ast(md)).unwrap();
    let mut found = Vec::new();
    root.walk(&mut |node| {
        if let Node::FootnoteReference { name, ix, .. } = node {
            found.push((name.clone(), *ix));
        }
    });
    found
}

fn named(pairs: &[(&str, u32)]) -> Vec<(String, u32)> {
    pairs.iter().map(|(n, ix)| (n.to_string(), *ix)).collect()
}

// 参照のない定義が同じ本文にあっても、最初の参照のixは1である。
#[test]
fn a_definition_without_a_reference_takes_no_number() {
    assert_eq!(
        references("[^d]: lone\n\nText[^r].\n\n[^r]: paired"),
        named(&[("r", 1)])
    );
}

// 参照のない定義がいくつあっても、ixは1から続けて数える。
#[test]
fn the_numbers_have_no_gap_beside_several_lone_definitions() {
    assert_eq!(
        references("[^d]: lone\n\n[^e]: lone\n\nA[^r] B^[in] C[^q] D[^r].\n\n[^r]: paired"),
        named(&[("r", 1), ("__inline_1", 2), ("q", 3), ("r", 1)])
    );
}

// 参照のない定義の中にある参照も、1から数える。
#[test]
fn a_reference_inside_a_lone_definition_is_numbered_from_one() {
    assert_eq!(references("[^a]: See [^b]."), named(&[("b", 1)]));
}
