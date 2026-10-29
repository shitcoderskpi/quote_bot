mod types;
mod functions;
pub mod image;

use crate::primitives::node::Node;
use crate::proto::quote::{Author, QuoteMessage, QuoteRequest};
use crate::templater::image::SteelImage;
use steel::HashMap;
use steel::SteelErr;
use steel::compiler::program::Executable;
use steel::gc::Gc;
use steel::rvals::{FromSteelVal, IntoSteelVal, SteelVal};
use steel::steel_vm::engine::Engine;
pub use types::SchemeNode;
use types::TextEntities;

pub struct Symbols {
    username: SteelVal,
    user_status: SteelVal,
    user_role: SteelVal,
    content: SteelVal,
    entities: SteelVal,
    image: SteelVal,
    avatar_initials: SteelVal,
    avatar_color_top: SteelVal,
    avatar_color_bottom: SteelVal,
    name_color: SteelVal,
    group_start: SteelVal,
    group_end: SteelVal,
    messages: SteelVal,
    theme: SteelVal,
}

impl Symbols {
    fn new() -> Self {
        let sym = |n: &str| SteelVal::SymbolV(n.into());
        Self {
            username: sym("username"),
            user_status: sym("user_status"),
            user_role: sym("user_role"),
            content: sym("content"),
            entities: sym("entities"),
            image: sym("image"),
            avatar_initials: sym("avatar_initials"),
            avatar_color_top: sym("avatar_color_top"),
            avatar_color_bottom: sym("avatar_color_bottom"),
            name_color: sym("name_color"),
            group_start: sym("group_start"),
            group_end: sym("group_end"),
            messages: sym("messages"),
            theme: sym("theme"),
        }
    }
}

pub struct Templater {
    engine: Engine,
    symbols: Symbols,
}

impl Templater {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        functions::register_all(&mut engine);
        Self { engine, symbols: Symbols::new() }
    }

    pub fn compile(&mut self, template_src: String) -> Result<Executable, SteelErr> {
        let program = self.engine.emit_raw_program_no_path(template_src)?;
        let executable = self.engine.raw_program_to_executable(program)?;
        Ok(executable)
    }

    fn message_to_steel(
        s: &Symbols,
        msg: QuoteMessage,
        group_start: bool,
        group_end: bool,
    ) -> Result<SteelVal, SteelErr> {
        let QuoteMessage { author, content, entities, image, .. } = msg;
        let Author { color_index, username, user_status, user_role, .. } =
            author.unwrap_or_else(|| Author {
                username: "Unknown".to_owned(),
                ..Default::default()
            });

        let initials: String = username
            .split_whitespace()
            .take(2)
            .filter_map(|w| w.chars().next())
            .flat_map(char::to_uppercase)
            .collect();
        let (top, bottom) = Self::avatar_colors(color_index);
        let peer_color = Self::peer_color(color_index);

        let mut m = HashMap::new();
        m.insert(s.username.clone(), SteelVal::StringV(username.into()));
        if let Some(v) = user_status {
            m.insert(s.user_status.clone(), SteelVal::StringV(v.into()));
        }
        if let Some(v) = user_role {
            m.insert(s.user_role.clone(), SteelVal::StringV(v.into()));
        }
        m.insert(s.content.clone(), SteelVal::StringV(content.into()));
        m.insert(
            s.entities.clone(),
            TextEntities(entities.into_boxed_slice()).into_steelval()?,
        );

        if !image.is_empty() {
            m.insert(s.image.clone(), SteelImage::new(image.into()).into_steelval());
        }
        m.insert(s.avatar_initials.clone(), SteelVal::StringV(initials.into()));
        m.insert(s.avatar_color_top.clone(), SteelVal::StringV(top.into()));
        m.insert(s.avatar_color_bottom.clone(), SteelVal::StringV(bottom.into()));
        m.insert(s.name_color.clone(), SteelVal::StringV(peer_color.into()));
        m.insert(s.group_start.clone(), SteelVal::BoolV(group_start));
        m.insert(s.group_end.clone(), SteelVal::BoolV(group_end));

        Ok(SteelVal::HashMapV(Gc::new(m).into()))
    }

    pub fn update_payload(&mut self, req: QuoteRequest) -> Result<(), SteelErr> {
        let QuoteRequest { theme, messages, .. } = req;

        // TODO: BAD
        let keys: Vec<Option<u32>> = messages
            .iter()
            .map(|m| m.author.as_ref().map(|a| a.sender_index))
            .collect();
        let n = messages.len();
        let s = &self.symbols;

        let list = messages
            .into_iter()
            .enumerate()
            .map(|(i, m)| {
                let start = i == 0 || keys[i - 1] != keys[i];
                let end = i + 1 == n || keys[i + 1] != keys[i];
                Self::message_to_steel(s, m, start, end)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut payload = HashMap::new();
        // TODO: also BAD
        payload.insert(s.messages.clone(), SteelVal::ListV(list.into_iter().collect()));
        payload.insert(s.theme.clone(), SteelVal::StringV(theme.into()));

        self.engine
            .update_value("payload", SteelVal::HashMapV(Gc::new(payload).into()));
        Ok(())
    }

    pub fn render_template(
        &mut self,
        template: &Executable,
        req: QuoteRequest,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        self.update_payload(req)?;
        let res = self.engine.run_executable(template)?;
        let last_val = res.last().ok_or("Template returned no value")?;
        extract_node(last_val)
    }

    #[cfg(test)]
    fn compile_and_render_template(
        &mut self,
        template: &str,
        messages: Vec<QuoteMessage>,
    ) -> Result<Node, Box<dyn std::error::Error>> {
        let executable = self.compile(template.to_owned())?;
        self.render_template(&executable, QuoteRequest { messages, ..Default::default() })
    }

    fn avatar_colors(color_index: i32) -> (&'static str, &'static str) {
        match color_index.rem_euclid(7) {
            0 => ("#FF845E", "#D45246"), // Red
            1 => ("#FEBB5B", "#F68136"), // Orange
            2 => ("#B694F9", "#6C61DF"), // Violet
            3 => ("#9AD164", "#46BA43"), // Green
            4 => ("#5BCBE3", "#359AD4"), // Cyan
            5 => ("#5CAFFA", "#408ACF"), // Blue
            _ => ("#FF8AAC", "#D95574"), // Pink
        }
    }

    fn peer_color(color_index: i32) -> &'static str {
        match color_index.rem_euclid(7) {
            0 => "#E17076", // Red
            1 => "#EFA65A", // Orange
            2 => "#A695E7", // Purple
            3 => "#7BC862", // Green
            4 => "#6EC9CB", // Cyan
            5 => "#65AADD", // Blue
            _ => "#EE7AAE", // Pink
        }
    }
}

