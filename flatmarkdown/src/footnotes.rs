//! Keeps footnote references and definitions that have no partner in the same item.

use serde_json::Value;
use std::collections::HashSet;

/// Marks the placeholder text that completes unpaired footnotes before parsing.
/// A noncharacter is used because it cannot occur in valid text.
const FOOTNOTE_PLACEHOLDER: char = '\u{FDD1}';

/// Footnote labels found by a raw scan of a body: `[^label]` occurrences and
/// `[^label]:` definition lines. The scan does not know about code spans or
/// code blocks, so a label may be a false positive. That is harmless, because the
/// completion only adds placeholders that comrak drops again when they stay unpaired.
struct FootnoteLabels {
    refs: Vec<String>,
    defs: Vec<String>,
}

/// Reads a footnote label after `[^` at `start`. Returns the label and the index
/// just past the closing `]`. Labels may not be empty or contain whitespace or `[`.
fn read_footnote_label(s: &str, start: usize) -> Option<(&str, usize)> {
    let rest = &s[start..];
    let end = rest.find(']')?;
    let label = &rest[..end];
    if label.is_empty() || label.chars().any(|c| c.is_whitespace() || c == '[') {
        return None;
    }
    Some((label, start + end + 1))
}

fn scan_footnote_labels(input: &str) -> FootnoteLabels {
    let mut labels = FootnoteLabels {
        refs: Vec::new(),
        defs: Vec::new(),
    };
    for line in input.lines() {
        // A definition may be indented by up to three spaces, like any block start.
        let trimmed = line.trim_start_matches(' ');
        let indent = line.len() - trimmed.len();
        let def_label = if indent <= 3 && trimmed.starts_with("[^") {
            read_footnote_label(trimmed, 2)
                .filter(|(_, end)| trimmed[*end..].starts_with(':'))
                .map(|(label, _)| label.to_string())
        } else {
            None
        };

        let mut pos = 0;
        while let Some(found) = line[pos..].find("[^") {
            let label_start = pos + found + 2;
            match read_footnote_label(line, label_start) {
                Some((label, end)) => {
                    let is_definition =
                        def_label.as_deref() == Some(label) && label_start == indent + 2;
                    if !is_definition {
                        labels.refs.push(label.to_string());
                    }
                    pos = end;
                }
                None => pos = label_start,
            }
        }
        if let Some(label) = def_label {
            labels.defs.push(label);
        }
    }
    labels
}

/// Collects the `name` of every node of `node_type` in the AST.
fn collect_footnote_names(node: &Value, node_type: &str, out: &mut HashSet<String>) {
    if node.get("type").and_then(|t| t.as_str()) == Some(node_type)
        && let Some(name) = node.get("name").and_then(|n| n.as_str())
    {
        out.insert(name.to_string());
    }
    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for child in children {
            collect_footnote_names(child, node_type, out);
        }
    }
}

/// Builds the text that completes the unpaired footnotes of `input`, or returns
/// `None` when every footnote is already paired.
///
/// comrak keeps a footnote only when its reference and its definition are in the
/// same document. An item body is parsed alone, so a body that holds only one of
/// the two loses it. The missing side is added as a placeholder in front of the
/// body: placeholder definitions first, then one paragraph holding the placeholder
/// references. The paragraph also separates the definitions from the body, so an
/// indented first line of the body cannot be read as a definition continuation.
/// Placing the placeholders after the body is not an option, because a body that
/// ends in an unclosed code fence would swallow them.
pub(crate) fn complete_footnotes(input: &str, ast: &Value) -> Option<(String, HashSet<String>)> {
    let labels = scan_footnote_labels(input);
    let mut real_defs = HashSet::new();
    let mut real_refs = HashSet::new();
    collect_footnote_names(ast, "footnote_definition", &mut real_defs);
    collect_footnote_names(ast, "footnote_reference", &mut real_refs);

    let mut missing_defs: Vec<String> = Vec::new();
    for label in labels.refs {
        if !real_defs.contains(&label) && !missing_defs.contains(&label) {
            missing_defs.push(label);
        }
    }
    let mut missing_refs: Vec<String> = Vec::new();
    for label in labels.defs {
        if !real_refs.contains(&label) && !missing_refs.contains(&label) {
            missing_refs.push(label);
        }
    }
    if missing_defs.is_empty() && missing_refs.is_empty() {
        return None;
    }

    let mut completed = String::new();
    for label in &missing_defs {
        completed.push_str(&format!("[^{label}]: {FOOTNOTE_PLACEHOLDER}\n\n"));
    }
    completed.push(FOOTNOTE_PLACEHOLDER);
    for label in &missing_refs {
        completed.push_str(&format!("[^{label}]"));
    }
    completed.push_str("\n\n");
    completed.push_str(input);
    Some((completed, missing_defs.into_iter().collect()))
}

