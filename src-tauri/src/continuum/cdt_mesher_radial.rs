use std::collections::HashMap;

use super::types::*;
use super::cdt_mesher::finalize_conformal_handoff;
use crate::frame::types::CanonicalError;
use crate::mesh::types::{BoundaryElement, ElementType, VolumeElement};

pub fn ray_from_point_to_box(
    px: f64,
    py: f64,
    dx: f64,
    dy: f64,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> (f64, f64) {
    let tx = if dx > 1e-9 {
        (max_x - px) / dx
    } else if dx < -1e-9 {
        (min_x - px) / dx
    } else {
        f64::MAX
    };
    let ty = if dy > 1e-9 {
        (max_y - py) / dy
    } else if dy < -1e-9 {
        (min_y - py) / dy
    } else {
        f64::MAX
    };
    let t = tx.min(ty).max(0.0);
    (px + t * dx, py + t * dy)
}

/// Generates graded radial conformal mesh for plates with circular cutout or inclusion.
pub fn generate_radial_conformal_mesh(
    domain: &ContinuumDomain,
    is_cutout: bool,
) -> Result<ConformalMeshHandoff, CanonicalError> {
    let (center_x, center_y, radius, ang_divs, tag, feat_id, h_override) = if is_cutout {
        let c = &domain.cutouts[0];
        match c.shape {
            CutoutShape::Circle { center_x, center_y, radius, angular_divisions } => {
                (center_x, center_y, radius, angular_divisions, c.tag.clone(), c.id.clone(), c.local_element_size)
            }
            _ => unreachable!(),
        }
    } else {
        let inc = &domain.inclusions[0];
        match inc.shape {
            CutoutShape::Circle { center_x, center_y, radius, angular_divisions } => {
                (center_x, center_y, radius, angular_divisions, "Inclusion".to_string(), inc.id.clone(), inc.local_element_size)
            }
            _ => unreachable!(),
        }
    };

    let prof = &domain.meshing_profile;
    let h = prof.target_element_size.max(0.05);
    let alpha = prof.hole_refinement_factor.clamp(0.1, 1.0);
    let h_hole = h_override.unwrap_or(h * alpha).max(0.01);

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
    let inclusion_vol_count = volume_elements.len();

    // Concentric outer rings from r = radius to outer boundary
    for ring in 0..=nr {
        let s = ring as f64 / nr as f64;
        let s_graded = s.powf(1.15);
        for th in 0..n_theta {
            let angle = (th as f64 / n_theta as f64) * 2.0 * std::f64::consts::PI;
            let cos_th = angle.cos();
            let sin_th = angle.sin();

            let x_inner = center_x + radius * cos_th;
            let y_inner = center_y + radius * sin_th;

            let (x_outer, y_outer) = ray_from_point_to_box(
                center_x, center_y, cos_th, sin_th, min_x, max_x, min_y, max_y,
            );

            let x = (1.0 - s_graded) * x_inner + s_graded * x_outer;
            let y = (1.0 - s_graded) * y_inner + s_graded * y_outer;

            if ring == 0 && !is_cutout {
                // Shared with inclusion outer perimeter
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
    let mut boundary_provenance: Vec<ElementProvenance> = Vec::new();

    // 1. Hole Boundary (only if is_cutout)
    if is_cutout {
        for th in 0..n_theta {
            let n0 = get_matrix_node(0, th);
            let n1 = get_matrix_node(0, th + 1);
            let mid_angle = ((th as f64 + 0.5) / n_theta as f64) * 2.0 * std::f64::consts::PI;
            let normal = [-mid_angle.cos(), -mid_angle.sin(), 0.0];

            boundary_elements.push(BoundaryElement {
                id: bnd_id,
                elem_type: ElementType::Line2,
                nodes: vec![n0, n1],
                tag: tag.clone(),
                normal,
            });
            boundary_provenance.push(ElementProvenance {
                element_id: bnd_id,
                parent_feature_id: feat_id.clone(),
                tag: tag.clone(),
            });
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            bnd_id += 1;
        }
    }

    // 2. Outer boundary elements (ring = nr)
    for th in 0..n_theta {
        let n0 = get_matrix_node(nr, th);
        let n1 = get_matrix_node(nr, th + 1);

        let (x_mid_bnd, y_mid_bnd) = {
            let p0 = nodes[n0];
            let p1 = nodes[n1];
            (0.5 * (p0[0] + p1[0]), 0.5 * (p0[1] + p1[1]))
        };

        let (seg_tag, default_seg_id, normal) = if (x_mid_bnd - max_x).abs() < 1e-3 {
            ("Right", "S2", [1.0, 0.0, 0.0])
        } else if (y_mid_bnd - max_y).abs() < 1e-3 {
            ("Top", "S3", [0.0, 1.0, 0.0])
        } else if (x_mid_bnd - min_x).abs() < 1e-3 {
            ("Left", "S4", [-1.0, 0.0, 0.0])
        } else {
            ("Bottom", "S1", [0.0, -1.0, 0.0])
        };

        let (final_tag, source_seg_id) = domain.outer_boundary.segments.iter()
            .find(|s| s.tag.eq_ignore_ascii_case(seg_tag))
            .map(|s| (s.tag.clone(), s.id.clone()))
            .unwrap_or_else(|| (seg_tag.to_string(), default_seg_id.to_string()));

        boundary_elements.push(BoundaryElement {
            id: bnd_id,
            elem_type: ElementType::Line2,
            nodes: vec![n0, n1],
            tag: final_tag.clone(),
            normal,
        });
        boundary_provenance.push(ElementProvenance {
            element_id: bnd_id,
            parent_feature_id: source_seg_id,
            tag: final_tag.clone(),
        });
        *tag_counts.entry(final_tag).or_insert(0) += 1;
        bnd_id += 1;
    }

    let volume_provenance: Vec<VolumeProvenance> = volume_elements
        .iter()
        .enumerate()
        .map(|(idx, v)| VolumeProvenance {
            element_id: v.id,
            subdomain_id: if !is_cutout && idx < inclusion_vol_count {
                feat_id.clone()
            } else {
                "matrix".to_string()
            },
        })
        .collect();

    Ok(finalize_conformal_handoff(
        domain,
        nodes,
        boundary_elements,
        volume_elements,
        tag_counts,
        boundary_provenance,
        volume_provenance,
    ))
}
