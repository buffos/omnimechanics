use super::types::{BoundaryElement, ElementType, MeshData, MeshStats, VolumeElement};

pub fn generate_beam_2d(
    length: f64,
    height: f64,
    nx: usize,
    ny: usize,
    use_triangles: bool,
) -> MeshData {
    let nx = nx.max(1);
    let ny = ny.max(1);

    let mut nodes = Vec::new();
    let dx = length / nx as f64;
    let dy = height / ny as f64;

    // Node index lookup: (i, j) -> index
    let node_idx = |i: usize, j: usize| -> usize { j * (nx + 1) + i };

    // Generate grid nodes
    for j in 0..=ny {
        let y = j as f64 * dy;
        for i in 0..=nx {
            let x = i as f64 * dx;
            nodes.push([x, y, 0.0]);
        }
    }

    let mut volume_elements = Vec::new();
    let mut boundary_elements = Vec::new();
    let mut vol_id = 0;
    let mut bnd_id = 0;

    // Generate volume elements (Quad4 or Tri3)
    for j in 0..ny {
        for i in 0..nx {
            let n0 = node_idx(i, j);
            let n1 = node_idx(i + 1, j);
            let n2 = node_idx(i + 1, j + 1);
            let n3 = node_idx(i, j + 1);

            if use_triangles {
                volume_elements.push(VolumeElement {
                    id: vol_id,
                    elem_type: ElementType::Tri3,
                    nodes: vec![n0, n1, n2],
                });
                vol_id += 1;
                volume_elements.push(VolumeElement {
                    id: vol_id,
                    elem_type: ElementType::Tri3,
                    nodes: vec![n0, n2, n3],
                });
                vol_id += 1;
            } else {
                volume_elements.push(VolumeElement {
                    id: vol_id,
                    elem_type: ElementType::Quad4,
                    nodes: vec![n0, n1, n2, n3],
                });
                vol_id += 1;
            }
        }
    }

    // Generate 1D Boundary Elements (Line2) for BEM with proper outward normals
    // 1. Bottom boundary (y = 0): normal = (0, -1, 0)
    for i in 0..nx {
        let n0 = node_idx(i, 0);
        let n1 = node_idx(i + 1, 0);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: "Bottom".to_string(),
            normal: [0.0, -1.0, 0.0],
        });
        bnd_id += 1;
    }

    // 2. Right boundary (x = length): normal = (1, 0, 0)
    for j in 0..ny {
        let n0 = node_idx(nx, j);
        let n1 = node_idx(nx, j + 1);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: "Right".to_string(),
            normal: [1.0, 0.0, 0.0],
        });
        bnd_id += 1;
    }

    // 3. Top boundary (y = height): normal = (0, 1, 0)
    for i in (0..nx).rev() {
        let n0 = node_idx(i + 1, ny);
        let n1 = node_idx(i, ny);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: "Top".to_string(),
            normal: [0.0, 1.0, 0.0],
        });
        bnd_id += 1;
    }

    // 4. Left boundary (x = 0): normal = (-1, 0, 0)
    for j in (0..ny).rev() {
        let n0 = node_idx(0, j + 1);
        let n1 = node_idx(0, j);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: "Left".to_string(),
            normal: [-1.0, 0.0, 0.0],
        });
        bnd_id += 1;
    }

    MeshData {
        dimension: 2,
        stats: MeshStats {
            num_nodes: nodes.len(),
            num_boundary_elements: boundary_elements.len(),
            num_volume_elements: volume_elements.len(),
            dimension: 2,
        },
        nodes,
        boundary_elements,
        volume_elements,
    }
}

