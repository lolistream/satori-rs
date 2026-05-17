//! CSS custom properties (CSS variables) — extraction and `var()` substitution.
//!
//! Port of `src/handler/variables.ts`. Implements:
//!   * Extraction of `--foo: value` declarations from a raw JS-style style
//!     object.
//!   * Substitution of `var(--name)` and `var(--name, fallback)` references
//!     in CSS value strings, including recursive resolution and circular
//!     reference detection.
//!
//! Semantics mirror the upstream JS implementation, with two minor
//! deviations called out in the porting task:
//!   * `var(--undefined)` without a fallback resolves to an empty string
//!     (instead of the literal `initial` keyword the JS code emits).
//!   * Custom-property *names* are case-sensitive (per the CSS spec).

use indexmap::IndexMap;
use std::collections::HashSet;

/// Ordered map of `--name -> value` declarations. Insertion order matters so
/// that nested variables resolve in the order they were declared (which
/// mirrors JS object iteration order).
pub type Vars = IndexMap<String, String>;

/// Pull every `--foo: value` declaration out of a raw JS-style style object.
///
/// Non-object inputs (or absent fields) return an empty map. Numeric values
/// are coerced to their string form so callers can substitute them into
/// dimension/color/etc. fields uniformly.
pub fn extract_vars(raw: &serde_json::Value) -> Vars {
    let mut vars = Vars::new();
    let Some(obj) = raw.as_object() else {
        return vars;
    };
    for (k, v) in obj {
        if !k.starts_with("--") {
            continue;
        }
        let s = match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        vars.insert(k.clone(), s);
    }
    vars
}

/// Merge two variable scopes — `child` overrides `parent` on conflict,
/// matching the JS `mergeVariables` helper.
pub fn merge_vars(parent: &Vars, child: &Vars) -> Vars {
    let mut out = parent.clone();
    for (k, v) in child {
        out.insert(k.clone(), v.clone());
    }
    out
}

/// Recursively substitute every `var(--name)` / `var(--name, fallback)`
/// reference in `input`, returning the resolved string.
///
/// Unknown variables with no fallback resolve to an empty string; the
/// fallback is itself substituted before being inlined. Circular references
/// are short-circuited (the offending `var()` resolves to empty / its
/// fallback) instead of recursing forever.
pub fn substitute(input: &str, vars: &Vars) -> String {
    if !input.contains("var(") {
        return input.to_string();
    }
    let mut visited: HashSet<String> = HashSet::new();
    substitute_inner(input, vars, &mut visited)
}

fn substitute_inner(input: &str, vars: &Vars, visited: &mut HashSet<String>) -> String {
    if !input.contains("var(") {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    loop {
        let Some(start) = find_var_start(rest) else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..start]);
        let after_open = start + 4; // "var(" len in ASCII bytes
        let inner = &rest[after_open..];
        let Some(close) = find_matching_close(inner) else {
            // Unbalanced — emit the rest verbatim and stop.
            out.push_str(&rest[start..]);
            return out;
        };
        let arg_str = &inner[..close];
        let (name_part, fallback_part) = split_first_top_level_comma(arg_str);
        let var_name = name_part.trim().to_string();
        let resolved = resolve_var(&var_name, fallback_part, vars, visited);
        out.push_str(&resolved);
        rest = &inner[close + 1..];
    }
}

fn resolve_var(
    name: &str,
    fallback: Option<&str>,
    vars: &Vars,
    visited: &mut HashSet<String>,
) -> String {
    if visited.contains(name) {
        return match fallback {
            Some(f) => substitute_inner(f.trim(), vars, visited),
            None => String::new(),
        };
    }
    match vars.get(name) {
        Some(val) => {
            visited.insert(name.to_string());
            let r = substitute_inner(val, vars, visited);
            visited.remove(name);
            r
        }
        None => match fallback {
            Some(f) => substitute_inner(f.trim(), vars, visited),
            None => String::new(),
        },
    }
}

/// Find the byte offset of the next `var(` token that is *not* part of a
/// longer identifier (so `mvar(` won't match).
fn find_var_start(s: &str) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = s[search_from..].find("var(") {
        let abs = search_from + pos;
        let prev_ok = abs == 0
            || !s[..abs]
                .chars()
                .next_back()
                .map(is_ident_continue)
                .unwrap_or(false);
        if prev_ok {
            return Some(abs);
        }
        search_from = abs + 1;
    }
    None
}

fn find_matching_close(s: &str) -> Option<usize> {
    let mut depth = 1usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_first_top_level_comma(s: &str) -> (&str, Option<&str>) {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            ',' if depth == 0 => return (&s[..i], Some(&s[i + 1..])),
            _ => {}
        }
    }
    (s, None)
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn vars_from(pairs: &[(&str, &str)]) -> Vars {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn extracts_only_custom_properties() {
        let raw = json!({
            "--foo": "red",
            "color": "blue",
            "--bar": 42,
        });
        let vars = extract_vars(&raw);
        assert_eq!(vars.get("--foo").map(String::as_str), Some("red"));
        assert_eq!(vars.get("--bar").map(String::as_str), Some("42"));
        assert!(vars.get("color").is_none());
    }

    #[test]
    fn substitute_basic() {
        let v = vars_from(&[("--c", "red")]);
        assert_eq!(substitute("var(--c)", &v), "red");
    }

    #[test]
    fn substitute_fallback_used_when_missing() {
        let v = Vars::new();
        assert_eq!(substitute("var(--missing, yellow)", &v), "yellow");
    }

    #[test]
    fn substitute_missing_without_fallback_is_empty() {
        let v = Vars::new();
        assert_eq!(substitute("var(--missing)", &v), "");
    }

    #[test]
    fn substitute_nested_variable_value() {
        let v = vars_from(&[("--base", "purple"), ("--primary", "var(--base)")]);
        assert_eq!(substitute("var(--primary)", &v), "purple");
    }

    #[test]
    fn substitute_fallback_chain() {
        let v = vars_from(&[("--fb", "pink")]);
        assert_eq!(
            substitute("var(--undef, var(--fb))", &v),
            "pink"
        );
    }

    #[test]
    fn substitute_tolerates_whitespace() {
        let v = vars_from(&[("--c", "red")]);
        assert_eq!(substitute("var( --c )", &v), "red");
        assert_eq!(
            substitute("var( --undef ,  green )", &v),
            "green"
        );
    }

    #[test]
    fn substitute_handles_circular_with_fallback() {
        let v = vars_from(&[("--a", "var(--a, blue)")]);
        assert_eq!(substitute("var(--a)", &v), "blue");
    }

    #[test]
    fn substitute_handles_circular_without_fallback() {
        let v = vars_from(&[("--a", "var(--a)")]);
        assert_eq!(substitute("var(--a)", &v), "");
    }

    #[test]
    fn substitute_leaves_non_var_text_alone() {
        let v = Vars::new();
        assert_eq!(substitute("10px solid red", &v), "10px solid red");
    }

    #[test]
    fn substitute_partial_replacement() {
        let v = vars_from(&[("--w", "5px")]);
        assert_eq!(
            substitute("var(--w) solid #333", &v),
            "5px solid #333"
        );
    }
}
