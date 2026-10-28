pub mod generators_2d;
pub mod generators_3d;
pub mod types;

use generators_2d::{generate_beam_2d, generate_plate_with_hole_2d};
use generators_3d::generate_beam_3d;
pub use types::*;

pub fn generate_mesh(req: MeshGeneratorRequest) -> MeshData {
    match req {
        MeshGeneratorRequest::Beam2D {
            length,
            height,
            nx,
            ny,
            use_triangles,
        } => generate_beam_2d(length, height, nx, ny, use_triangles),

        MeshGeneratorRequest::PlateWithHole2D {
            width,
            height,
            radius,
            radial_divisions,
            tangential_divisions,
        } => generate_plate_with_hole_2d(
            width,
            height,
            radius,
            radial_divisions,
            tangential_divisions,
        ),

        MeshGeneratorRequest::Beam3D {
            length,
            height,
            depth,
            nx,
            ny,
            nz,
            use_tetrahedra,
        } => generate_beam_3d(length, height, depth, nx, ny, nz, use_tetrahedra),

        MeshGeneratorRequest::BlockWithCavity3D {
            size,
            radius: _,
            divisions,
        } => {
            // Default 3D block with cavity fallback to beam 3D
            generate_beam_3d(size, size, size, divisions, divisions, divisions, true)
        }
    }
}