pub fn generate_plate_with_hole_2d(
    width: f64,
    height: f64,
    radius: f64,
    radial_divs: usize,
    angular_divs: usize,
) -> MeshData {
    let nr = radial_divs.max(2);
    let n_theta = (angular_divs.max(8) / 4) * 4; // multiple of 4

    let mut nodes = Vec::new();
    let mut boundary_elements = Vec::new();
    let mut volume_elements = Vec::new();

    let half_w = width / 2.0;
    let half_h = height / 2.0;

    // Node index: (ring, theta_idx) -> index
    let node_idx = |ring: usize, th: usize| -> usize {
        let th_norm = th % n_theta;
        ring * n_theta + th_norm
    };

    // Generate concentric rings from r = radius to outer boundary
    for ring in 0..=nr {
        let s = ring as f64 / nr as f64; // 0.0 at hole, 1.0 at outer boundary
        for th in 0..n_theta {
            let angle = (th as f64 / n_theta as f64) * 2.0 * std::f64::consts::PI;
            let cos_th = angle.cos();
            let sin_th = angle.sin();

            // Inner circle point
            let x_inner = radius * cos_th;
            let y_inner = radius * sin_th;

            // Outer box boundary point along the ray
            let (x_outer, y_outer) = ray_to_box_intersection(cos_th, sin_th, half_w, half_h);

            // Interpolate (geometric progression for boundary clustering)
            let s_graded = s.powf(1.1);
            let x = (1.0 - s_graded) * x_inner + s_graded * x_outer;
            let y = (1.0 - s_graded) * y_inner + s_graded * y_outer;

            nodes.push([x, y, 0.0]);
        }
    }

    // Generate volume elements (Tri3)
    let mut vol_id = 0;
    for ring in 0..nr {
        for th in 0..n_theta {
            let n0 = node_idx(ring, th);
            let n1 = node_idx(ring, th + 1);
            let n2 = node_idx(ring + 1, th + 1);
            let n3 = node_idx(ring + 1, th);

            volume_elements.push(VolumeElement {
                id: vol_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n1, n2],
            });
            vol_id += 1;
            volume_elements.push(VolumeElement {
                id: vol_id,
                elem_type: ElementType::Tri3,
                nodes: vec![n0, n2, n3],
            });
            vol_id += 1;
        }
    }

    // Generate boundary elements
    let mut bnd_id = 0;

    // 1. Hole Boundary (ring = 0): Normal points into the hole (away from material)
    for th in 0..n_theta {
        let n0 = node_idx(0, th);
        let n1 = node_idx(0, th + 1);
        let mid_angle = ((th as f64 + 0.5) / n_theta as f64) * 2.0 * std::f64::consts::PI;
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: "Hole".to_string(),
            normal: [-mid_angle.cos(), -mid_angle.sin(), 0.0],
        });
        bnd_id += 1;
    }

    // 2. Outer boundary (ring = nr)
    let quarter = n_theta / 4;
    for th in 0..n_theta {
        let n0 = node_idx(nr, th);
        let n1 = node_idx(nr, th + 1);

        let tag = if th < quarter / 2 || th >= n_theta - quarter / 2 {
            "Right"
        } else if th < quarter + quarter / 2 {
            "Top"
        } else if th < 2 * quarter + quarter / 2 {
            "Left"
        } else {
            "Bottom"
        };

        let normal = match tag {
            "Right" => [1.0, 0.0, 0.0],
            "Top" => [0.0, 1.0, 0.0],
            "Left" => [-1.0, 0.0, 0.0],
            _ => [0.0, -1.0, 0.0],
        };

        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: tag.to_string(),
            normal,
        });
        bnd_id += 1;
    }

    MeshData {
        dimension: 2,
        stats: MeshStats {
            num_nodes: nodes.len(),
            num_boundary_elements: boundary_elements.len(),
            num_volume_elements: volume_elements.len(),
            dimension: 2,
        },
        nodes,
        boundary_elements,
        volume_elements,
    }
}

fn ray_to_box_intersection(dx: f64, dy: f64, half_w: f64, half_h: f64) -> (f64, f64) {
    let tx = if dx.abs() > 1e-9 { half_w / dx.abs() } else { f64::MAX };
    let ty = if dy.abs() > 1e-9 { half_h / dy.abs() } else { f64::MAX };
    let t = tx.min(ty);
    (t * dx, t * dy)
}

#[cfg(test)]
#[path = "generators_2d_test.rs"]
mod tests;
