mod types;
mod functions;

use steel::compiler::program::Executable;
use crate::primitives::node::Node;
use steel::SteelErr;
use steel::gc::Gc;
use steel::rvals::IntoSteelVal;
use steel::HashMap;
use steel::rvals::{FromSteelVal, SteelVal, SteelString};
use steel::steel_vm::engine::Engine;
pub use types::SchemeNode;
use types::TextContext;

pub struct Templater {
    engine: Engine,
}

impl Templater {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        functions::register_all(&mut engine);
        Self { engine }
    }

    pub fn compile_str(&mut self, template_src: &'static str) -> Result<Executable, SteelErr> {
        let program = self.engine.emit_raw_program_no_path(template_src)?;
        let executable = self.engine.raw_program_to_executable(program)?;
        Ok(executable)
    }
    pub fn compile(&mut self, template_src: String) -> Result<Executable, SteelErr> {
        let program = self.engine.emit_raw_program_no_path(template_src)?;
        let executable = self.engine.raw_program_to_executable(program)?;
        Ok(executable)
    }

    pub fn render_template(
        &mut self,
        template: &Executable,
        payload_json: &str,
        content: String,
        entities: Vec<serde_json::Value>,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let mut payload_val: serde_json::Value = serde_json::from_str(payload_json)?;

        if let Some(obj) = payload_val.as_object_mut() {
            let username = obj.get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let initials = username
                .split_whitespace()
                .take(2)
                .filter_map(|s| s.chars().next())
                .collect::<String>()
                .to_uppercase();
            obj.insert("avatar_initials".to_string(),
                       serde_json::Value::String(initials));
            
            let grad_id = obj.get("grad_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let colors = Self::grad_colors(grad_id);
            obj.insert("avatar_color_top".to_string(),
                       serde_json::Value::String(colors.0.to_string()));
            obj.insert("avatar_color_bottom".to_string(),
                       serde_json::Value::String(colors.1.to_string()));
        }

        let ctx = TextContext { content, entities, };
        self.engine.update_value("payload", json_to_steelval(&payload_val));
        self.engine.update_value("text-context", ctx.into_steelval()?);

        let res = self.engine.run_executable(template)?;
        let last_val = res
            .last()
            .ok_or("Template returned no value")?;

        extract_node(last_val)
    }

    fn compile_and_render_template(&mut self,
                                   template: &'static str,
                                   payload_json: &str,
                                   content: String,
                                   entities: Vec<serde_json::Value>,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let executable = self.compile_str(template)?;
        self.render_template(&executable, payload_json, content, entities)

    }

    fn grad_colors(grad_id: u64) -> (&'static str, &'static str) {
        match grad_id % 7 {
            0 => ("#FF516A", "#FF885E"), // Red
            1 => ("#FFA85C", "#FFCD6A"), // Orange
            2 => ("#8C79F2", "#B37DF2"), // Purple
            3 => ("#51BB3F", "#8AE451"), // Green
            4 => ("#34C6CD", "#4CE9C2"), // Cyan
            5 => ("#549CFF", "#3CB9FE"), // Blue
            _ => ("#F2799B", "#F27DF2"), // Pink
        }
    }
}

fn extract_node(val: &SteelVal) -> Result<Node, Box<dyn std::error::Error>> {
    let scheme_node = SchemeNode::from_steelval(val)
        .map_err(|e| format!("Template must return a (node ...), got: {:?} ({})", val, e))?;
    Ok(scheme_node.0)
}

