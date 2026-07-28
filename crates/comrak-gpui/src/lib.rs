use std::cell::RefCell;

use comrak::arena_tree::Node;
use comrak::nodes::Ast;
use comrak::nodes::NodeValue::{Document, Paragraph, Text};
use comrak::{Arena, Options, parse_document};
use gpui::{AnyElement, IntoElement, ParentElement, SharedString, div};

// parses a markdown document using comrak and returns a
// renderable gpui element
pub fn render_document(document: &str) -> AnyElement {
    let arena = Arena::new();
    let root = parse_document(&arena, document, &Options::default());
    render_node(root)
}

// recursively traverses the AST tree by using the children() method and maps
// each child into a gpui element
fn render_node<'a>(node: &'a Node<'a, RefCell<Ast>>) -> AnyElement {
    match &node.data().value {
        Document => div()
            .children(node.children().map(render_node).collect::<Vec<_>>())
            .into_any_element(),
        Text(cow) => div()
            .child(SharedString::from(cow.as_ref().to_string()))
            .into_any_element(),
        Paragraph => div()
            .children(node.children().map(render_node).collect::<Vec<_>>())
            .into_any_element(),
        // TODO: handle other nodes.
        _ => div().into_any_element(),
    }
}
