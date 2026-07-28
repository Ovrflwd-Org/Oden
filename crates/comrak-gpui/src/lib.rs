use std::cell::RefCell;

use comrak::arena_tree::Node;
use comrak::nodes::Ast;
use comrak::nodes::NodeValue::{Document, Heading, Paragraph, Strong, Text};
use comrak::{Arena, Options, parse_document};
use gpui::{
    AnyElement, App, Font, FontFeatures, FontStyle, FontWeight, IntoElement, ParentElement, Styled,
    StyledText, TextRun, div,
};
use gpui_component::{ActiveTheme, Theme};
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
            .flex_1()
            .min_w_0()
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
            collect_segments(node, &mut text, FontWeight::NORMAL, &mut runs, theme);
            StyledText::new(text).with_runs(runs).into_any_element()
        },
        Text(cow) => {
            let text = cow.as_ref().to_string();
            div().child(text).into_any_element()
        },
        _ => div().into_any_element(),
    }
}

// collect text segments, register text runs, and return one StyledText block.
fn collect_segments<'a>(
    node: &'a Node<'a, RefCell<Ast>>,
    text: &mut String,
    font_weight: FontWeight,
    runs: &mut Vec<TextRun>,
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
                        weight: font_weight,
                        style: FontStyle::Normal,
                    },
                    color: theme.foreground,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                });
            }
            Strong => {
                collect_segments(child, text, FontWeight::EXTRA_BOLD, runs, theme);
            }
            _ => {}
        }
    }
}
