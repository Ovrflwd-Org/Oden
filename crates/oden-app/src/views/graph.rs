#[cfg(debug_assertions)]
use gpui::{ParentElement, Path, PathBuilder, Pixels, Point, Size, canvas};
#[cfg(debug_assertions)]
use gpui::{Render, Styled, div, px};
#[cfg(debug_assertions)]
use gpui_component::ActiveTheme;
#[cfg(debug_assertions)]
use oden_graph::fixtures::{construct_graph, fully_connected};
#[cfg(debug_assertions)]
use oden_graph::forces::LinkForce;
#[cfg(debug_assertions)]
use oden_graph::simulation::{Graph, Simulation};

#[cfg(debug_assertions)]
pub struct GraphView {
    simulation: Simulation,
}

#[cfg(debug_assertions)]
impl GraphView {
    pub fn new() -> Self {
        let graph: Graph = construct_graph(50, fully_connected);
        let simulation = Simulation::new(graph).with_force(Box::new(LinkForce::default()));
        Self { simulation }
    }

    fn to_pixels(point: Point<f32>, view_port: Size<Pixels>) -> Point<Pixels> {
        // TODO: handle edge cases
        // for now scaling is 1 to 1 from floats to pixels.
        Point {
            x: px(point.x) + view_port.width / 2.,
            y: px(point.y) + view_port.height / 2.,
        }
    }

    fn draw_vertex(point: Point<f32>, vertex_radius: f32, view_port: Size<Pixels>) -> Path<Pixels> {
        let point = Self::to_pixels(point, view_port);
        let radius_vector = Point {
            x: px(vertex_radius),
            y: px(0.),
        };
        let radii = Point {
            x: px(vertex_radius),
            y: px(vertex_radius),
        };
        let mut builder = PathBuilder::fill();
        builder.move_to(point);
        // draw the first half of the circle
        builder.line_to(point + radius_vector);
        builder.arc_to(radii, px(0.), false, false, point - radius_vector);
        // draw the second half
        builder.arc_to(radii, px(0.), false, false, point + radius_vector);
        builder.build().unwrap()
    }

    pub fn draw_edge(
        source: Point<f32>,
        target: Point<f32>,
        view_port: Size<Pixels>,
    ) -> Path<Pixels> {
        let source = Self::to_pixels(source, view_port);
        let target = Self::to_pixels(target, view_port);
        let mut builder = PathBuilder::stroke(px(1.));
        builder.move_to(source);
        builder.line_to(target);
        builder.build().unwrap()
    }
}

#[cfg(debug_assertions)]
impl Render for GraphView {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::prelude::Context<Self>,
    ) -> impl gpui::prelude::IntoElement {
        let vertices_color = cx.theme().red_light;
        let edges_color = cx.theme().green_light;
        if !self.simulation.should_stop {
            self.simulation.tick();
            window.request_animation_frame();
        }
        let positions = self.simulation.positions();
        let edges = self.simulation.edges();
        div().w_full().h_full().child(
            canvas(
                move |_bounds, window, _cx| {
                    let edges: Vec<Path<Pixels>> = edges
                        .iter()
                        .map(|(source, target)| {
                            Self::draw_edge(*source, *target, window.viewport_size())
                        })
                        .collect();

                    let vertices: Vec<Path<Pixels>> = positions
                        .iter()
                        .map(|position| Self::draw_vertex(*position, 5.0, window.viewport_size()))
                        .collect();

                    (vertices, edges)
                },
                move |_bounds, (vertices, edges), window, _cx| {
                    for path in edges {
                        window.paint_path(path, edges_color);
                    }
                    for path in vertices {
                        window.paint_path(path, vertices_color);
                    }
                },
            )
            .size_full(),
        )
    }
}
