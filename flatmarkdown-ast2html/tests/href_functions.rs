//! The public functions that make an href template and fill it, for apps that make the URL of
//! a page without rendering a wikilink.

use flatmarkdown_ast2html::{fill_href, href_template_from_prefix, split_wikilink};

const ITEM_ID: &str = "[0-9]{14}[-_][0-9A-Z]{7}";

// 前置きから作ったテンプレートは、前置きを符号化し、名前を入れる場所を後ろに付けたものである。
#[test]
fn a_template_from_a_prefix_is_the_encoded_prefix_followed_by_the_place_of_the_name() {
    assert_eq!(
        fill_href(&href_template_from_prefix("/pages/タグ/"), "Rust"),
        "/pages/%E3%82%BF%E3%82%B0/Rust"
    );
}

// 前置きの中の波かっこは、名前を入れる場所にならない。
#[test]
fn braces_in_a_prefix_do_not_take_the_name() {
    assert_eq!(
        fill_href(&href_template_from_prefix("/pages/{}/"), "Rust"),
        "/pages/%7B%7D/Rust"
    );
}

// fill_hrefの結果は、HTMLのエスケープにかけない。
#[test]
fn fill_href_does_not_escape_for_html() {
    assert_eq!(
        fill_href(&href_template_from_prefix("/pages/"), "A&B"),
        "/pages/A&B"
    );
}

// wikilinkのurlは、最後の#の後ろが形に合えば、名前とフラグメントに分かれる。
#[test]
fn a_wikilink_splits_at_the_last_hash_when_the_fragment_matches() {
    assert_eq!(
        split_wikilink("C##20260426035700_QZ998PX", ITEM_ID),
        ("C#", Some("20260426035700_QZ998PX"))
    );
}

// 最後の#の後ろが形に合わなければ、url全体が名前である。
#[test]
fn the_whole_url_is_the_name_when_the_fragment_does_not_match() {
    assert_eq!(split_wikilink("C#入門", ITEM_ID), ("C#入門", None));
}
