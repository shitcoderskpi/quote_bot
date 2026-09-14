mod types;
mod functions;

use crate::primitives::node::Node;
use crate::proto::quote::SerializableMessage;
use steel::HashMap;
use steel::SteelErr;
use steel::compiler::program::Executable;
use steel::gc::Gc;
use steel::rvals::IntoSteelVal;
use steel::rvals::{FromSteelVal, SteelVal};
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
        msg: &SerializableMessage,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let mut map: HashMap<SteelVal, SteelVal> = HashMap::new();
        
        map.insert(SteelVal::SymbolV("username".into()), SteelVal::StringV(msg.username.as_str().into()));
        if let Some(ref status) = msg.user_status {
            map.insert(SteelVal::SymbolV("user_status".into()), SteelVal::StringV(status.as_str().into()));
        }
        if let Some(ref role) = msg.user_role {
            map.insert(SteelVal::SymbolV("user_role".into()), SteelVal::StringV(role.as_str().into()));
        }
        map.insert(SteelVal::SymbolV("content".into()), SteelVal::StringV(msg.content.as_str().into()));

        use base64::prelude::*;
        let b64_img = BASE64_STANDARD.encode(&msg.image);
        map.insert(SteelVal::SymbolV("image".into()), SteelVal::StringV(b64_img.into()));
        
        let initials = msg.username.split_whitespace().take(2).filter_map(|s| s.chars().next()).collect::<String>().to_uppercase();
        map.insert(SteelVal::SymbolV("avatar_initials".into()), SteelVal::StringV(initials.into()));
        
        let colors = Self::grad_colors(msg.grad_id as u64);
        map.insert(SteelVal::SymbolV("avatar_color_top".into()), SteelVal::StringV(colors.0.into()));
        map.insert(SteelVal::SymbolV("avatar_color_bottom".into()), SteelVal::StringV(colors.1.into()));
        
        let payload_val = SteelVal::HashMapV(Gc::new(map).into());

        let ctx = TextContext { content: msg.content.clone(), entities: msg.entities.clone(), };
        self.engine.update_value("payload", payload_val);
        self.engine.update_value("text-context", ctx.into_steelval()?);

        let res = self.engine.run_executable(template)?;
        let last_val = res
            .last()
            .ok_or("Template returned no value")?;

        extract_node(last_val)
    }

    #[cfg(test)]
    fn compile_and_render_template(
        &mut self,
        template: &'static str,
        msg: &SerializableMessage,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let executable = self.compile_str(template)?;
        self.render_template(&executable, msg)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::node::Content;
    use crate::quote::{Entity, SerializableMessage};

    fn epsilon() -> f32 {
        0.0001
    }

    #[test]
    fn render_minimal_node() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            "(node (style (width (px 100)) (height (px 50))))",
            &SerializableMessage::default(),
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
            &SerializableMessage::default(),
        ).unwrap();
        assert!(matches!(node.content, Content::Shape { ref fill, .. } if fill.is_some()));
    }

    #[test]
    fn render_text_from_payload() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'username "Default")))"#,
            &SerializableMessage {
                username: "Alice".to_string(),
                ..Default::default()
            },
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
            &SerializableMessage::default(),
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
            &SerializableMessage {
                username: "John Doe".to_string(),
                ..Default::default()
            },
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
            &SerializableMessage {
                username: "Alice".to_string(),
                ..Default::default()
            },
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
            &SerializableMessage {
                username: "Foo Bar Baz".to_string(),
                ..Default::default()
            },
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
            &SerializableMessage {
                grad_id: 3,
                ..Default::default()
            },
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
            &SerializableMessage {
                content: "Hello bold world".to_string(),
                entities: vec![Entity { r#type: "bold".to_string(), offset: 6, length: 4 }],
                ..Default::default()
            },
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
            &SerializableMessage {
                content: "hi there".to_string(),
                entities: vec![Entity { r#type: "italic".to_string(), offset: 0, length: 2 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].italic, true);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_link() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            &SerializableMessage {
                content: "click here".to_string(),
                entities: vec![Entity { r#type: "text_link".to_string(), offset: 6, length: 4 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].underline, true);
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
            &SerializableMessage::default(),
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
            &SerializableMessage::default(),
        ).unwrap();
        assert!((node.style.opacity - 0.8).abs() < epsilon());
        assert!((node.style.rotate_deg - 45.0).abs() < epsilon() as f64);
    }

    #[test]
    fn render_invalid_script_returns_error() {
        let mut t = Templater::new();
        let result = t.compile_and_render_template(
            "(this is not valid scheme +++",
            &SerializableMessage::default(),
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
            &SerializableMessage::default(),
        ).unwrap();
        assert_eq!(node.style.layout.size.width, taffy::prelude::Dimension::length(100.0));
        assert_eq!(node.style.layout.size.height, taffy::prelude::Dimension::length(100.0));
    }

    #[test]
    fn render_with_gradients() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r##"(node (shape (rect) (fill (linear-gradient (hex "#FF0000") (hex "#0000FF")))))"##,
            &SerializableMessage::default(),
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
            &SerializableMessage::default(),
        ).unwrap();
        assert!(matches!(node.content, Content::Shape { .. }));
    }

    #[test]
    fn render_circle_shape() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r##"(node (shape (circle (px 50)) (fill (solid (hex "#FF0000")))))"##,
            &SerializableMessage::default(),
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
            &SerializableMessage {
                grad_id: grad_id as i32,
                ..Default::default()
            },
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
            &SerializableMessage {
                grad_id: 5,
                ..Default::default()
            },
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
            &SerializableMessage {
                content: "hello".to_string(),
                entities: vec![Entity { r#type: "underline".to_string(), offset: 0, length: 5 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].underline, true);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_strikethrough() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            &SerializableMessage {
                content: "hello".to_string(),
                entities: vec![Entity { r#type: "strikethrough".to_string(), offset: 0, length: 5 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert_eq!(rich.spans[0].strikethrough, true);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_code() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            &SerializableMessage {
                content: "hello".to_string(),
                entities: vec![Entity { r#type: "code".to_string(), offset: 0, length: 5 }],
                ..Default::default()
            },
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
            &SerializableMessage {
                content: "/start".to_string(),
                entities: vec![Entity { r#type: "bot_command".to_string(), offset: 0, length: 6 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].color.is_some());
                assert!(!rich.spans[0].underline);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_url() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            &SerializableMessage {
                content: "http://a.com".to_string(),
                entities: vec![Entity { r#type: "url".to_string(), offset: 0, length: 12 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].color.is_some());
                assert_eq!(rich.spans[0].underline, true);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn render_with_entities_mention() {
        let mut t = Templater::new();
        let node = t.compile_and_render_template(
            r#"(node (text (get-payload 'content "")))"#,
            &SerializableMessage {
                content: "@user".to_string(),
                entities: vec![Entity { r#type: "mention".to_string(), offset: 0, length: 5 }],
                ..Default::default()
            },
        ).unwrap();
        match &node.content {
            Content::Text(rich) => {
                assert!(rich.spans[0].color.is_some());
                assert_eq!(rich.spans[0].underline, true);
            }
            other => panic!("Expected Text, got {:?}", other),
        }
    }
}

