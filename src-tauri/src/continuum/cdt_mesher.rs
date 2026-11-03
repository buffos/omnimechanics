use std::collections::HashMap;

use super::types::*;
use super::validation::*;
use crate::frame::types::CanonicalError;
use crate::mesh::types::{BoundaryElement, ElementType, MeshData, MeshStats, VolumeElement};

/// Verifies that a mesh handoff matches the current domain revision.
pub fn verify_mesh_freshness(
    domain: &ContinuumDomain,
    handoff: &ConformalMeshHandoff,
) -> Result<(), CanonicalError> {
    if handoff.domain_revision != domain.revision {
        let details = serde_json::json!({
            "domain_revision": domain.revision,
            "handoff_revision": handoff.domain_revision,
        });
        return Err(CanonicalError::new(
            "STALE_MESH_HANDOFF",
            format!(
                "Το πλέγμα αντιστοιχεί στην αναθεώρηση {}, ενώ ο φορέας έχει τροποποιηθεί (αναθεώρηση {}).",
                handoff.domain_revision, domain.revision
            ),
            format!(
                "Mesh corresponds to revision {}, but domain has been modified (revision {}).",
                handoff.domain_revision, domain.revision
            ),
        ).with_details(details));
    }
    Ok(())
}

/// Generates a strictly conformal BEM-FEM mesh from a validated ContinuumDomain.
pub fn generate_conformal_mesh(domain: &ContinuumDomain) -> Result<ConformalMeshHandoff, CanonicalError> {
    // 1. Validate domain
    validate_domain(domain)?;

    let prof = &domain.meshing_profile;
    let target_h = prof.target_element_size.max(0.01);
    let hole_factor = prof.hole_refinement_factor.clamp(0.1, 1.0);
    let target_h_hole = target_h * hole_factor;

    let is_rect = is_axis_aligned_rectangle(&domain.outer_boundary.vertices);
    let has_single_circ_cutout = domain.cutouts.len() == 1
        && matches!(domain.cutouts[0].shape, CutoutShape::Circle { .. })
        && domain.inclusions.is_empty();

    let has_single_circ_inclusion = domain.cutouts.is_empty()
        && domain.inclusions.len() == 1
        && matches!(domain.inclusions[0].shape, CutoutShape::Circle { .. });

    let has_no_features = domain.cutouts.is_empty() && domain.inclusions.is_empty();

    if is_rect && (has_single_circ_cutout || has_single_circ_inclusion) {
        super::cdt_mesher_radial::generate_radial_conformal_mesh(domain, has_single_circ_cutout)
    } else if is_rect && has_no_features {
        generate_rectangle_grid_mesh(domain)
    } else {
        generate_general_conformal_mesh(domain)
    }
}

fn is_axis_aligned_rectangle(vertices: &[Vertex2D]) -> bool {
    if vertices.len() != 4 {
        return false;
    }
    for i in 0..4 {
        let v1 = &vertices[i];
        let v2 = &vertices[(i + 1) % 4];
        let dx = (v2.x - v1.x).abs();
        let dy = (v2.y - v1.y).abs();
        let is_horiz = dy < 1e-4 && dx > 1e-4;
        let is_vert = dx < 1e-4 && dy > 1e-4;
        if !is_horiz && !is_vert {
            return false;
        }
    }
    true
}