fn extract_node(val: &SteelVal) -> Result<Node, Box<dyn std::error::Error>> {
    let scheme_node = SchemeNode::from_steelval(val)
        .map_err(|e| format!("Template must return a (node ...), got: {:?} ({})", val, e))?;
    Ok(scheme_node.0.borrow_mut().take().ok_or("Reusing root node is forbidden!")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::node::Content;
    use crate::primitives::paint::Paint;
    use crate::proto::quote::Entity;
    use taffy::prelude::Dimension;

    const EPSILON: f32 = 0.0001;
    const FIRST: &str = "(define m (car (hash-ref payload 'messages)))\n";

    fn entity(kind: &str, offset: i32, length: i32) -> Entity {
        Entity { r#type: kind.to_owned(), offset, length }
    }

    fn msg_with_author(author: Author) -> QuoteMessage {
        QuoteMessage { author: Some(author), ..Default::default() }
    }

    fn msg_with_user(username: &str) -> QuoteMessage {
        msg_with_author(Author { username: username.to_owned(), ..Default::default() })
    }

    fn msg_with_color(color_index: i32) -> QuoteMessage {
        msg_with_author(Author { color_index, ..Default::default() })
    }

    fn msg_from_sender(sender_index: u32) -> QuoteMessage {
        msg_with_author(Author { sender_index, ..Default::default() })
    }

    fn msg_with_text(content: &str, entities: Vec<Entity>) -> QuoteMessage {
        QuoteMessage { content: content.to_owned(), entities, ..Default::default() }
    }

    macro_rules! text_of {
        ($node:expr) => {
            match &$node.content {
                Content::Text(rich) => rich,
                other => panic!("Expected Text, got {:?}", other),
            }
        };
    }

    fn render_many(template: &str, msgs: Vec<QuoteMessage>) -> Node {
        Templater::new().compile_and_render_template(template, msgs).unwrap()
    }

    fn render(template: &str, msg: QuoteMessage) -> Node {
        render_many(&format!("{FIRST}{template}"), vec![msg])
    }

    fn payload_text(key: &str, msg: QuoteMessage) -> String {
        let node = render(&format!(r#"(node (text (get-payload m '{key} "")))"#), msg);
        text_of!(node).text.clone()
    }

    fn render_entity(content: &str, kind: &str, offset: i32, length: i32) -> Node {
        render(
            r#"(node (text (get-payload m 'content "") #:entities (get-payload m 'entities #f)))"#,
            msg_with_text(content, vec![entity(kind, offset, length)]),
        )
    }

    #[test]
    fn render_minimal_node() {
        let node = render(
            "(node (style (width (px 100)) (height (px 50))))",
            QuoteMessage::default(),
        );
        assert!(matches!(node.content, Content::Group(ref c) if c.is_empty()));
        assert_eq!(node.style.layout.size.width, Dimension::length(100.0));
        assert_eq!(node.style.layout.size.height, Dimension::length(50.0));
    }

    #[test]
    fn render_nested_nodes() {
        let node = render(
            r#"(node (style (flex-column))
                 (node (style (width (px 100)) (height (px 30))))
                 (node (style (width (px 100)) (height (px 30)))))"#,
            QuoteMessage::default(),
        );
        match &node.content {
            Content::Group(children) => assert_eq!(children.len(), 2),
            other => panic!("Expected Group with 2 children, got {:?}", other),
        }
    }

    #[test]
    fn render_with_style_modifiers() {
        let node = render(
            r#"(node (style (flex-row) (align-center) (justify-between)
                         (padding (px 10)) (gap (px 5))
                         (opacity 0.8) (rotate 45)))"#,
            QuoteMessage::default(),
        );
        assert!((node.style.opacity - 0.8).abs() < EPSILON);
        assert!((node.style.rotate_deg - 45.0).abs() < EPSILON as f64);
    }

    #[test]
    fn render_with_conditional() {
        let node = render(
            r#"(if #t
                 (node (style (width (px 100)) (height (px 100))))
                 (node (style (width (px 50)) (height (px 50)))))"#,
            QuoteMessage::default(),
        );
        assert_eq!(node.style.layout.size.width, Dimension::length(100.0));
        assert_eq!(node.style.layout.size.height, Dimension::length(100.0));
    }

    #[test]
    fn render_invalid_script_returns_error() {
        let result = Templater::new()
            .compile_and_render_template("(this is not valid scheme +++", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn render_shape_node() {
        let node = render(
            r##"(node (shape (rect) (fill (solid (hex "#FF0000")))))"##,
            QuoteMessage::default(),
        );
        assert!(matches!(node.content, Content::Shape { ref fill, .. } if fill.is_some()));
    }

    #[test]
    fn render_rounded_rect() {
        let node = render(
            r##"(node (shape (rounded-rect (px 10)) (fill (solid (hex "#000000")))))"##,
            QuoteMessage::default(),
        );
        assert!(matches!(node.content, Content::Shape { .. }));
    }

    #[test]
    fn render_circle_shape() {
        let node = render(
            r##"(node (shape (circle (px 50)) (fill (solid (hex "#FF0000")))))"##,
            QuoteMessage::default(),
        );
        assert!(matches!(node.content, Content::Shape { .. }));
    }

    #[test]
    fn render_with_gradients() {
        let node = render(
            r##"(node (shape (rect) (fill (linear-gradient (hex "#FF0000") (hex "#0000FF")))))"##,
            QuoteMessage::default(),
        );
        assert!(matches!(
            node.content,
            Content::Shape { fill: Some(Paint::LinearGradient { .. }), .. }
        ));
    }

    #[test]
    fn render_text_from_payload() {
        assert_eq!(payload_text("username", msg_with_user("Alice")), "Alice");
    }

    #[test]
    fn render_text_payload_default() {
        let node = render(
            r#"(node (text (get-payload m 'missing_key "fallback")))"#,
            QuoteMessage::default(),
        );
        assert_eq!(text_of!(node).text, "fallback");
    }

    #[test]
    fn render_missing_author_uses_unknown() {
        assert_eq!(payload_text("username", QuoteMessage::default()), "Unknown");
    }

    #[test]
    fn render_optional_author_fields() {
        let msg = msg_with_author(Author {
            user_status: Some("online".to_owned()),
            user_role: Some("admin".to_owned()),
            ..Default::default()
        });
        let node = render(
            r#"(node (text (string-append (get-payload m 'user_status "") "/" (get-payload m 'user_role ""))))"#,
            msg,
        );
        assert_eq!(text_of!(node).text, "online/admin");
    }

    #[test]
    fn render_empty_image_is_absent() {
        let node = render(
            r#"(node (text (if (get-payload m 'image #f) "has" "none")))"#,
            QuoteMessage::default(),
        );
        assert_eq!(text_of!(node).text, "none");
    }

    #[test]
    fn render_avatar_initials() {
        for (username, expected) in [
            ("John Doe", "JD"),
            ("Alice", "A"),
            ("Foo Bar Baz", "FB"),
            ("alice smith", "AS"),
        ] {
            assert_eq!(
                payload_text("avatar_initials", msg_with_user(username)),
                expected,
                "username={username:?}",
            );
        }
    }

    #[test]
    fn render_avatar_colors_all_color_indices() {
        let expected = [
            (0, "#FF845E", "#D45246"),
            (1, "#FEBB5B", "#F68136"),
            (2, "#B694F9", "#6C61DF"),
            (3, "#9AD164", "#46BA43"),
            (4, "#5BCBE3", "#359AD4"),
            (5, "#5CAFFA", "#408ACF"),
            (6, "#FF8AAC", "#D95574"),
            (7, "#FF845E", "#D45246"),
            (-1, "#FF8AAC", "#D95574"),
        ];
        for (idx, top, bottom) in expected {
            assert_eq!(payload_text("avatar_color_top", msg_with_color(idx)), top, "top, idx={idx}");
            assert_eq!(payload_text("avatar_color_bottom", msg_with_color(idx)), bottom, "bottom, idx={idx}");
        }
    }

    #[test]
    fn render_multiple_messages_in_order() {
        let node = render_many(
            r#"(%make-node
                 (cons (style (flex-column))
                       (map (lambda (m) (node (text (get-payload m 'content ""))))
                            (hash-ref payload 'messages))))"#,
            vec![
                msg_with_text("one", vec![]),
                msg_with_text("two", vec![]),
                msg_with_text("three", vec![]),
            ],
        );
        let Content::Group(children) = &node.content else {
            panic!("Expected Group, got {:?}", node.content)
        };
        let texts: Vec<_> = children.iter().map(|c| text_of!(c).text.clone()).collect();
        assert_eq!(texts, ["one", "two", "three"]);
    }

    #[test]
    fn render_group_flags_by_sender_index() {
        let node = render_many(
            r#"(%make-node
                 (cons (style (flex-column))
                       (map (lambda (m)
                              (node (text (string-append
                                            (if (get-payload m 'group_start #f) "S" "-")
                                            (if (get-payload m 'group_end #f) "E" "-")))))
                            (hash-ref payload 'messages))))"#,
            vec![msg_from_sender(0), msg_from_sender(0), msg_from_sender(1)],
        );
        let Content::Group(children) = &node.content else {
            panic!("Expected Group, got {:?}", node.content)
        };
        let flags: Vec<_> = children.iter().map(|c| text_of!(c).text.clone()).collect();
        assert_eq!(flags, ["S-", "-E", "SE"]);
    }

    #[test]
    fn render_group_flags_single_message() {
        let node = render(
            r#"(node (text (string-append
                             (if (get-payload m 'group_start #f) "S" "-")
                             (if (get-payload m 'group_end #f) "E" "-"))))"#,
            QuoteMessage::default(),
        );
        assert_eq!(text_of!(node).text, "SE");
    }

    #[test]
    fn render_entities_stay_with_their_message() {
        let node = render_many(
            r#"(%make-node
                 (cons (style (flex-column))
                       (map (lambda (m)
                              (node (text (get-payload m 'content "")
                                          #:entities (get-payload m 'entities #f))))
                            (hash-ref payload 'messages))))"#,
            vec![
                msg_with_text("plain", vec![]),
                msg_with_text("bold", vec![entity("bold", 0, 4)]),
            ],
        );
        let Content::Group(children) = &node.content else {
            panic!("Expected Group, got {:?}", node.content)
        };
        assert!(text_of!(children[0]).spans.is_empty());
        assert_eq!(text_of!(children[1]).spans.len(), 1);
    }

    #[test]
    fn render_with_entities_bold() {
        let node = render_entity("Hello bold world", "bold", 6, 4);
        let rich = text_of!(node);
        assert_eq!(rich.text, "Hello bold world");
        assert_eq!(rich.spans.len(), 1);
        assert_eq!(rich.spans[0].weight, Some(parley::FontWeight::BOLD));
        assert_eq!(rich.spans[0].range, 6..10);
    }

    #[test]
    fn render_with_entities_italic() {
        let node = render_entity("hi there", "italic", 0, 2);
        assert!(text_of!(node).spans[0].italic);
    }

    #[test]
    fn render_with_entities_underline() {
        let node = render_entity("hello", "underline", 0, 5);
        assert!(text_of!(node).spans[0].underline);
    }

    #[test]
    fn render_with_entities_strikethrough() {
        let node = render_entity("hello", "strikethrough", 0, 5);
        assert!(text_of!(node).spans[0].strikethrough);
    }

    #[test]
    fn render_with_entities_code() {
        let node = render_entity("hello", "code", 0, 5);
        assert!(text_of!(node).spans[0].font_family.is_some());
    }

    #[test]
    fn render_with_entities_bot_command() {
        let node = render_entity("/start", "bot_command", 0, 6);
        let span = &text_of!(node).spans[0];
        assert!(span.paint.is_some());
        assert!(!span.underline);
    }

    #[test]
    fn render_with_entities_linkish() {
        for (kind, content, offset, length) in [
            ("text_link", "click here", 6, 4),
            ("url", "http://a.com", 0, 12),
            ("mention", "@user", 0, 5),
        ] {
            let node = render_entity(content, kind, offset, length);
            let span = &text_of!(node).spans[0];
            assert!(span.paint.is_some(), "{kind}: paint");
            assert!(span.underline, "{kind}: underline");
        }
    }
}