/// Removes the nodes that the placeholders of `complete_footnotes` produced.
pub(crate) fn strip_footnote_placeholders(ast: &mut Value, placeholder_defs: &HashSet<String>) {
    let Some(Value::Array(children)) = ast.get_mut("children") else {
        return;
    };
    children.retain(|child| {
        let node_type = child.get("type").and_then(|t| t.as_str());
        match node_type {
            Some("footnote_definition") => {
                let name = child.get("name").and_then(|n| n.as_str()).unwrap_or("");
                !placeholder_defs.contains(name)
            }
            Some("paragraph") => {
                let first_text = child
                    .get("children")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|n| n.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                !first_text.starts_with(FOOTNOTE_PLACEHOLDER)
            }
            _ => true,
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::body_to_ast;
    use crate::test_support::page;

    // ---- Footnotes split across items ----

    fn footnote_nodes(v: &Value, node_type: &str) -> Vec<String> {
        let mut out = HashSet::new();
        collect_footnote_names(v, node_type, &mut out);
        let mut names: Vec<String> = out.into_iter().collect();
        names.sort();
        names
    }

    fn contains_placeholder(v: &Value) -> bool {
        v.to_string().contains(FOOTNOTE_PLACEHOLDER)
    }

    #[test]
    fn ast_footnote_reference_without_definition_is_kept() {
        let v: Value = serde_json::from_str(&body_to_ast("Text[^note].")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["note"]);
        assert!(footnote_nodes(&v, "footnote_definition").is_empty());
        assert_eq!(v["children"].as_array().unwrap().len(), 1);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_definition_without_reference_is_kept() {
        let v: Value = serde_json::from_str(&body_to_ast("[^note]: The note.")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["note"]);
        assert!(footnote_nodes(&v, "footnote_reference").is_empty());
        let blocks = v["children"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0]["children"][0]["children"][0]["value"],
            "The note."
        );
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_paired_in_one_body_is_unchanged() {
        let v: Value = serde_json::from_str(&body_to_ast("Text[^1].\n\n[^1]: Footnote.")).unwrap();
        assert_eq!(v["children"].as_array().unwrap().len(), 2);
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["1"]);
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["1"]);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_definition_may_reference_another_footnote() {
        let v: Value = serde_json::from_str(&body_to_ast("[^a]: See [^b].")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["a"]);
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["b"]);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_syntax_in_code_stays_text() {
        let v: Value = serde_json::from_str(&body_to_ast("`[^1]`\n\n```\n[^2]: x\n```")).unwrap();
        assert!(footnote_nodes(&v, "footnote_reference").is_empty());
        assert!(footnote_nodes(&v, "footnote_definition").is_empty());
        assert_eq!(v["children"].as_array().unwrap().len(), 2);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_reference_before_unclosed_fence() {
        let v: Value = serde_json::from_str(&body_to_ast("Text[^1].\n\n```\ncode")).unwrap();
        assert_eq!(footnote_nodes(&v, "footnote_reference"), vec!["1"]);
        assert_eq!(v["children"][1]["type"], "code_block");
        assert_eq!(v["children"][1]["literal"], "code");
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn ast_footnote_definition_before_indented_code() {
        let v: Value = serde_json::from_str(&body_to_ast("    code\n\n[^1]: note")).unwrap();
        assert_eq!(v["children"][0]["type"], "code_block");
        assert_eq!(v["children"][0]["literal"], "code\n");
        assert_eq!(footnote_nodes(&v, "footnote_definition"), vec!["1"]);
        assert!(!contains_placeholder(&v));
    }

    #[test]
    fn page_footnote_definition_after_another_block_is_not_duplicated() {
        let v = page("Text[^1].\n\nMiddle.\n\n[^1]: Footnote.\n\nLast.\n");
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["body"], "Text[^1].");
        assert_eq!(items[1]["body"], "Middle.\n\n[^1]: Footnote.");
        assert_eq!(items[2]["body"], "Last.");
        assert_eq!(
            footnote_nodes(&items[0]["ast"], "footnote_reference"),
            vec!["1"]
        );
        assert_eq!(
            footnote_nodes(&items[1]["ast"], "footnote_definition"),
            vec!["1"]
        );
    }
}