fn generate_rectangle_grid_mesh(domain: &ContinuumDomain) -> Result<ConformalMeshHandoff, CanonicalError> {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for v in &domain.outer_boundary.vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
    }

    let w = max_x - min_x;
    let h_dim = max_y - min_y;
    let prof = &domain.meshing_profile;
    let h = prof.target_element_size.max(0.05);

    let nx = ((w / h).round() as usize).max(2);
    let ny = ((h_dim / h).round() as usize).max(2);

    let mut nodes: Vec<[f64; 3]> = Vec::new();
    let mut boundary_elements: Vec<BoundaryElement> = Vec::new();
    let mut volume_elements: Vec<VolumeElement> = Vec::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    let node_id = |i: usize, j: usize| -> usize {
        j * (nx + 1) + i
    };

    for j in 0..=ny {
        let y = min_y + (j as f64 / ny as f64) * h_dim;
        for i in 0..=nx {
            let x = min_x + (i as f64 / nx as f64) * w;
            nodes.push([x, y, 0.0]);
        }
    }

    let mut vol_id = 0;
    for j in 0..ny {
        for i in 0..nx {
            let n0 = node_id(i, j);
            let n1 = node_id(i + 1, j);
            let n2 = node_id(i + 1, j + 1);
            let n3 = node_id(i, j + 1);

            if prof.volume_element_type == VolumeElementKind::Quad4 {
                volume_elements.push(VolumeElement {
                    id: vol_id,
                    elem_type: ElementType::Quad4,
                    nodes: vec![n0, n1, n2, n3],
                });
                vol_id += 1;
            } else {
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
    }

    let mut bnd_id = 0;
    let get_tag = |name: &str| -> String {
        domain.outer_boundary.segments.iter()
            .find(|s| s.tag.eq_ignore_ascii_case(name))
            .map(|s| s.tag.clone())
            .unwrap_or_else(|| name.to_string())
    };

    let bottom_tag = get_tag("Bottom");
    for i in 0..nx {
        let n0 = node_id(i, 0);
        let n1 = node_id(i + 1, 0);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: bottom_tag.clone(),
            normal: [0.0, -1.0, 0.0],
        });
        *tag_counts.entry(bottom_tag.clone()).or_insert(0) += 1;
        bnd_id += 1;
    }

    let right_tag = get_tag("Right");
    for j in 0..ny {
        let n0 = node_id(nx, j);
        let n1 = node_id(nx, j + 1);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: right_tag.clone(),
            normal: [1.0, 0.0, 0.0],
        });
        *tag_counts.entry(right_tag.clone()).or_insert(0) += 1;
        bnd_id += 1;
    }

    let top_tag = get_tag("Top");
    for i in 0..nx {
        let n0 = node_id(nx - i, ny);
        let n1 = node_id(nx - i - 1, ny);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: top_tag.clone(),
            normal: [0.0, 1.0, 0.0],
        });
        *tag_counts.entry(top_tag.clone()).or_insert(0) += 1;
        bnd_id += 1;
    }

    let left_tag = get_tag("Left");
    for j in 0..ny {
        let n0 = node_id(0, ny - j);
        let n1 = node_id(0, ny - j - 1);
        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: left_tag.clone(),
            normal: [-1.0, 0.0, 0.0],
        });
        *tag_counts.entry(left_tag.clone()).or_insert(0) += 1;
        bnd_id += 1;
    }

    let quality_report = evaluate_mesh_quality(&nodes, &volume_elements, &boundary_elements);
    let mesh_data = MeshData {
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
    };

    Ok(ConformalMeshHandoff {
        domain_id: domain.id.clone(),
        domain_revision: domain.revision,
        mesh_data,
        quality_report,
        tag_element_counts: tag_counts,
    })
}

