use super::types::{BoundaryElement, ElementType, MeshData, MeshStats, VolumeElement};

pub fn generate_beam_3d(
    length: f64,
    height: f64,
    depth: f64,
    nx: usize,
    ny: usize,
    nz: usize,
    use_tetrahedra: bool,
) -> MeshData {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let nz = nz.max(1);

    let mut nodes = Vec::new();
    let dx = length / nx as f64;
    let dy = height / ny as f64;
    let dz = depth / nz as f64;

    let node_idx = |i: usize, j: usize, k: usize| -> usize {
        k * (nx + 1) * (ny + 1) + j * (nx + 1) + i
    };

    // Generate 3D grid nodes
    for k in 0..=nz {
        let z = k as f64 * dz;
        for j in 0..=ny {
            let y = j as f64 * dy;
            for i in 0..=nx {
                let x = i as f64 * dx;
                nodes.push([x, y, z]);
            }
        }
    }

    let mut volume_elements = Vec::new();
    let mut boundary_elements = Vec::new();
    let mut vol_id = 0;
    let mut bnd_id = 0;

    // Generate volume elements (Hex8 or 6 Tet4 per cube)
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let n0 = node_idx(i, j, k);
                let n1 = node_idx(i + 1, j, k);
                let n2 = node_idx(i + 1, j + 1, k);
                let n3 = node_idx(i, j + 1, k);
                let n4 = node_idx(i, j, k + 1);
                let n5 = node_idx(i + 1, j, k + 1);
                let n6 = node_idx(i + 1, j + 1, k + 1);
                let n7 = node_idx(i, j + 1, k + 1);

                if use_tetrahedra {
                    // Standard Kuhn triangulation splitting a cube into 6 tetrahedra
                    let tets = [
                        [n0, n1, n2, n6],
                        [n0, n2, n3, n6],
                        [n0, n3, n7, n6],
                        [n0, n7, n4, n6],
                        [n0, n4, n5, n6],
                        [n0, n5, n1, n6],
                    ];
                    for tet in tets {
                        volume_elements.push(VolumeElement {
                            id: vol_id,
                            elem_type: ElementType::Tet4,
                            nodes: tet.to_vec(),
                        });
                        vol_id += 1;
                    }
                } else {
                    volume_elements.push(VolumeElement {
                        id: vol_id,
                        elem_type: ElementType::Hex8,
                        nodes: vec![n0, n1, n2, n3, n4, n5, n6, n7],
                    });
                    vol_id += 1;
                }
            }
        }
    }

    // Generate 2D surface boundary elements (Tri3) on the 6 outer faces
    // 1. Left (x = 0): normal = (-1, 0, 0)
    for k in 0..nz {
        for j in 0..ny {
            let n0 = node_idx(0, j, k);
            let n1 = node_idx(0, j, k + 1);
            let n2 = node_idx(0, j + 1, k + 1);
            let n3 = node_idx(0, j + 1, k);

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n1, n2],
                tag: "Left".to_string(),
                normal: [-1.0, 0.0, 0.0],
            });
            bnd_id += 1;
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n3],
                tag: "Left".to_string(),
                normal: [-1.0, 0.0, 0.0],
            });
            bnd_id += 1;
        }
    }

    // 2. Right (x = length): normal = (1, 0, 0)
    for k in 0..nz {
        for j in 0..ny {
            let n0 = node_idx(nx, j, k);
            let n1 = node_idx(nx, j + 1, k);
            let n2 = node_idx(nx, j + 1, k + 1);
            let n3 = node_idx(nx, j, k + 1);

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n1, n2],
                tag: "Right".to_string(),
                normal: [1.0, 0.0, 0.0],
            });
            bnd_id += 1;
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n3],
                tag: "Right".to_string(),
                normal: [1.0, 0.0, 0.0],
            });
            bnd_id += 1;
        }
    }

    // 3. Bottom (y = 0): normal = (0, -1, 0)
    for k in 0..nz {
        for i in 0..nx {
            let n0 = node_idx(i, 0, k);
            let n1 = node_idx(i + 1, 0, k);
            let n2 = node_idx(i + 1, 0, k + 1);
            let n3 = node_idx(i, 0, k + 1);

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n3, n2],
                tag: "Bottom".to_string(),
                normal: [0.0, -1.0, 0.0],
            });
            bnd_id += 1;
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n1],
                tag: "Bottom".to_string(),
                normal: [0.0, -1.0, 0.0],
            });
            bnd_id += 1;
        }
    }

    // 4. Top (y = height): normal = (0, 1, 0)
    for k in 0..nz {
        for i in 0..nx {
            let n0 = node_idx(i, ny, k);
            let n1 = node_idx(i + 1, ny, k);
            let n2 = node_idx(i + 1, ny, k + 1);
            let n3 = node_idx(i, ny, k + 1);

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n1, n2],
                tag: "Top".to_string(),
                normal: [0.0, 1.0, 0.0],
            });
            bnd_id += 1;
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n3],
                tag: "Top".to_string(),
                normal: [0.0, 1.0, 0.0],
            });
            bnd_id += 1;
        }
    }

    // 5. Back (z = 0): normal = (0, 0, -1)
    for j in 0..ny {
        for i in 0..nx {
            let n0 = node_idx(i, j, 0);
            let n1 = node_idx(i, j + 1, 0);
            let n2 = node_idx(i + 1, j + 1, 0);
            let n3 = node_idx(i + 1, j, 0);

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n1, n2],
                tag: "Back".to_string(),
                normal: [0.0, 0.0, -1.0],
            });
            bnd_id += 1;
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n3],
                tag: "Back".to_string(),
                normal: [0.0, 0.0, -1.0],
            });
            bnd_id += 1;
        }
    }

    // 6. Front (z = depth): normal = (0, 0, 1)
    for j in 0..ny {
        for i in 0..nx {
            let n0 = node_idx(i, j, nz);
            let n1 = node_idx(i + 1, j, nz);
            let n2 = node_idx(i + 1, j + 1, nz);
            let n3 = node_idx(i, j + 1, nz);

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n1, n2],
                tag: "Front".to_string(),
                normal: [0.0, 0.0, 1.0],
            });
            bnd_id += 1;
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n3],
                tag: "Front".to_string(),
                normal: [0.0, 0.0, 1.0],
            });
            bnd_id += 1;
        }
    }

    MeshData {
        dimension: 3,
        stats: MeshStats {
            num_nodes: nodes.len(),
            num_boundary_elements: boundary_elements.len(),
            num_volume_elements: volume_elements.len(),
            dimension: 3,
        },
        nodes,
        boundary_elements,
        volume_elements,
    }
}

#[cfg(test)]
#[path = "generators_3d_test.rs"]
mod tests;
