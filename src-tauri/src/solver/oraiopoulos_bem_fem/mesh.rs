use crate::mesh::types::MeshData;

pub struct DecoupledBndElem {
    pub p0: [f64; 2],
    pub p1: [f64; 2],
    pub normal: [f64; 2],
    pub length: f64,
    pub tag: String,
}

/// Structure representing a 2D planar quadrilateral boundary element in 3D.
#[derive(Debug, Clone)]
pub struct DecoupledBndElem3D {
    pub p0: [f64; 3],
    pub v1: [f64; 3],
    pub v2: [f64; 3],
    pub normal: [f64; 3],
    pub area: f64,
    pub center: [f64; 3],
    pub tag: String,
}

/// Structure representing a planar triangular boundary element in 3D.
#[derive(Debug, Clone)]
pub struct Tri3BndElem3D {
    pub p0: [f64; 3],
    pub p1: [f64; 3],
    pub p2: [f64; 3],
    pub normal: [f64; 3],
    pub area: f64,
    pub center: [f64; 3],
    pub tag: String,
}

impl Tri3BndElem3D {
    pub fn new(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], tag: String) -> Self {
        let v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = v1[1] * v2[2] - v1[2] * v2[1];
        let ny = v1[2] * v2[0] - v1[0] * v2[2];
        let nz = v1[0] * v2[1] - v1[1] * v2[0];
        let n_len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-14);
        let area = 0.5 * n_len;
        let normal = [nx / n_len, ny / n_len, nz / n_len];
        let center = [
            (p0[0] + p1[0] + p2[0]) / 3.0,
            (p0[1] + p1[1] + p2[1]) / 3.0,
            (p0[2] + p1[2] + p2[2]) / 3.0,
        ];
        Self { p0, p1, p2, normal, area, center, tag }
    }
}

/// Evaluates 3D Kelvin fundamental solution tensors U*(x, xi) and T*(x, xi).
/// x is field point on boundary, xi is source / interior evaluation point.
/// Returns (U*, T*) as 3x3 matrices: [[xx, xy, xz], [yx, yy, yz], [zx, zy, zz]].
pub fn kelvin_3d_kernels(
    x: [f64; 3],
    xi: [f64; 3],
    normal_x: [f64; 3],
    mu: f64,
    nu: f64,
) -> ([[f64; 3]; 3], [[f64; 3]; 3]) {
    let rx = x[0] - xi[0];
    let ry = x[1] - xi[1];
    let rz = x[2] - xi[2];
    let r = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-10);

    let dr = [rx / r, ry / r, rz / r];
    let dr_dn = dr[0] * normal_x[0] + dr[1] * normal_x[1] + dr[2] * normal_x[2];

    let factor_u = 1.0 / (16.0 * std::f64::consts::PI * mu * (1.0 - nu) * r);
    let kappa = 3.0 - 4.0 * nu;

    let mut u_star = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let delta = if i == j { 1.0 } else { 0.0 };
            u_star[i][j] = factor_u * (kappa * delta + dr[i] * dr[j]);
        }
    }

    let factor_t = -1.0 / (8.0 * std::f64::consts::PI * (1.0 - nu) * r * r);
    let one_minus_2nu = 1.0 - 2.0 * nu;

    let mut t_star = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let delta = if i == j { 1.0 } else { 0.0 };
            t_star[i][j] = factor_t
                * (dr_dn * (one_minus_2nu * delta + 3.0 * dr[i] * dr[j])
                    + one_minus_2nu * (dr[j] * normal_x[i] - dr[i] * normal_x[j]));
        }
    }

    (u_star, t_star)
}

