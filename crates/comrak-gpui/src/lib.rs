use std::cell::RefCell;
use std::ops::Range;

use comrak::arena_tree::Node;
use comrak::nodes::Ast;
use comrak::nodes::NodeValue::{
    Code, CodeBlock, Document, Emph, Heading, LineBreak, Link, Paragraph, Strong, Text,
};
use comrak::{Arena, Options, parse_document};
use gpui::{
    AnyElement, App, ElementId, Font, FontFeatures, FontStyle, FontWeight, InteractiveText,
    IntoElement, ParentElement, SharedString, Styled, StyledText, TextRun, div,
};
use gpui_component::{ActiveTheme, Theme};

#[derive(Clone, Default)]
struct TextStyle {
    weight: FontWeight,
    style: FontStyle,
    color: Option<gpui::Hsla>,
    background_color: Option<gpui::Hsla>,
    underline: Option<gpui::UnderlineStyle>,
    strikethrough: Option<gpui::StrikethroughStyle>,
}
// parses a markdown document using comrak and returns a
// renderable gpui element
pub fn render_document(document: &str, cx: &App) -> AnyElement {
    let arena = Arena::new();
    let root = parse_document(&arena, document, &Options::default());
    render_node(root, cx)
}

// recursively traverses the AST tree by using the children() method and maps
// each child into a gpui element
fn render_node<'a>(node: &'a Node<'a, RefCell<Ast>>, cx: &App) -> AnyElement {
    let theme = cx.theme();
    match &node.data().value {
        Document => div()
            .p_4()
            .size_full()
            .children(
                node.children()
                    .map(|node| render_node(node, cx))
                    .collect::<Vec<_>>(),
            )
            .into_any_element(),
        Heading(node_heading) => match node_heading.level {
            1 => div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
            2 => div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
            3 => div()
                .text_lg()
                .font_weight(FontWeight::BOLD)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
            _ => div()
                .children(
                    node.children()
                        .map(|node| render_node(node, cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
        },
        Paragraph => {
            let mut text = String::new();
            let mut runs = vec![];
            let mut links = vec![];
            collect_segments(
                node,
                &mut text,
                &TextStyle::default(),
                &mut runs,
                &mut links,
                theme,
            );
            let ranges: Vec<Range<usize>> = links.iter().map(|link| link.range.clone()).collect();
            InteractiveText::new(
                ElementId::Name(SharedString::from("md-paragraph")),
                StyledText::new(text).with_runs(runs),
            )
            .on_click(ranges, move |index, _window, cx| {
                let link = &links[index];
                cx.open_url(&link.href);
            })
            .into_any_element()
        }
        Text(cow) => {
            let text = cow.as_ref().to_string();
            div().child(text).into_any_element()
        }
        CodeBlock(code_block) => {
            let literal = &code_block.literal;
            let mut literal = literal.clone();
            let _ = literal.pop();
            div()
                .border_r_2()
                .border_color(theme.border)
                .bg(theme.secondary)
                .p_2()
                .mt_2()
                .mb_2()
                .flex()
                .flex_col()
                .justify_center()
                .child(
                    div()
                        // TODO: syntax highlighting for code blocks.
                        .child(SharedString::from(literal.to_string()))
                        .font_family(theme.mono_font_family.clone())
                        .text_sm()
                        .text_color(theme.secondary_foreground),
                )
                .into_any_element()
        }
        _ => div().into_any_element(),
    }
}

// collect text segments, register text runs, and return one StyledText block.
fn collect_segments<'a>(
    node: &'a Node<'a, RefCell<Ast>>,
    text: &mut String,
    style: &TextStyle,
    runs: &mut Vec<TextRun>,
    links: &mut Vec<LinkSpan>,
    theme: &Theme,
) {
    for child in node.children() {
        match &child.data().value {
            Text(cow) => {
                let s = cow.as_ref();
                text.push_str(s);
                runs.push(TextRun {
                    len: s.len(),
                    font: Font {
                        family: theme.font_family.clone(),
                        features: FontFeatures::default(),
                        fallbacks: None,
                        weight: style.weight,
                        style: style.style,
                    },
                    color: style.color.unwrap_or(theme.foreground),
                    background_color: style.background_color,
                    underline: style.underline,
                    strikethrough: style.strikethrough,
                });
            }
            LineBreak => {
                text.push('\n');
                runs.push(TextRun {
                    len: 1,
                    font: Font {
                        family: theme.font_family.clone(),
                        features: FontFeatures::default(),
                        fallbacks: None,
                        weight: style.weight,
                        style: style.style,
                    },
                    color: style.color.unwrap_or(theme.foreground),
                    background_color: style.background_color,
                    underline: style.underline,
                    strikethrough: style.strikethrough,
                });
            }
            Strong => {
                let mut child_style = style.clone();
                child_style.weight = FontWeight::BOLD;
                collect_segments(child, text, &child_style, runs, links, theme);
            }
            Emph => {
                let mut child_style = style.clone();
                child_style.style = FontStyle::Italic;
                collect_segments(child, text, &child_style, runs, links, theme);
            }
            Link(node_link) => {
                let mut child_style = style.clone();
                child_style.color = Some(theme.link);
                let start = text.len();
                collect_segments(child, text, &child_style, runs, links, theme);
                let end = text.len();
                if end > start {
                    links.push(LinkSpan {
                        range: Range { start, end },
                        href: node_link.url.clone(),
                    });
                }
            }
            Code(node_code) => {
                let s = &node_code.literal;
                text.push_str(s);
                runs.push(TextRun {
                    len: s.len(),
                    font: Font {
                        family: theme.mono_font_family.clone(),
                        features: FontFeatures::default(),
                        fallbacks: None,
                        weight: style.weight,
                        style: style.style,
                    },
                    color: theme.secondary_foreground,
                    background_color: Some(theme.secondary),
                    underline: style.underline,
                    strikethrough: style.strikethrough,
                });
            }
            _ => {}
        }
    }
}

struct LinkSpan {
    range: Range<usize>,
    href: String,
}
