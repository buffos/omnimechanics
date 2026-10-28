use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    Line2,
    Line3,
    Tri3,
    Tri6,
    Quad4,
    Quad8,
    Tet4,
    Tet10,
    Hex8,
    Hex20,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryElement {
    pub id: usize,
    pub elem_type: ElementType,
    pub nodes: Vec<usize>,
    pub tag: String,
    pub normal: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeElement {
    pub id: usize,
    pub elem_type: ElementType,
    pub nodes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshStats {
    pub num_nodes: usize,
    pub num_boundary_elements: usize,
    pub num_volume_elements: usize,
    pub dimension: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub dimension: usize,
    pub nodes: Vec<[f64; 3]>,
    pub boundary_elements: Vec<BoundaryElement>,
    pub volume_elements: Vec<VolumeElement>,
    pub stats: MeshStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "preset", content = "params")]
pub enum MeshGeneratorRequest {
    Beam2D {
        length: f64,
        height: f64,
        nx: usize,
        ny: usize,
        use_triangles: bool,
    },
    PlateWithHole2D {
        width: f64,
        height: f64,
        radius: f64,
        radial_divisions: usize,
        tangential_divisions: usize,
    },
    Beam3D {
        length: f64,
        height: f64,
        depth: f64,
        nx: usize,
        ny: usize,
        nz: usize,
        use_tetrahedra: bool,
    },
    BlockWithCavity3D {
        size: f64,
        radius: f64,
        divisions: usize,
    },
}