/// Generates a decoupled 3D boundary surface mesh for the rectangular beam
/// [0, length] x [0, height] x [0, depth] with regular quad elements on all 6 faces.
pub fn generate_decoupled_boundary_mesh_3d(
    length: f64,
    height: f64,
    depth: f64,
    nx: usize,
    ny: usize,
    nz: usize,
) -> Vec<DecoupledBndElem3D> {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let nz = nz.max(1);

    let dx = length / nx as f64;
    let dy = height / ny as f64;
    let dz = depth / nz as f64;

    let total_elems = 2 * (ny * nz + nx * nz + nx * ny);
    let mut elems = Vec::with_capacity(total_elems);

    // 1. Left face (x = 0): normal [-1, 0, 0]
    for k in 0..nz {
        let z = k as f64 * dz;
        for j in 0..ny {
            let y = j as f64 * dy;
            elems.push(DecoupledBndElem3D {
                p0: [0.0, y, z],
                v1: [0.0, dy, 0.0],
                v2: [0.0, 0.0, dz],
                normal: [-1.0, 0.0, 0.0],
                area: dy * dz,
                center: [0.0, y + 0.5 * dy, z + 0.5 * dz],
                tag: "Left".to_string(),
            });
        }
    }

    // 2. Right face (x = length): normal [1, 0, 0]
    for k in 0..nz {
        let z = k as f64 * dz;
        for j in 0..ny {
            let y = j as f64 * dy;
            elems.push(DecoupledBndElem3D {
                p0: [length, y, z],
                v1: [0.0, dy, 0.0],
                v2: [0.0, 0.0, dz],
                normal: [1.0, 0.0, 0.0],
                area: dy * dz,
                center: [length, y + 0.5 * dy, z + 0.5 * dz],
                tag: "Right".to_string(),
            });
        }
    }

    // 3. Bottom face (y = 0): normal [0, -1, 0]
    for k in 0..nz {
        let z = k as f64 * dz;
        for i in 0..nx {
            let x = i as f64 * dx;
            elems.push(DecoupledBndElem3D {
                p0: [x, 0.0, z],
                v1: [dx, 0.0, 0.0],
                v2: [0.0, 0.0, dz],
                normal: [0.0, -1.0, 0.0],
                area: dx * dz,
                center: [x + 0.5 * dx, 0.0, z + 0.5 * dz],
                tag: "Bottom".to_string(),
            });
        }
    }

    // 4. Top face (y = height): normal [0, 1, 0]
    for k in 0..nz {
        let z = k as f64 * dz;
        for i in 0..nx {
            let x = i as f64 * dx;
            elems.push(DecoupledBndElem3D {
                p0: [x, height, z],
                v1: [dx, 0.0, 0.0],
                v2: [0.0, 0.0, dz],
                normal: [0.0, 1.0, 0.0],
                area: dx * dz,
                center: [x + 0.5 * dx, height, z + 0.5 * dz],
                tag: "Top".to_string(),
            });
        }
    }

    // 5. Back face (z = 0): normal [0, 0, -1]
    for j in 0..ny {
        let y = j as f64 * dy;
        for i in 0..nx {
            let x = i as f64 * dx;
            elems.push(DecoupledBndElem3D {
                p0: [x, y, 0.0],
                v1: [dx, 0.0, 0.0],
                v2: [0.0, dy, 0.0],
                normal: [0.0, 0.0, -1.0],
                area: dx * dy,
                center: [x + 0.5 * dx, y + 0.5 * dy, 0.0],
                tag: "Back".to_string(),
            });
        }
    }

    // 6. Front face (z = depth): normal [0, 0, 1]
    for j in 0..ny {
        let y = j as f64 * dy;
        for i in 0..nx {
            let x = i as f64 * dx;
            elems.push(DecoupledBndElem3D {
                p0: [x, y, depth],
                v1: [dx, 0.0, 0.0],
                v2: [0.0, dy, 0.0],
                normal: [0.0, 0.0, 1.0],
                area: dx * dy,
                center: [x + 0.5 * dx, y + 0.5 * dy, depth],
                tag: "Front".to_string(),
            });
        }
    }

    elems
}

/// Generates a decoupled boundary mesh of the rectangle [0, length] x [0, height]
/// with nx elements along horizontal edges and ny elements along vertical edges.
pub fn generate_decoupled_boundary_mesh(
    length: f64,
    height: f64,
    nx: usize,
    ny: usize,
) -> Vec<DecoupledBndElem> {
    let mut elems = Vec::with_capacity(2 * nx + 2 * ny);

    // 1. Bottom edge (y = 0): x from 0 to L, normal [0, -1]
    let dx = length / nx as f64;
    for i in 0..nx {
        let x0 = i as f64 * dx;
        let x1 = (i + 1) as f64 * dx;
        elems.push(DecoupledBndElem {
            p0: [x0, 0.0],
            p1: [x1, 0.0],
            normal: [0.0, -1.0],
            length: dx,
            tag: "Bottom".to_string(),
        });
    }

    // 2. Right edge (x = L): y from 0 to H, normal [1, 0]
    let dy = height / ny as f64;
    for j in 0..ny {
        let y0 = j as f64 * dy;
        let y1 = (j + 1) as f64 * dy;
        elems.push(DecoupledBndElem {
            p0: [length, y0],
            p1: [length, y1],
            normal: [1.0, 0.0],
            length: dy,
            tag: "Right".to_string(),
        });
    }

    // 3. Top edge (y = H): x from L to 0, normal [0, 1]
    for i in 0..nx {
        let x0 = length - i as f64 * dx;
        let x1 = length - (i + 1) as f64 * dx;
        elems.push(DecoupledBndElem {
            p0: [x0, height],
            p1: [x1, height],
            normal: [0.0, 1.0],
            length: dx,
            tag: "Top".to_string(),
        });
    }

    // 4. Left edge (x = 0): y from H to 0, normal [-1, 0]
    for j in 0..ny {
        let y0 = height - j as f64 * dy;
        let y1 = height - (j + 1) as f64 * dy;
        elems.push(DecoupledBndElem {
            p0: [0.0, y0],
            p1: [0.0, y1],
            normal: [-1.0, 0.0],
            length: dy,
            tag: "Left".to_string(),
        });
    }

    elems
}