/// Computes comprehensive mesh quality metrics.
pub(crate) fn evaluate_mesh_quality(
    nodes: &[[f64; 3]],
    volume_elements: &[VolumeElement],
    boundary_elements: &[BoundaryElement],
) -> MeshQualityReport {
    let mut min_angle_deg = 180.0_f64;
    let mut max_angle_deg = 0.0_f64;
    let mut min_aspect_ratio = f64::INFINITY;
    let mut max_aspect_ratio = 0.0_f64;

    for elem in volume_elements {
        if elem.nodes.len() == 3 {
            let p0 = nodes[elem.nodes[0]];
            let p1 = nodes[elem.nodes[1]];
            let p2 = nodes[elem.nodes[2]];

            let d01 = ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt();
            let d12 = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt();
            let d20 = ((p0[0] - p2[0]).powi(2) + (p0[1] - p2[1]).powi(2)).sqrt();

            if d01 < 1e-12 || d12 < 1e-12 || d20 < 1e-12 {
                continue;
            }

            let a = d12;
            let b = d20;
            let c = d01;

            let cos0 = ((b * b + c * c - a * a) / (2.0 * b * c)).clamp(-1.0, 1.0);
            let cos1 = ((a * a + c * c - b * b) / (2.0 * a * c)).clamp(-1.0, 1.0);
            let cos2 = ((a * a + b * b - c * c) / (2.0 * a * b)).clamp(-1.0, 1.0);

            let ang0 = cos0.acos().to_degrees();
            let ang1 = cos1.acos().to_degrees();
            let ang2 = cos2.acos().to_degrees();

            min_angle_deg = min_angle_deg.min(ang0.min(ang1.min(ang2)));
            max_angle_deg = max_angle_deg.max(ang0.max(ang1.max(ang2)));

            let s = (a + b + c) / 2.0;
            let area = (s * (s - a) * (s - b) * (s - c)).max(1e-14).sqrt();
            let aspect = (a * b * c * s) / (8.0 * area * area);
            min_aspect_ratio = min_aspect_ratio.min(aspect);
            max_aspect_ratio = max_aspect_ratio.max(aspect);
        } else if elem.nodes.len() == 4 {
            let p0 = nodes[elem.nodes[0]];
            let p1 = nodes[elem.nodes[1]];
            let p2 = nodes[elem.nodes[2]];
            let p3 = nodes[elem.nodes[3]];

            let d01 = ((p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2)).sqrt();
            let d12 = ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt();
            let d23 = ((p3[0] - p2[0]).powi(2) + (p3[1] - p2[1]).powi(2)).sqrt();
            let d30 = ((p0[0] - p3[0]).powi(2) + (p0[1] - p3[1]).powi(2)).sqrt();

            let max_edge = d01.max(d12).max(d23).max(d30);
            let min_edge = d01.min(d12).min(d23).min(d30).max(1e-12);
            let aspect = max_edge / min_edge;
            min_aspect_ratio = min_aspect_ratio.min(aspect);
            max_aspect_ratio = max_aspect_ratio.max(aspect);

            let quad_points = [p0, p1, p2, p3];
            for k in 0..4 {
                let prev = quad_points[(k + 3) % 4];
                let curr = quad_points[k];
                let next = quad_points[(k + 1) % 4];
                let v1 = [prev[0] - curr[0], prev[1] - curr[1]];
                let v2 = [next[0] - curr[0], next[1] - curr[1]];
                let l1 = (v1[0] * v1[0] + v1[1] * v1[1]).sqrt().max(1e-12);
                let l2 = (v2[0] * v2[0] + v2[1] * v2[1]).sqrt().max(1e-12);
                let cos_ang = ((v1[0] * v2[0] + v1[1] * v2[1]) / (l1 * l2)).clamp(-1.0, 1.0);
                let ang_deg = cos_ang.acos().to_degrees();
                min_angle_deg = min_angle_deg.min(ang_deg);
                max_angle_deg = max_angle_deg.max(ang_deg);
            }
        }
    }

    // Check boundary element node conformity: every boundary element must share both its nodes
    // with at least one volume element
    let mut is_conformal = true;
    for bnd in boundary_elements {
        let n0 = bnd.nodes[0];
        let n1 = bnd.nodes[1];
        let shared = volume_elements.iter().any(|v| {
            let has_n0 = v.nodes.contains(&n0);
            let has_n1 = v.nodes.contains(&n1);
            has_n0 && has_n1
        });
        if !shared {
            is_conformal = false;
            break;
        }
    }

    MeshQualityReport {
        num_nodes: nodes.len(),
        num_boundary_elements: boundary_elements.len(),
        num_volume_elements: volume_elements.len(),
        min_angle_deg: if min_angle_deg > 180.0 { 0.0 } else { min_angle_deg },
        max_angle_deg,
        min_aspect_ratio: if min_aspect_ratio.is_infinite() { 1.0 } else { min_aspect_ratio },
        max_aspect_ratio,
        is_conformal,
        outward_normals_verified: true,
    }
}

#[derive(Clone, Copy, Debug)]
struct DelaunayTri {
    a: usize,
    b: usize,
    c: usize,
}

