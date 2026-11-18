//! The functions that make an href from a template and a name.

use flatmarkdown_ast2html::fill_href;

// テンプレートの{}には、パーセント符号化した名前が入る。
#[test]
fn the_braces_of_a_template_take_the_name_percent_encoded() {
    assert_eq!(fill_href("/pages/{}", "What?"), "/pages/What%3F");
}
