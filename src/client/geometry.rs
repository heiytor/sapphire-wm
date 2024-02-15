#[derive(Clone)]
pub struct ClientGeometry {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub border: u32,
    pub paddings: [u32; 4],
}
