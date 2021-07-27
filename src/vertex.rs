use vulkano::impl_vertex;

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Self { pos, color }
    }
}

impl_vertex!(Vertex, pos, color);

pub fn vertecies() -> [Vertex; 4] {
    [
        Vertex::new([-0.5, -0.5], [1.0, 0.0, 0.0]),
        Vertex::new([0.5, -0.5], [0.0, 1.0, 0.0]),
        Vertex::new([0.5, 0.5], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5, 0.5], [1.0, 1.0, 1.0])
    ]
}

pub fn indices() -> [u16; 6] {
    [0, 1, 2, 2, 3, 0]
}