with open('src-tauri/src/solver/dynamic_bem.rs', 'r', encoding='utf-8') as f:
    text = f.read()

lines = text.splitlines(True)
eol = '\r\n' if '\r\n' in text else '\n'

start_idx = None
end_idx = None

for i in range(1190, len(lines)):
    if "let num_nodes = mesh.nodes.len();" in lines[i]:
        start_idx = i
        break

for i in range(start_idx, len(lines)):
    if "u_vec\n" in lines[i] or "u_vec\r\n" in lines[i]:
        end_idx = i
        break

replacement_block = """    let num_nodes = mesh.nodes.len();
    let mut u_vec = Vec::with_capacity(num_nodes * dim);

    for i in 0..num_nodes {
        let p = mesh.nodes[i];
        let is_left_node = p[0] <= 1e-5;

        let (ux, uy) = if is_left_node {
            (Complex64::new(0.0, 0.0), Complex64::new(0.0, 0.0))
        } else {
            // Check if node lies directly on a boundary element: use quadratic interpolation
            let mut on_bnd = None;
            for j in 0..num_bnd_elems {
                let bnd = &mesh.boundary_elements[j];
                let p0 = mesh.nodes[bnd.nodes[0]];
                let p1 = mesh.nodes[bnd.nodes[1]];
                let len = elem_lengths[j];
                let vx = (p1[0] - p0[0]) / len;
                let vy = (p1[1] - p0[1]) / len;
                let dx = p[0] - p0[0];
                let dy = p[1] - p0[1];
                let proj = dx * vx + dy * vy;
                let perp = dx * (-vy) + dy * vx;
                if perp.abs() < 1e-4 && proj >= -1e-4 && proj <= len + 1e-4 {
                    let xi = (2.0 * proj / len - 1.0).max(-1.0).min(1.0);
                    on_bnd = Some((j, xi));
                    break;
                }
            }

            if let Some((j, xi)) = on_bnd {
                let n_vals = [
                    1.125 * xi * xi - 0.75 * xi,
                    1.0 - 2.25 * xi * xi,
                    1.125 * xi * xi + 0.75 * xi,
                ];
                let mut bnd_ux = Complex64::new(0.0, 0.0);
                let mut bnd_uy = Complex64::new(0.0, 0.0);
                for alpha in 0..3 {
                    let node_idx = j * 3 + alpha;
                    bnd_ux += Complex64::new(n_vals[alpha], 0.0) * bnd_u[node_idx][0];
                    bnd_uy += Complex64::new(n_vals[alpha], 0.0) * bnd_u[node_idx][1];
                }
                (bnd_ux, bnd_uy)
            } else {
                // Interior volume node: evaluate Somigliana representation
                let mut u_som = [Complex64::new(0.0, 0.0); 2];
                for j in 0..num_bnd_elems {
                    let bnd = &mesh.boundary_elements[j];
                    let p0 = [mesh.nodes[bnd.nodes[0]][0], mesh.nodes[bnd.nodes[0]][1]];
                    let p1 = [mesh.nodes[bnd.nodes[1]][0], mesh.nodes[bnd.nodes[1]][1]];
                    let len_j = elem_lengths[j];
                    let normal_j = [bnd.normal[0], bnd.normal[1]];

                    for (g_idx, &xi_g) in g_pts.iter().enumerate() {
                        let w_g = g_wts[g_idx];
                        let n0_geom = 0.5 * (1.0 - xi_g);
                        let n1_geom = 0.5 * (1.0 + xi_g);
                        let x_g = [n0_geom * p0[0] + n1_geom * p1[0], n0_geom * p0[1] + n1_geom * p1[1]];

                        let rx = x_g[0] - p[0];
                        let ry = x_g[1] - p[1];
                        let r = (rx * rx + ry * ry).sqrt().max(1e-10);
                        let dr = [rx / r, ry / r];

                        let u_star = dynamic_kelvin_u(r, dr, s, reference);
                        let t_star = dynamic_kelvin_t(r, dr, normal_j, s, reference);
                        let d_gamma = 0.5 * len_j * w_g;

                        let shapes = [
                            shape_coeffs[0][0] + shape_coeffs[0][1] * xi_g + shape_coeffs[0][2] * xi_g * xi_g,
                            shape_coeffs[1][0] + shape_coeffs[1][1] * xi_g + shape_coeffs[1][2] * xi_g * xi_g,
                            shape_coeffs[2][0] + shape_coeffs[2][1] * xi_g + shape_coeffs[2][2] * xi_g * xi_g,
                        ];

                        let uj = [
                            shapes[0] * bnd_u[j * 3][0] + shapes[1] * bnd_u[j * 3 + 1][0] + shapes[2] * bnd_u[j * 3 + 2][0],
                            shapes[0] * bnd_u[j * 3][1] + shapes[1] * bnd_u[j * 3 + 1][1] + shapes[2] * bnd_u[j * 3 + 2][1],
                        ];
                        let tj = [
                            shapes[0] * bnd_t[j * 3][0] + shapes[1] * bnd_t[j * 3 + 1][0] + shapes[2] * bnd_t[j * 3 + 2][0],
                            shapes[0] * bnd_t[j * 3][1] + shapes[1] * bnd_t[j * 3 + 1][1] + shapes[2] * bnd_t[j * 3 + 2][1],
                        ];

                        for d1 in 0..dim {
                            for d2 in 0..dim {
                                u_som[d1] += (u_star[d1][d2] * tj[d2] - t_star[d1][d2] * uj[d2]) * Complex64::new(d_gamma, 0.0);
                            }
                        }
                    }
                }
                (u_som[0], u_som[1])
            }
        };

        u_vec.push(ux);
        u_vec.push(uy);
    }
"""

replacement_lines = [l + eol for l in replacement_block.splitlines()]

new_lines = lines[:start_idx] + replacement_lines + [lines[end_idx]] + lines[end_idx+1:]
with open('src-tauri/src/solver/dynamic_bem.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print("Replaced reconstruction block successfully!")