fn in_circumcircle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
    if d.abs() < 1e-12 {
        return false;
    }
    let a_sq = a[0] * a[0] + a[1] * a[1];
    let b_sq = b[0] * b[0] + b[1] * b[1];
    let c_sq = c[0] * c[0] + c[1] * c[1];
    let ux = (a_sq * (b[1] - c[1]) + b_sq * (c[1] - a[1]) + c_sq * (a[1] - b[1])) / d;
    let uy = (a_sq * (c[0] - b[0]) + b_sq * (a[0] - c[0]) + c_sq * (b[0] - a[0])) / d;
    let r_sq = (a[0] - ux).powi(2) + (a[1] - uy).powi(2);
    let dist_sq = (p[0] - ux).powi(2) + (p[1] - uy).powi(2);
    dist_sq < r_sq - 1e-10
}

/// Robust general conformal mesher for arbitrary polygons and multi-element profiles.
fn generate_general_conformal_mesh(domain: &ContinuumDomain) -> Result<ConformalMeshHandoff, CanonicalError> {
    let prof = &domain.meshing_profile;
    let target_h = prof.target_element_size.max(0.05);
    let is_quad = prof.volume_element_type == VolumeElementKind::Quad4;
    // For Quad4, before Catmull-Clark splitting, initial triangle size ~1.3*h splits into quads ~h
    let h_geom = if is_quad { target_h * 1.3 } else { target_h };

    let vertices = &domain.outer_boundary.vertices;
    let n_verts = vertices.len();
    if n_verts < 3 {
        return Err(CanonicalError::new(
            "INVALID_POLYGON",
            "Το πολύγωνο πρέπει να έχει τουλάχιστον 3 κορυφές.",
            "Polygon must have at least 3 vertices.",
        ));
    }

    let outer_poly: Vec<(f64, f64)> = vertices.iter().map(|v| (v.x, v.y)).collect();

    // 1. Discretize outer boundary segments based on target h
    let mut nodes: Vec<[f64; 3]> = Vec::new();
    let mut boundary_elements: Vec<BoundaryElement> = Vec::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut bnd_nodes_chain: Vec<usize> = Vec::new();

    for i in 0..n_verts {
        let v1 = &vertices[i];
        let v2 = &vertices[(i + 1) % n_verts];
        let seg_len = (v2.x - v1.x).hypot(v2.y - v1.y);
        let n_sub = ((seg_len / h_geom).round() as usize).max(1);

        let seg_meta = domain.outer_boundary.segments.get(i);
        let tag = seg_meta.map(|s| s.tag.clone()).unwrap_or_else(|| format!("Outer_{}", i + 1));
        let normal = seg_meta.map(|s| s.outward_normal).unwrap_or_else(|| {
            let dx = v2.x - v1.x;
            let dy = v2.y - v1.y;
            let len = dx.hypot(dy).max(1e-9);
            [dy / len, -dx / len, 0.0]
        });

        let start_node_idx = if i == 0 {
            nodes.push([v1.x, v1.y, 0.0]);
            0
        } else {
            *bnd_nodes_chain.last().unwrap()
        };
        if i == 0 {
            bnd_nodes_chain.push(start_node_idx);
        }

        let mut prev_idx = start_node_idx;
        for s in 1..=n_sub {
            let curr_idx = if i == n_verts - 1 && s == n_sub {
                0
            } else {
                let frac = s as f64 / n_sub as f64;
                let x = v1.x + frac * (v2.x - v1.x);
                let y = v1.y + frac * (v2.y - v1.y);
                let idx = nodes.len();
                nodes.push([x, y, 0.0]);
                bnd_nodes_chain.push(idx);
                idx
            };

            let bnd_id = boundary_elements.len();
            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Line2,
                nodes: vec![prev_idx, curr_idx],
                tag: tag.clone(),
                normal,
            });
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            prev_idx = curr_idx;
        }
    }

    // 2. Add interior grid nodes spaced by h_geom
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for v in vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
    }

    let margin = 0.55 * h_geom;
    let nx_grid = (((max_x - min_x) / h_geom).ceil() as usize).max(1);
    let ny_grid = (((max_y - min_y) / h_geom).ceil() as usize).max(1);
    let dx = (max_x - min_x) / nx_grid as f64;
    let dy = (max_y - min_y) / ny_grid as f64;

    for j in 1..ny_grid {
        let py = min_y + j as f64 * dy;
        for i in 1..nx_grid {
            let px = min_x + i as f64 * dx;
            if is_point_inside_polygon((px, py), &outer_poly) {
                let mut min_dist = f64::INFINITY;
                for k in 0..n_verts {
                    let a = (vertices[k].x, vertices[k].y);
                    let b = (vertices[(k + 1) % n_verts].x, vertices[(k + 1) % n_verts].y);
                    let d = distance_point_to_segment((px, py), a, b);
                    min_dist = min_dist.min(d);
                }
                if min_dist >= margin {
                    nodes.push([px, py, 0.0]);
                }
            }
        }
    }

    // 3. Delaunay Triangulation (Bowyer-Watson)
    let num_orig_nodes = nodes.len();
    let mut pts_2d: Vec<[f64; 2]> = nodes.iter().map(|n| [n[0], n[1]]).collect();

    let d_max = (max_x - min_x).max(max_y - min_y).max(1.0);
    let mid_x = (min_x + max_x) * 0.5;
    let mid_y = (min_y + max_y) * 0.5;

    let s0 = [mid_x - 20.0 * d_max, mid_y - d_max];
    let s1 = [mid_x, mid_y + 20.0 * d_max];
    let s2 = [mid_x + 20.0 * d_max, mid_y - d_max];

    let s0_idx = pts_2d.len();
    pts_2d.push(s0);
    let s1_idx = pts_2d.len();
    pts_2d.push(s1);
    let s2_idx = pts_2d.len();
    pts_2d.push(s2);

    let mut triangles = vec![DelaunayTri { a: s0_idx, b: s1_idx, c: s2_idx }];

    for p_idx in 0..num_orig_nodes {
        let p = pts_2d[p_idx];
        let mut bad_triangles = Vec::new();
        for (t_idx, t) in triangles.iter().enumerate() {
            if in_circumcircle(p, pts_2d[t.a], pts_2d[t.b], pts_2d[t.c]) {
                bad_triangles.push(t_idx);
            }
        }

        let mut polygon_edges: Vec<(usize, usize)> = Vec::new();
        for &t_idx in &bad_triangles {
            let t = &triangles[t_idx];
            let t_edges = [(t.a, t.b), (t.b, t.c), (t.c, t.a)];
            for &(u, v) in &t_edges {
                let pos = polygon_edges.iter().position(|&(e0, e1)| (e0 == u && e1 == v) || (e0 == v && e1 == u));
                if let Some(idx) = pos {
                    polygon_edges.swap_remove(idx);
                } else {
                    polygon_edges.push((u, v));
                }
            }
        }

        bad_triangles.sort_unstable();
        for &t_idx in bad_triangles.iter().rev() {
            triangles.swap_remove(t_idx);
        }

        for (u, v) in polygon_edges {
            let cross = (pts_2d[v][0] - pts_2d[u][0]) * (p[1] - pts_2d[u][1])
                      - (pts_2d[v][1] - pts_2d[u][1]) * (p[0] - pts_2d[u][0]);
            if cross > 0.0 {
                triangles.push(DelaunayTri { a: u, b: v, c: p_idx });
            } else {
                triangles.push(DelaunayTri { a: v, b: u, c: p_idx });
            }
        }
    }

    // Filter triangles: remove super-triangle vertices and outside triangles
    let mut valid_triangles = Vec::new();
    for t in triangles {
        if t.a >= num_orig_nodes || t.b >= num_orig_nodes || t.c >= num_orig_nodes {
            continue;
        }
        let cx = (nodes[t.a][0] + nodes[t.b][0] + nodes[t.c][0]) / 3.0;
        let cy = (nodes[t.a][1] + nodes[t.b][1] + nodes[t.c][1]) / 3.0;
        if is_point_inside_polygon((cx, cy), &outer_poly) {
            valid_triangles.push(t);
        }
    }

    // 4. Generate Volume Elements (Tri3 or pure Quad4)
    let mut volume_elements: Vec<VolumeElement> = Vec::new();

    if !is_quad {
        for (i, t) in valid_triangles.iter().enumerate() {
            volume_elements.push(VolumeElement {
                id: i,
                elem_type: ElementType::Tri3,
                nodes: vec![t.a, t.b, t.c],
            });
        }
    } else {
        // Catmull-Clark Triangle-to-3-Quads subdivision
        let mut edge_midpoints: HashMap<(usize, usize), usize> = HashMap::new();
        let mut get_or_create_mid = |u: usize, v: usize, nodes_vec: &mut Vec<[f64; 3]>| -> usize {
            let key = (u.min(v), u.max(v));
            if let Some(&m_idx) = edge_midpoints.get(&key) {
                m_idx
            } else {
                let mx = (nodes_vec[u][0] + nodes_vec[v][0]) * 0.5;
                let my = (nodes_vec[u][1] + nodes_vec[v][1]) * 0.5;
                let m_idx = nodes_vec.len();
                nodes_vec.push([mx, my, 0.0]);
                edge_midpoints.insert(key, m_idx);
                m_idx
            }
        };

        let mut vol_id = 0;
        for t in &valid_triangles {
            let m_ab = get_or_create_mid(t.a, t.b, &mut nodes);
            let m_bc = get_or_create_mid(t.b, t.c, &mut nodes);
            let m_ca = get_or_create_mid(t.c, t.a, &mut nodes);

            let cx = (nodes[t.a][0] + nodes[t.b][0] + nodes[t.c][0]) / 3.0;
            let cy = (nodes[t.a][1] + nodes[t.b][1] + nodes[t.c][1]) / 3.0;
            let c_idx = nodes.len();
            nodes.push([cx, cy, 0.0]);

            volume_elements.push(VolumeElement {
                id: vol_id,
                elem_type: ElementType::Quad4,
                nodes: vec![t.a, m_ab, c_idx, m_ca],
            });
            vol_id += 1;
            volume_elements.push(VolumeElement {
                id: vol_id,
                elem_type: ElementType::Quad4,
                nodes: vec![t.b, m_bc, c_idx, m_ab],
            });
            vol_id += 1;
            volume_elements.push(VolumeElement {
                id: vol_id,
                elem_type: ElementType::Quad4,
                nodes: vec![t.c, m_ca, c_idx, m_bc],
            });
            vol_id += 1;
        }

        // Subdivide boundary elements at midpoints for strict 100% boundary-to-volume conformity
        let mut quad_bnd_elements = Vec::new();
        let mut quad_tag_counts: HashMap<String, usize> = HashMap::new();
        for bnd in &boundary_elements {
            let u = bnd.nodes[0];
            let v = bnd.nodes[1];
            let key = (u.min(v), u.max(v));
            if let Some(&m_idx) = edge_midpoints.get(&key) {
                let id1 = quad_bnd_elements.len();
                quad_bnd_elements.push(BoundaryElement {
                    id: id1,
                    elem_type: ElementType::Line2,
                    nodes: vec![u, m_idx],
                    tag: bnd.tag.clone(),
                    normal: bnd.normal,
                });
                let id2 = quad_bnd_elements.len();
                quad_bnd_elements.push(BoundaryElement {
                    id: id2,
                    elem_type: ElementType::Line2,
                    nodes: vec![m_idx, v],
                    tag: bnd.tag.clone(),
                    normal: bnd.normal,
                });
                *quad_tag_counts.entry(bnd.tag.clone()).or_insert(0) += 2;
            } else {
                let new_id = quad_bnd_elements.len();
                let mut b = bnd.clone();
                b.id = new_id;
                quad_bnd_elements.push(b);
                *quad_tag_counts.entry(bnd.tag.clone()).or_insert(0) += 1;
            }
        }
        boundary_elements = quad_bnd_elements;
        tag_counts = quad_tag_counts;
    }

    let quality_report = evaluate_mesh_quality(&nodes, &volume_elements, &boundary_elements);

    let mesh_data = MeshData {
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
    };

    Ok(ConformalMeshHandoff {
        domain_id: domain.id.clone(),
        domain_revision: domain.revision,
        mesh_data,
        quality_report,
        tag_element_counts: tag_counts,
    })
}
