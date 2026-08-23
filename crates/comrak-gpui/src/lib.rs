use std::cell::RefCell;
use std::ops::Range;

use comrak::arena_tree::Node;
use comrak::nodes::NodeValue::{
    self, BlockQuote, Code, CodeBlock, Document, Emph, Heading, Item, LineBreak, Link, List,
    Paragraph, Strong, TaskItem, Text,
};
use comrak::nodes::{Ast, ListType};
use comrak::{Arena, Options, parse_document};
use gpui::{
    AnyElement, App, ElementId, Font, FontFeatures, FontStyle, FontWeight, InteractiveText,
    IntoElement, ParentElement, SharedString, Styled, StyledText, TextRun, div, px, rems,
};
use gpui_component::checkbox::Checkbox;
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

#[derive(Default, Debug, Copy, Clone)]
// a struct to store any context that needs to be
// passed between parent and child nodes.
// Now only used for BlockQuote, to change the text color
// of a paragraph inside a blockquote.
struct RenderingContext {
    in_blockquote: bool,
}

/// parses a markdown document using comrak and returns a
/// renderable gpui element
pub fn render_document(document: &str, cx: &App) -> AnyElement {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.tasklist = true;
    let root = parse_document(&arena, document, &options);
    render_node(root, cx, RenderingContext::default())
}

fn render_block_for_children<'a>(
    node: &'a Node<'a, RefCell<Ast>>,
    cx: &App,
    render_context: RenderingContext,
) -> Vec<AnyElement> {
    let children = node.children().collect::<Vec<_>>();
    node.children()
        .enumerate()
        .map(|(idx, child)| {
            let is_last = idx == children.len() - 1;
            let gap = if is_last { 0. } else { block_for_node(child) };
            div()
                .pb(rems(gap))
                .child(render_node(child, cx, render_context))
                .into_any_element()
        })
        .collect()
}

// applies semantic gaps between children based on their node type
fn block_for_node(node: &Node<'_, RefCell<Ast>>) -> f32 {
    match &node.data().value {
        NodeValue::Paragraph => 0.8, // rem
        NodeValue::Heading(_) => 0.4,
        NodeValue::CodeBlock(_) => 1.0,
        NodeValue::List(_) => 1.0,
        NodeValue::BlockQuote => 1.0,
        _ => 0.6,
    }
}

// recursively traverses the AST tree by using the children() method and maps
// each child into a gpui element
fn render_node<'a>(
    node: &'a Node<'a, RefCell<Ast>>,
    cx: &App,
    render_cx: RenderingContext,
) -> AnyElement {
    let theme = cx.theme();
    match &node.data().value {
        Document => div()
            .p_4()
            .gap_12()
            .size_full()
            .children(render_block_for_children(node, cx, render_cx))
            .into_any_element(),
        Heading(node_heading) => match node_heading.level {
            1 => div()
                .text_2xl()
                .text_color(cx.theme().primary)
                .font_weight(FontWeight::BOLD)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx, render_cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
            2 => div()
                .text_xl()
                .text_color(cx.theme().secondary_foreground)
                .font_weight(FontWeight::BOLD)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx, render_cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
            3 => div()
                .text_lg()
                .text_color(cx.theme().accent_foreground)
                .font_weight(FontWeight::BOLD)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx, render_cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
            _ => div()
                .children(
                    node.children()
                        .map(|node| render_node(node, cx, render_cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element(),
        },
        Paragraph => {
            let mut text = String::new();
            let mut runs = vec![];
            let mut links = vec![];
            let text_style = if render_cx.in_blockquote {
                TextStyle {
                    color: Some(theme.muted_foreground),
                    ..Default::default()
                }
            } else {
                TextStyle::default()
            };
            collect_segments(node, &mut text, &text_style, &mut runs, &mut links, theme);
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
        BlockQuote => {
            let block_quote_render_cx = RenderingContext {
                in_blockquote: true,
            };
            div()
                .flex()
                .flex_col()
                .p_2()
                .border_l(px(4.))
                .border_color(theme.border)
                .bg(theme.muted)
                .children(
                    node.children()
                        .map(|node| render_node(node, cx, block_quote_render_cx))
                        .collect::<Vec<_>>(),
                )
                .into_any_element()
        }
        List(_) => div()
            .flex()
            .flex_col()
            .children(
                node.children()
                    .map(|node| render_node(node, cx, render_cx))
                    .collect::<Vec<_>>(),
            )
            .into_any_element(),
        Item(metadata) => {
            let symbol = match metadata.list_type {
                ListType::Bullet => SharedString::from("•"),
                ListType::Ordered => SharedString::from(format!("{}.", metadata.start)),
            };
            div()
                .flex()
                .flex_row()
                .gap_2()
                .pl(px(metadata.padding as f32))
                .child(symbol)
                .child(
                    // text wrappers should have flex_1, so text wraps to available width.
                    div().flex_1().w_full().min_w_0().children(
                        node.children()
                            .map(|node| render_node(node, cx, render_cx))
                            .collect::<Vec<_>>(),
                    ),
                )
                .into_any_element()
        }
        TaskItem(metadata) => div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(Checkbox::new("list-item-checkbox").checked(metadata.symbol.is_some()))
            .children(
                node.children()
                    .map(|node| render_node(node, cx, render_cx))
                    .collect::<Vec<_>>(),
            )
            .into_any_element(),
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
                child_style.color = Some(theme.red);
                child_style.weight = FontWeight::BOLD;
                collect_segments(child, text, &child_style, runs, links, theme);
            }
            Emph => {
                let mut child_style = style.clone();
                child_style.color = Some(theme.green);
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
                    color: theme.primary,
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