pub fn json_to_steelval(val: &serde_json::Value) -> SteelVal {
    match val {
        serde_json::Value::Null => SteelVal::ListV(Default::default()),
        serde_json::Value::Bool(b) => SteelVal::BoolV(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                SteelVal::IntV(i as isize)
            } else {
                SteelVal::NumV(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => SteelVal::StringV(SteelString::from(s.as_str())),
        serde_json::Value::Array(arr) => {
            SteelVal::ListV(arr.iter().map(json_to_steelval).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<SteelVal, SteelVal> = obj
                .iter()
                .map(|(k, v)| (
                    SteelVal::SymbolV(SteelString::from(k.as_str())),
                    json_to_steelval(v),
                ))
                .collect();
            
            SteelVal::HashMapV(Gc::new(map).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::node::Content;
    use serde_json::json;

    fn epsilon() -> f32 {
        0.0001
    }

    #[test]
    fn render_minimal_node() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            "(node (style (width (px 100)) (height (px 50))))",
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        assert!(matches!(node.content, Content::Group(ref c) if c.is_empty()));
        assert_eq!(node.style.layout.size.width, taffy::prelude::Dimension::length(100.0));
        assert_eq!(node.style.layout.size.height, taffy::prelude::Dimension::length(50.0));
    }

    #[test]
    fn render_shape_node() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            "(node (shape (rect) (fill (solid (hex \"#FF0000\")))))",
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        assert!(matches!(node.content, Content::Shape { ref fill, .. } if fill.is_some()));
    }

    #[test]
    fn render_text_from_payload() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'username "Default")))"#,
            r#"{"username": "Alice"}"#,
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "Alice"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_text_payload_default() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'missing_key "fallback")))"#,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "fallback"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_avatar_initials() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'avatar_initials "")))"#,
            r#"{"username": "John Doe"}"#,
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "JD"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_avatar_initials_single_name() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'avatar_initials "")))"#,
            r#"{"username": "alice"}"#,
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "A"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_avatar_initials_three_names() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'avatar_initials "")))"#,
            r#"{"username": "Foo Bar Baz"}"#,
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "FB"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_avatar_colors_by_grad_id() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'avatar_color_top "")))"#,
            r#"{"username": "X", "grad_id": 3}"#,
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "#51BB3F"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_bold() {
        let mut t = Templater::new();
        let payload = r#"{"content": "Hello bold world"}"#;
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            payload,
            "Hello bold world".to_string(),
            vec![json!({"type": "bold", "offset": 6, "length": 4})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.text, "Hello bold world");
                assert!(!rich.spans.is_empty(), "Should have entity spans");
                assert_eq!(rich.spans[0].weight, Some(parley::FontWeight::BOLD));
                assert_eq!(rich.spans[0].range, 6..10);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_italic() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "hi there"}"#,
            "hi there".to_string(),
            vec![json!({"type": "italic", "offset": 0, "length": 2})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].italic, Some(true));
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_link() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "click here"}"#,
            "click here".to_string(),
            vec![json!({"type": "text_link", "offset": 6, "length": 4})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].underline, Some(true));
                assert!(rich.spans[0].color.is_some());
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_nested_nodes() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (style (flex-column))
                 (node (style (width (px 100)) (height (px 30))))
                 (node (style (width (px 100)) (height (px 30)))))"#,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Group(children) => assert_eq!(children.len(), 2),
            other => panic!("Expected Group with 2 children, got {:?}", other),
        }
    }

    #[test]
    fn render_with_style_modifiers() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (style (flex-row) (align-center) (justify-between)
                         (padding (px 10)) (gap (px 5))
                         (opacity 0.8) (rotate 45)))"#,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        assert!((node.style.opacity - 0.8).abs() < epsilon());
        assert!((node.style.rotate_deg - 45.0).abs() < epsilon() as f64);
    }

    #[test]
    fn render_invalid_script_returns_error() {
        let mut t = Templater::new();
        let result = t.compile_and_render_template(
            "(this is not valid scheme +++",
            "{}",
            "".to_string(),
            vec![],
        );
        assert!(result.is_err());
    }

    #[test]
    fn render_with_conditional() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(if #t
                 (node (style (width (px 100)) (height (px 100))))
                 (node (style (width (px 50)) (height (px 50)))))"#,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        assert_eq!(node.style.layout.size.width, taffy::prelude::Dimension::length(100.0));
        assert_eq!(node.style.layout.size.height, taffy::prelude::Dimension::length(100.0));
    }

    #[test]
    fn render_with_gradients() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r##"(node (shape (rect) (fill (linear-gradient (hex "#FF0000") (hex "#0000FF")))))"##,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Shape { fill: Some(crate::primitives::paint::Paint::LinearGradient { .. }), .. } => {}
            other => panic!("Expected Shape with LinearGradient fill, got {:?}", other),
        }
    }

    #[test]
    fn render_rounded_rect() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r##"(node (shape (rounded-rect (px 10)) (fill (solid (hex "#000000")))))"##,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        assert!(matches!(node.content, Content::Shape { .. }));
    }

    #[test]
    fn render_circle_shape() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r##"(node (shape (circle (px 50)) (fill (solid (hex "#FF0000")))))"##,
            "{}",
            "".to_string(),
            vec![],
        ).unwrap();
        assert!(matches!(node.content, Content::Shape { .. }));
    }

    #[test]
    fn render_avatar_colors_all_grad_ids() {
        let expected_tops = [
            (0, "#FF516A"),
            (1, "#FFA85C"),
            (2, "#8C79F2"),
            (3, "#51BB3F"),
            (4, "#34C6CD"),
            (5, "#549CFF"),
            (6, "#F2799B"),
        ];
        for (grad_id, expected_top) in expected_tops {
            let mut t = Templater::new();
            let node = t.compile_and_render_template(
                r#"(node (text (get-payload 'avatar_color_top "")))"#,
                &format!(r#"{{"username": "X", "grad_id": {}}}"#, grad_id),
                "".to_string(),
                vec![],
            ).unwrap();
            match &node.content {
                Content::Text(rich) => assert_eq!(rich.text, expected_top, "grad_id={}", grad_id),
                other => panic!("Expected Text for grad_id={}, got {:?}", grad_id, other),
            }
        }
    }

    #[test]
    fn render_avatar_colors_bottom() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'avatar_color_bottom "")))"#,
            r#"{"username": "X", "grad_id": 5}"#,
            "".to_string(),
            vec![],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "#3CB9FE"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_underline() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "hello"}"#,
            "hello".to_string(),
            vec![json!({"type": "underline", "offset": 0, "length": 5})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].underline, Some(true));
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_strikethrough() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "hello"}"#,
            "hello".to_string(),
            vec![json!({"type": "strikethrough", "offset": 0, "length": 5})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].strikethrough, Some(true));
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_code() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "hello"}"#,
            "hello".to_string(),
            vec![json!({"type": "code", "offset": 0, "length": 5})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].font_family.is_some());
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_bot_command() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "/start"}"#,
            "/start".to_string(),
            vec![json!({"type": "bot_command", "offset": 0, "length": 6})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].color.is_some());
                assert!(rich.spans[0].underline.is_none());
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_url() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "http://a.com"}"#,
            "http://a.com".to_string(),
            vec![json!({"type": "url", "offset": 0, "length": 12})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].color.is_some());
                assert_eq!(rich.spans[0].underline, Some(true));
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_mention() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            r#"{"content": "@user"}"#,
            "@user".to_string(),
            vec![json!({"type": "mention", "offset": 0, "length": 5})],
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].color.is_some());
                assert_eq!(rich.spans[0].underline, Some(true));
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }
}

