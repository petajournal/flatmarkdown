use flatmarkdown::page_to_ast;

// メタデータ行は、次のブロックにレベル、ID、プロパティを与える。メタデータ行のないブロックはレベル1である。
#[test]
fn page_items_with_metadata() {
    let input = "<!-- @@ 2026-09-27-15-00-16-W3N8ASY plan=\"2026-09-28\" -->\n- [ ] Task1\n\nPlain paragraph\n";
    let v: serde_json::Value = serde_json::from_str(&page_to_ast(input).to_json()).unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["level"], 2);
    assert_eq!(items[0]["id"], "2026-09-27-15-00-16-W3N8ASY");
    assert_eq!(items[0]["properties"]["plan"], "2026-09-28");
    assert_eq!(items[0]["body"], "- [ ] Task1");
    assert_eq!(items[1]["level"], 1);
    assert_eq!(items[1]["body"], "Plain paragraph");
}
