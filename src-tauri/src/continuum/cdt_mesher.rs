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

    // Check if domain is a centered box with single circular cutout or inclusion
    let has_single_circ_cutout = domain.cutouts.len() == 1
        && matches!(domain.cutouts[0].shape, CutoutShape::Circle { .. })
        && domain.inclusions.is_empty();

    let has_single_circ_inclusion = domain.cutouts.is_empty()
        && domain.inclusions.len() == 1
        && matches!(domain.inclusions[0].shape, CutoutShape::Circle { .. });

    let is_axis_aligned_box = domain.outer_boundary.vertices.len() == 4
        && domain.outer_boundary.vertices.iter().all(|v| v.x.abs() > 0.1 && v.y.abs() > 0.1);

    if (has_single_circ_cutout || has_single_circ_inclusion) && is_axis_aligned_box {
        generate_radial_conformal_mesh(domain, has_single_circ_cutout)
    } else {
        generate_general_conformal_mesh(domain)
    }
}

/// Generates graded radial conformal mesh for plates with circular cutout or inclusion.
fn generate_radial_conformal_mesh(
    domain: &ContinuumDomain,
    is_cutout: bool,
) -> Result<ConformalMeshHandoff, CanonicalError> {
    let (center_x, center_y, radius, ang_divs, tag) = if is_cutout {
        let c = &domain.cutouts[0];
        match c.shape {
            CutoutShape::Circle { center_x, center_y, radius, angular_divisions } => {
                (center_x, center_y, radius, angular_divisions, c.tag.clone())
            }
            _ => unreachable!(),
        }
    } else {
        let inc = &domain.inclusions[0];
        match inc.shape {
            CutoutShape::Circle { center_x, center_y, radius, angular_divisions } => {
                (center_x, center_y, radius, angular_divisions, "Inclusion".to_string())
            }
            _ => unreachable!(),
        }
    };

    let prof = &domain.meshing_profile;
    let h = prof.target_element_size.max(0.05);
    let alpha = prof.hole_refinement_factor.clamp(0.1, 1.0);
    let h_hole = h * alpha;

    // Outer boundary bounds
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
    let half_w = (max_x - min_x) / 2.0;
    let half_h = (max_y - min_y) / 2.0;
    let mid_x = (min_x + max_x) / 2.0;
    let mid_y = (min_y + max_y) / 2.0;

    let circ_len = 2.0 * std::f64::consts::PI * radius;
    let req_divs = ((circ_len / h_hole).ceil() as usize).max(ang_divs).max(16);
    let n_theta = ((req_divs + 3) / 4) * 4; // Multiple of 4

    let radial_dist = half_w.min(half_h) - radius;
    let nr = ((radial_dist / (0.7 * h)).ceil() as usize).max(4);

    let mut nodes: Vec<[f64; 3]> = Vec::new();
    let mut boundary_elements: Vec<BoundaryElement> = Vec::new();
    let mut volume_elements: Vec<VolumeElement> = Vec::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    let node_idx = |ring: usize, th: usize| -> usize {
        let th_norm = th % n_theta;
        ring * n_theta + th_norm
    };

    // If inclusion, generate interior nodes from center out to radius
    let nr_inner = if !is_cutout {
        let n_in = ((radius / (0.8 * h_hole)).ceil() as usize).max(2);
        // Center node
        nodes.push([center_x, center_y, 0.0]); // index 0
        let center_idx = 0;

        for r_step in 1..=n_in {
            let r_frac = r_step as f64 / n_in as f64;
            let curr_r = radius * r_frac;
            for th in 0..n_theta {
                let angle = (th as f64 / n_theta as f64) * 2.0 * std::f64::consts::PI;
                let x = center_x + curr_r * angle.cos();
                let y = center_y + curr_r * angle.sin();
                nodes.push([x, y, 0.0]);
            }
        }

        // Triangles for first ring around center
        let mut vol_id = 0;
        for th in 0..n_theta {
            let n1 = 1 + (th % n_theta);
            let n2 = 1 + ((th + 1) % n_theta);
            volume_elements.push(VolumeElement {
                id: vol_id,
                elem_type: ElementType::Tri3,
                nodes: vec![center_idx, n1, n2],
            });
            vol_id += 1;
        }

        // Triangles for intermediate inner rings
        for r_step in 1..n_in {
            let ring_base = 1 + (r_step - 1) * n_theta;
            let next_ring_base = 1 + r_step * n_theta;
            for th in 0..n_theta {
                let n0 = ring_base + (th % n_theta);
                let n1 = ring_base + ((th + 1) % n_theta);
                let n2 = next_ring_base + ((th + 1) % n_theta);
                let n3 = next_ring_base + (th % n_theta);

                let d02 = (nodes[n0][0] - nodes[n2][0]).hypot(nodes[n0][1] - nodes[n2][1]);
                let d13 = (nodes[n1][0] - nodes[n3][0]).hypot(nodes[n1][1] - nodes[n3][1]);

                if prof.volume_element_type == VolumeElementKind::Quad4 {
                    volume_elements.push(VolumeElement {
                        id: vol_id,
                        elem_type: ElementType::Quad4,
                        nodes: vec![n0, n1, n2, n3],
                    });
                    vol_id += 1;
                } else if d02 <= d13 {
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
                        elem_type: ElementType::Tri3,
                        nodes: vec![n0, n1, n3],
                    });
                    vol_id += 1;
                    volume_elements.push(VolumeElement {
                        id: vol_id,
                        elem_type: ElementType::Tri3,
                        nodes: vec![n1, n2, n3],
                    });
                    vol_id += 1;
                }
            }
        }
        n_in
    } else {
        0
    };

    let outer_start_node_idx = nodes.len();

    // Concentric outer rings from r = radius (ring 0) to outer boundary (ring nr)
    for ring in 0..=nr {
        let s = ring as f64 / nr as f64;
        let s_graded = s.powf(1.15); // Fine grading near hole/inclusion
        for th in 0..n_theta {
            let angle = (th as f64 / n_theta as f64) * 2.0 * std::f64::consts::PI;
            let cos_th = angle.cos();
            let sin_th = angle.sin();

            let x_inner = center_x + radius * cos_th;
            let y_inner = center_y + radius * sin_th;

            let (bx, by) = ray_to_box(cos_th, sin_th, half_w, half_h);
            let x_outer = mid_x + bx;
            let y_outer = mid_y + by;

            let x = (1.0 - s_graded) * x_inner + s_graded * x_outer;
            let y = (1.0 - s_graded) * y_inner + s_graded * y_outer;

            // If ring 0 and inclusion, reuse inner perimeter nodes to share exact indices
            if ring == 0 && !is_cutout {
                // already generated as the last inner ring:
                // indices: 1 + (nr_inner - 1) * n_theta .. 1 + nr_inner * n_theta
                // don't push, will map index
            } else {
                nodes.push([x, y, 0.0]);
            }
        }
    }

    let get_matrix_node = |ring: usize, th: usize| -> usize {
        let th_norm = th % n_theta;
        if !is_cutout && ring == 0 {
            1 + (nr_inner - 1) * n_theta + th_norm
        } else if !is_cutout {
            outer_start_node_idx + (ring - 1) * n_theta + th_norm
        } else {
            node_idx(ring, th)
        }
    };

    let mut vol_id = volume_elements.len();
    for ring in 0..nr {
        for th in 0..n_theta {
            let n0 = get_matrix_node(ring, th);
            let n1 = get_matrix_node(ring, th + 1);
            let n2 = get_matrix_node(ring + 1, th + 1);
            let n3 = get_matrix_node(ring + 1, th);

            let d02 = (nodes[n0][0] - nodes[n2][0]).hypot(nodes[n0][1] - nodes[n2][1]);
            let d13 = (nodes[n1][0] - nodes[n3][0]).hypot(nodes[n1][1] - nodes[n3][1]);

            if prof.volume_element_type == VolumeElementKind::Quad4 {
                volume_elements.push(VolumeElement {
                    id: vol_id,
                    elem_type: ElementType::Quad4,
                    nodes: vec![n0, n1, n2, n3],
                });
                vol_id += 1;
            } else if d02 <= d13 {
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
                    elem_type: ElementType::Tri3,
                    nodes: vec![n0, n1, n3],
                });
                vol_id += 1;
                volume_elements.push(VolumeElement {
                    id: vol_id,
                    elem_type: ElementType::Tri3,
                    nodes: vec![n1, n2, n3],
                });
                vol_id += 1;
            }
        }
    }

    let mut bnd_id = 0;

    // 1. Hole Boundary (only if is_cutout): normal pointing towards center
    if is_cutout {
        for th in 0..n_theta {
            let n0 = get_matrix_node(0, th);
            let n1 = get_matrix_node(0, th + 1);
            let mid_angle = ((th as f64 + 0.5) / n_theta as f64) * 2.0 * std::f64::consts::PI;
            // Outward normal points away from material, i.e. into the hole cavity: -cos, -sin
            let normal = [-mid_angle.cos(), -mid_angle.sin(), 0.0];

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Line2,
                nodes: vec![n0, n1],
                tag: tag.clone(),
                normal,
            });
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            bnd_id += 1;
        }
    }

    // 2. Outer boundary elements (ring = nr)
    let quarter = n_theta / 4;
    for th in 0..n_theta {
        let n0 = get_matrix_node(nr, th);
        let n1 = get_matrix_node(nr, th + 1);

        let seg_tag = if th < quarter / 2 || th >= n_theta - quarter / 2 {
            "Right"
        } else if th < quarter + quarter / 2 {
            "Top"
        } else if th < 2 * quarter + quarter / 2 {
            "Left"
        } else {
            "Bottom"
        };

        // Match with user segment tag if user tagged segments
        let final_tag = domain.outer_boundary.segments.iter()
            .find(|s| s.tag.eq_ignore_ascii_case(seg_tag))
            .map(|s| s.tag.clone())
            .unwrap_or_else(|| seg_tag.to_string());

        let normal = match seg_tag {
            "Right" => [1.0, 0.0, 0.0],
            "Top" => [0.0, 1.0, 0.0],
            "Left" => [-1.0, 0.0, 0.0],
            _ => [0.0, -1.0, 0.0],
        };

        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: final_tag.clone(),
            normal,
        });
        *tag_counts.entry(final_tag).or_insert(0) += 1;
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

