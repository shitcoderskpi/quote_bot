//! Steel Scheme template engine.
//! Templates are `.scm` files that call registered Rust functions to build
//! a `Node` tree. The engine evaluates the script and extracts the final
//! `SchemeNode` from the result.

mod types;
mod functions;

use crate::primitives::node::Node;
use steel::rvals::{FromSteelVal, SteelVal};
use steel::steel_vm::engine::Engine;

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
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let payload_val: serde_json::Value = serde_json::from_str(payload_json)?;
        let assoc_str = json_to_scheme(&payload_val);

        let script = format!(
            "(define payload {})\n\
             (define (get-payload key default-val)\n\
               (let ((found (assoc key payload)))\n\
                 (if (and found (not (null? (cdr found))))\n\
                     (cdr found)\n\
                     default-val)))\n\
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
