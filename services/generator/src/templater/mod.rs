//! Steel Scheme template engine.
//! Templates are `.scm` files that call registered Rust functions to build
//! a `Node` tree. The engine evaluates the script and extracts the final
//! `SchemeNode` from the result.

mod types;
mod functions;

use crate::primitives::node::Node;
use steel::rvals::{FromSteelVal, SteelVal};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
pub use types::SchemeNode;

pub struct Templater {
    engine: Engine,
}

impl Templater {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        functions::register_all(&mut engine);
        Self { engine }
    }

    pub fn render_template(
        &mut self,
        template_src: &str,
        payload_json: &str,
        content_opt: Option<String>,
        entities_opt: Option<Vec<serde_json::Value>>,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let mut payload_val: serde_json::Value = serde_json::from_str(payload_json)?;
        
        // Calculate avatar initials and gradient colors in Rust
        if let Some(obj) = payload_val.as_object_mut() {
            let username = obj.get("username").and_then(|v| v.as_str()).unwrap_or("");
            let initials = username
                .split_whitespace()
                .take(2)
                .filter_map(|s| s.chars().next())
                .collect::<String>()
                .to_uppercase();
            obj.insert("avatar_initials".to_string(), serde_json::Value::String(initials));
            
            let grad_id = obj.get("grad_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let colors = match grad_id % 7 {
                0 => ("#FF516A", "#FF885E"), // Red
                1 => ("#FFA85C", "#FFCD6A"), // Orange
                2 => ("#8C79F2", "#B37DF2"), // Purple
                3 => ("#51BB3F", "#8AE451"), // Green
                4 => ("#34C6CD", "#4CE9C2"), // Cyan
                5 => ("#549CFF", "#3CB9FE"), // Blue
                _ => ("#F2799B", "#F27DF2"), // Pink (6)
            };
            obj.insert("avatar_color_top".to_string(), serde_json::Value::String(colors.0.to_string()));
            obj.insert("avatar_color_bottom".to_string(), serde_json::Value::String(colors.1.to_string()));
        }
        
        let assoc_str = json_to_scheme(&payload_val);
        
        self.engine.register_fn("%make-text", move |content: SteelVal, mods: SteelVal| {
            functions::fn_make_text(content, mods, &content_opt, &entities_opt)
        });

        let script = format!(
            "(define payload {})\n\
             (define (get-payload key default-val)\n\
               (let ((found (assoc key payload)))\n\
                 (if (and found (not (null? (cdr found))))\n\
                     (cdr found)\n\
                     default-val)))\n\
             (define (text content . mods) (%make-text content mods))\n\
             {}",
            assoc_str, template_src
        );

        let res = self.engine.compile_and_run_raw_program(script)?;
        let last_val = res
            .last()
            .ok_or("Template returned no value")?;

        extract_node(last_val)
    }
}

/// Extract a `Node` from the final `SteelVal` returned by the template.
fn extract_node(val: &SteelVal) -> Result<Node, Box<dyn std::error::Error>> {
    let scheme_node = SchemeNode::from_steelval(val)
        .map_err(|e| format!("Template must return a (node ...), got: {:?} ({})", val, e))?;
    Ok(scheme_node.0)
}

/// Convert a serde_json::Value into a Steel Scheme association list literal.
fn json_to_scheme(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "'()".to_string(),
        serde_json::Value::Bool(b) => {
            if *b {
                "#t".to_string()
            } else {
                "#f".to_string()
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\"")),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_scheme).collect();
            format!("(list {})", items.join(" "))
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("(cons '{} {})", k, json_to_scheme(v)))
                .collect();
            format!("(list {})", items.join(" "))
        }
    }
}