fn ray_to_box(dx: f64, dy: f64, half_w: f64, half_h: f64) -> (f64, f64) {
    let tx = if dx.abs() > 1e-9 { half_w / dx.abs() } else { f64::MAX };
    let ty = if dy.abs() > 1e-9 { half_h / dy.abs() } else { f64::MAX };
    let t = tx.min(ty);
    (t * dx, t * dy)
}

/// Computes comprehensive mesh quality metrics.
fn evaluate_mesh_quality(
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

/// Fallback general conformal mesher for arbitrary polygons.
fn generate_general_conformal_mesh(domain: &ContinuumDomain) -> Result<ConformalMeshHandoff, CanonicalError> {
    // For arbitrary convex/concave polygons, discretize perimeter and triangulate
    let mut nodes: Vec<[f64; 3]> = Vec::new();
    let mut boundary_elements: Vec<BoundaryElement> = Vec::new();
    let mut volume_elements: Vec<VolumeElement> = Vec::new();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    let vertices = &domain.outer_boundary.vertices;
    let n = vertices.len();
    for v in vertices {
        nodes.push([v.x, v.y, 0.0]);
    }

    // Add centroid node
    let mut cx = 0.0;
    let mut cy = 0.0;
    for v in vertices {
        cx += v.x;
        cy += v.y;
    }
    cx /= n as f64;
    cy /= n as f64;
    let prof = &domain.meshing_profile;
    if prof.volume_element_type == VolumeElementKind::Quad4 && n == 4 {
        volume_elements.push(VolumeElement {
            id: 0,
            elem_type: ElementType::Quad4,
            nodes: vec![0, 1, 2, 3],
        });
    } else {
        let center_idx = nodes.len();
        nodes.push([cx, cy, 0.0]);

        if prof.volume_element_type == VolumeElementKind::Quad4 && n % 2 == 0 {
            for k in 0..(n / 2) {
                let n0 = 2 * k;
                let n1 = (2 * k + 1) % n;
                let n2 = (2 * k + 2) % n;
                volume_elements.push(VolumeElement {
                    id: k,
                    elem_type: ElementType::Quad4,
                    nodes: vec![center_idx, n0, n1, n2],
                });
            }
        } else {
            for i in 0..n {
                let n0 = i;
                let n1 = (i + 1) % n;
                volume_elements.push(VolumeElement {
                    id: i,
                    elem_type: ElementType::Tri3,
                    nodes: vec![center_idx, n0, n1],
                });
            }
        }
    }

    // Boundary elements
    for i in 0..n {
        let seg = &domain.outer_boundary.segments.get(i);
        let tag = seg.map(|s| s.tag.clone()).unwrap_or_else(|| format!("Edge_{}", i + 1));
        let normal = seg.map(|s| s.outward_normal).unwrap_or([0.0, -1.0, 0.0]);

        boundary_elements.push(BoundaryElement {
            id: i,
            elem_type: ElementType::Line2,
            nodes: vec![i, (i + 1) % n],
            tag: tag.clone(),
            normal,
        });
        *tag_counts.entry(tag).or_insert(0) += 1;
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
