use std::{collections::HashMap, f32::consts::PI};

use gpui::Point;
use uuid::Uuid;

use crate::forces::Force;

pub struct SimulationOptions {
    // decay options
    pub velocity_decay: f32,
    pub alpha_target: f32,
    pub alpha_decay: f32,
    pub alpha_min: f32,

    // inital radius and angles for position seeding
    pub initial_radius: f32,
    pub initial_angle: f32,
}

pub struct Simulation {
    pub options: SimulationOptions,
    pub graph: Graph,
    pub should_stop: bool,
    pub alpha: f32,
    pub forces: Vec<Box<dyn Force>>,
}

pub struct Graph {
    pub nodes: HashMap<Uuid, Node>,
    pub adjacency_list: HashMap<Uuid, Vec<Uuid>>,
}

pub struct Node {
    pub position: Point<f32>,
    pub velocity: Point<f32>,
}

impl Simulation {
    pub fn new(graph: Graph) -> Self {
        // default values are taken from the d3-force repository
        let options: SimulationOptions = SimulationOptions {
            velocity_decay: 0.4,
            alpha_target: 0.0,
            alpha_decay: 0.0228,
            alpha_min: 0.001,
            initial_radius: 50.0,
            initial_angle: PI * (3.0 - f32::sqrt(5.0)),
        };
        let mut simulation = Self {
            options,
            graph,
            should_stop: false,
            alpha: 1.0,
            forces: Vec::new(),
        };
        simulation.initialize_nodes();
        simulation
    }

    pub fn with_force(mut self, force: Box<dyn Force>) -> Self {
        self.forces.push(force);
        self
    }

    fn initialize_nodes(&mut self) {
        let ids: Vec<Uuid> = self.graph.nodes.keys().copied().collect();
        for (i, id) in ids.iter().enumerate() {
            // initialize node positions in a spiral
            let node = self.graph.nodes.get_mut(id).unwrap();
            let angle = self.options.initial_angle * i as f32;
            let radius = self.options.initial_radius * (i as f32 + 0.5).sqrt();
            node.position.x = radius * angle.cos();
            node.position.y = radius * angle.sin();
        }
    }

    pub fn tick(&mut self) {
        self.alpha += (self.options.alpha_target - self.alpha) * self.options.alpha_decay;
        if self.alpha <= self.options.alpha_min {
            tracing::debug!(alpha = self.alpha, "simulation converged, stopping");
            self.should_stop = true;
            return;
        }
        for force in &mut self.forces {
            // apply the registered forces to the nodes.
            // forces will influence the velocity of the nodes.
            force.apply_force(self.alpha, &mut self.graph);
        }
        for node in self.graph.nodes.values_mut() {
            node.velocity *= 1.0 - self.options.velocity_decay;
            node.position += node.velocity;
        }
    }

    // returns an array containing the positions of all the nodes in the graph.
    pub fn positions(&self) -> Vec<Point<f32>> {
        self.graph
            .nodes
            .values()
            .map(|node| node.position)
            .collect()
    }

    // returns the edges of the graph to draw by iterating over the adjacency list.
    pub fn edges(&self) -> Vec<(Point<f32>, Point<f32>)> {
        let mut edges: Vec<(Point<f32>, Point<f32>)> = Vec::new();
        for source_id in self.graph.adjacency_list.keys() {
            let source_node = self.graph.nodes.get(source_id).unwrap();
            let neighbours: &Vec<Uuid> = self.graph.adjacency_list.get(source_id).unwrap();
            for neighbour_id in neighbours {
                let neighbour_node = self.graph.nodes.get(neighbour_id).unwrap();
                edges.push((source_node.position, neighbour_node.position));
            }
        }
        edges
    }

    pub fn stop(&mut self) {
        self.should_stop = true;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gpui::Point;
    use uuid::Uuid;

    use crate::simulation::{Graph, Node, Simulation};

    // instantiates a Graph object with `size` nodes.
    // all nodes are disconnected, for now.
    fn construct_graph(size: usize) -> Graph {
        let mut nodes = HashMap::new();
        for _ in 0..size {
            nodes.insert(
                Uuid::new_v4(),
                Node {
                    position: Point::default(),
                    velocity: Point::default(),
                },
            );
        }
        Graph {
            nodes,
            adjacency_list: HashMap::new(),
        }
    }

    #[test]
    fn test_alpha_should_decay_after_tick() {
        let graph = construct_graph(1);
        let mut simulation = Simulation::new(graph);
        let alpha_before = simulation.alpha;
        simulation.tick();
        let alpha_after = simulation.alpha;
        assert!(alpha_before > alpha_after);
    }

    #[test]
    fn test_node_positions_should_be_initialized() {
        let graph = construct_graph(1);
        let simulation = Simulation::new(graph);
        let positions = &simulation.positions();
        let node = positions.first().unwrap();
        assert_eq!(
            node.x,
            simulation.options.initial_radius * f32::sqrt(2.) / 2.
        );
        assert_eq!(node.y, 0.)
    }

    #[test]
    fn test_should_stop_after_thousand_ticks() {
        let graph = construct_graph(1);
        let mut simulation = Simulation::new(graph);
        for _ in 0..1000 {
            simulation.tick();
        }
        assert!(simulation.should_stop)
    }
}
