use super::types::*;
use crate::frame::types::CanonicalError;

const EPSILON_LEN: f64 = 1e-4;
const EPSILON_AREA: f64 = 1e-6;

/// Computes the signed area of a 2D polygon using the shoelace formula.
/// Positive area indicates Counter-Clockwise (CCW) winding.
/// Negative area indicates Clockwise (CW) winding.
pub fn compute_signed_area(pts: &[(f64, f64)]) -> f64 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % n];
        area += x1 * y2 - x2 * y1;
    }
    0.5 * area
}

/// Checks intersection between segment [p1, p2] and [p3, p4].
/// Returns Some((intersection_x, intersection_y)) if segments intersect strictly or at interior.
pub fn segment_intersection(
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
    p4: (f64, f64),
    tol: f64,
) -> Option<(f64, f64)> {
    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;

    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-12 {
        return None; // parallel or collinear
    }

    let s = ((p3.0 - p1.0) * d2y - (p3.1 - p1.1) * d2x) / denom;
    let t = ((p3.0 - p1.0) * d1y - (p3.1 - p1.1) * d1x) / denom;

    if s >= -tol && s <= 1.0 + tol && t >= -tol && t <= 1.0 + tol {
        Some((p1.0 + s * d1x, p1.1 + s * d1y))
    } else {
        None
    }
}

/// Checks if point p lies on segment [a, b].
pub fn point_on_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64), tol: f64) -> bool {
    let cross = (p.1 - a.1) * (b.0 - a.0) - (p.0 - a.0) * (b.1 - a.1);
    if cross.abs() > tol {
        return false;
    }
    let dot = (p.0 - a.0) * (b.0 - a.0) + (p.1 - a.1) * (b.1 - a.1);
    if dot < -tol {
        return false;
    }
    let sq_len = (b.0 - a.0).powi(2) + (b.1 - a.1).powi(2);
    if dot > sq_len + tol {
        return false;
    }
    true
}

/// Closes and normalizes a loop:
/// - Enforces N >= 3 vertices, non-zero segment lengths, area >= 1e-6
/// - Reverses vertices to CCW if outer boundary, or CW if hole
/// - Computes outward unit normals pointing away from solid material
pub fn close_and_normalize_loop(
    loop_id: &str,
    vertices: &[Vertex2D],
    is_hole: bool,
) -> Result<BoundaryLoop, CanonicalError> {
    let n = vertices.len();
    if n < 3 {
        return Err(CanonicalError::new(
            "OPEN_BOUNDARY_LOOP",
            "Το περίγραμμα πρέπει να έχει τουλάχιστον 3 κορυφές.",
            "Boundary loop must contain at least 3 vertices.",
        ));
    }

    // Check segment lengths and collinear folds
    for i in 0..n {
        let v1 = &vertices[i];
        let v2 = &vertices[(i + 1) % n];
        let dx = v2.x - v1.x;
        let dy = v2.y - v1.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < EPSILON_LEN {
            return Err(CanonicalError::new(
                "ZERO_LENGTH_SEGMENT",
                format!("Το τμήμα {} έχει μηδενικό μήκος (< 10⁻⁴ m).", i + 1),
                format!("Segment {} has zero length (< 10⁻⁴ m).", i + 1),
            ));
        }

        // Check fold back
        let v0 = &vertices[(i + n - 1) % n];
        let prev_dx = v1.x - v0.x;
        let prev_dy = v1.y - v0.y;
        let prev_len = (prev_dx * prev_dx + prev_dy * prev_dy).sqrt();
        if prev_len >= EPSILON_LEN {
            let cos_angle = (prev_dx * dx + prev_dy * dy) / (prev_len * len);
            if cos_angle < -1.0 + 1e-7 {
                return Err(CanonicalError::new(
                    "ZERO_LENGTH_SEGMENT",
                    format!("Αντίστροφη αναδίπλωση 180° στην κορυφή {}.", v1.id),
                    format!("180-degree collinear fold at vertex {}.", v1.id),
                ));
            }
        }
    }

    let coords: Vec<(f64, f64)> = vertices.iter().map(|v| (v.x, v.y)).collect();

    // Check self-intersections first (a bowtie/figure-8 may have 0 net signed area)
    for i in 0..n {
        let p1 = coords[i];
        let p2 = coords[(i + 1) % n];
        for j in (i + 1)..n {
            // Skip adjacent segments
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            let p3 = coords[j];
            let p4 = coords[(j + 1) % n];
            if let Some(int_pt) = segment_intersection(p1, p2, p3, p4, 1e-6) {
                let details = serde_json::json!({
                    "loop_id": loop_id,
                    "segment_a": format!("S{}", i + 1),
                    "segment_b": format!("S{}", j + 1),
                    "intersection_point": [int_pt.0, int_pt.1],
                });
                return Err(CanonicalError::new(
                    "SELF_INTERSECTING_LOOP",
                    format!("Το περίγραμμα τέμνει τον εαυτό του μεταξύ των τμημάτων S{} και S{}.", i + 1, j + 1),
                    format!("Boundary loop self-intersects between segments S{} and S{}.", i + 1, j + 1),
                ).with_details(details));
            }
        }
        // Check non-incident vertex lying on segment [p1, p2]
        for k in 0..n {
            if k == i || k == (i + 1) % n {
                continue;
            }
            let vk = coords[k];
            if point_on_segment(vk, p1, p2, 1e-6) || distance_point_to_segment(vk, p1, p2) < EPSILON_LEN {
                let details = serde_json::json!({
                    "loop_id": loop_id,
                    "segment_a": format!("S{}", i + 1),
                    "vertex": vertices[k].id,
                    "intersection_point": [vk.0, vk.1],
                });
                return Err(CanonicalError::new(
                    "SELF_INTERSECTING_LOOP",
                    format!("Η κορυφή '{}' εφάπτεται στο μη γειτονικό τμήμα S{}.", vertices[k].id, i + 1),
                    format!("Vertex '{}' touches non-incident segment S{}.", vertices[k].id, i + 1),
                ).with_details(details));
            }
        }
    }

    let area = compute_signed_area(&coords);
    if area.abs() < EPSILON_AREA {
        return Err(CanonicalError::new(
            "DEGENERATE_POLYGON_AREA",
            "Το εμβαδόν του πολυγώνου είναι εκφυλισμένα μικρό (< 10⁻⁶ m²).",
            "Polygon area is degenerate (< 10⁻⁶ m²).",
        ));
    }

    // Winding orientation:
    // Outer boundary should be CCW (area > 0)
    // Hole should be CW (area < 0)
    let should_reverse = if is_hole { area > 0.0 } else { area < 0.0 };
    let normalized_vertices: Vec<Vertex2D> = if should_reverse {
        vertices.iter().rev().cloned().collect()
    } else {
        vertices.to_vec()
    };

    let norm_coords: Vec<(f64, f64)> = normalized_vertices.iter().map(|v| (v.x, v.y)).collect();
    let norm_area = compute_signed_area(&norm_coords);
    let orientation = if norm_area > 0.0 {
        LoopOrientation::CounterClockwise
    } else {
        LoopOrientation::Clockwise
    };

    // Calculate domain center for default tags
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for v in &normalized_vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
    }
    let mid_y = (min_y + max_y) / 2.0;

    let mut segments = Vec::new();
    let num_v = normalized_vertices.len();
    for i in 0..num_v {
        let v_curr = &normalized_vertices[i];
        let v_next = &normalized_vertices[(i + 1) % num_v];
        let dx = v_next.x - v_curr.x;
        let dy = v_next.y - v_curr.y;
        let len = (dx * dx + dy * dy).sqrt();

        // Outward unit normal: [dy / len, -dx / len, 0.0]
        let nx = dy / len;
        let ny = -dx / len;

        // Auto tag
        let tag = if is_hole {
            "Hole".to_string()
        } else if dy.abs() < 1e-4 {
            if ny < 0.0 || (v_curr.y < mid_y && v_next.y < mid_y) {
                "Bottom".to_string()
            } else {
                "Top".to_string()
            }
        } else if dx.abs() < 1e-4 {
            if nx < 0.0 {
                "Left".to_string()
            } else {
                "Right".to_string()
            }
        } else {
            format!("Outer_{}", i + 1)
        };

        segments.push(BoundarySegment {
            id: format!("S{}", i + 1),
            start_vertex_id: v_curr.id.clone(),
            end_vertex_id: v_next.id.clone(),
            tag,
            local_element_size: None,
            outward_normal: [nx, ny, 0.0],
        });
    }

    Ok(BoundaryLoop {
        id: loop_id.to_string(),
        orientation,
        closure_state: LoopClosureState::ClosedValid,
        vertices: normalized_vertices,
        segments,
    })
}

/// Previews a candidate segment to check if it introduces self-intersections.
pub fn preview_segment(
    existing_vertices: &[Vertex2D],
    candidate_point: &Vertex2D,
    closing_loop: bool,
) -> CandidateSegmentPreview {
    let n = existing_vertices.len();
    if n < 1 {
        return CandidateSegmentPreview {
            is_valid: true,
            intersecting_segment_indices: Vec::new(),
            snap_closure_ready: false,
        };
    }

    let p_last = (existing_vertices[n - 1].x, existing_vertices[n - 1].y);
    let p_cand = (candidate_point.x, candidate_point.y);

    let mut intersecting_indices = Vec::new();

    // Check intersection of segment (p_last -> p_cand) with all existing segments
    if n >= 2 {
        for i in 0..(n - 1) {
            let p1 = (existing_vertices[i].x, existing_vertices[i].y);
            let p2 = (existing_vertices[i + 1].x, existing_vertices[i + 1].y);
            // Skip adjacent segment (starts at p_last)
            if i + 1 == n - 1 {
                continue;
            }
            if segment_intersection(p_last, p_cand, p1, p2, 1e-6).is_some() {
                intersecting_indices.push(i);
            }
        }
    }

    if closing_loop && n >= 3 {
        let p_first = (existing_vertices[0].x, existing_vertices[0].y);
        for i in 0..(n - 1) {
            let p1 = (existing_vertices[i].x, existing_vertices[i].y);
            let p2 = (existing_vertices[i + 1].x, existing_vertices[i + 1].y);
            if i == 0 {
                continue; // Adjacent to p_first
            }
            if segment_intersection(p_cand, p_first, p1, p2, 1e-6).is_some() {
                if !intersecting_indices.contains(&i) {
                    intersecting_indices.push(i);
                }
            }
        }
    }

    let p_first = (existing_vertices[0].x, existing_vertices[0].y);
    let dist_to_start = ((p_cand.0 - p_first.0).powi(2) + (p_cand.1 - p_first.1).powi(2)).sqrt();
    let snap_closure_ready = n >= 3 && dist_to_start < 0.25;

    CandidateSegmentPreview {
        is_valid: intersecting_indices.is_empty(),
        intersecting_segment_indices: intersecting_indices,
        snap_closure_ready,
    }
}

/// Point in polygon check using ray casting.
pub fn is_point_inside_polygon(pt: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let n = poly.len();
    if n < 3 {
        return false;
    }
    for i in 0..n {
        let (x1, y1) = poly[i];
        let (x2, y2) = poly[(i + 1) % n];
        if (y1 > pt.1) != (y2 > pt.1) {
            let x_cross = (x2 - x1) * (pt.1 - y1) / (y2 - y1) + x1;
            if pt.0 < x_cross {
                inside = !inside;
            }
        }
    }
    inside
}

/// Computes minimum distance from point p to segment [a, b].
pub fn distance_point_to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let sq_len = dx * dx + dy * dy;
    if sq_len < 1e-12 {
        return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
    }
    let t = ((p.0 - a.0) * dx + (p.1 - a.1) * dy) / sq_len;
    let clamped_t = t.clamp(0.0, 1.0);
    let proj = (a.0 + clamped_t * dx, a.1 + clamped_t * dy);
    ((p.0 - proj.0).powi(2) + (p.1 - proj.1).powi(2)).sqrt()
}

/// Discretizes a CutoutShape into boundary sample points / polygon vertices.
pub fn discretize_cutout_shape(shape: &CutoutShape, default_angular_divs: usize) -> Vec<(f64, f64)> {
    match shape {
        CutoutShape::Circle { center_x, center_y, radius, angular_divisions } => {
            let divs = (*angular_divisions).max(12).max(default_angular_divs);
            let mut pts = Vec::with_capacity(divs);
            for k in 0..divs {
                // CW order: negative angle
                let angle = -2.0 * std::f64::consts::PI * (k as f64) / (divs as f64);
                pts.push((center_x + radius * angle.cos(), center_y + radius * angle.sin()));
            }
            pts
        }
        CutoutShape::Rectangle { center_x, center_y, width, height, rotation_deg } => {
            let hw = width / 2.0;
            let hh = height / 2.0;
            let rad = rotation_deg.to_radians();
            let c = rad.cos();
            let s = rad.sin();
            let local_corners = [
                (-hw, -hh),
                (-hw, hh),
                (hw, hh),
                (hw, -hh),
            ];
            local_corners
                .iter()
                .map(|(lx, ly)| (center_x + lx * c - ly * s, center_y + lx * s + ly * c))
                .collect()
        }
        CutoutShape::Polygon { vertices } => vertices.iter().map(|v| (v.x, v.y)).collect(),
    }
}

/// Validates intrinsic shape geometry of a CutoutShape (used for both cutouts and inclusions).
fn validate_feature_shape(id: &str, shape: &CutoutShape, is_hole: bool) -> Result<(), CanonicalError> {
    match shape {
        CutoutShape::Circle { radius, angular_divisions, .. } => {
            if !radius.is_finite() || *radius < 1e-3 {
                return Err(CanonicalError::new(
                    "DEGENERATE_POLYGON_AREA",
                    format!("Η ακτίνα του κυκλικού στοιχείου '{}' πρέπει να είναι >= 10⁻³ m.", id),
                    format!("Radius of circular feature '{}' must be >= 10⁻³ m.", id),
                ));
            }
            if *angular_divisions < 12 {
                return Err(CanonicalError::new(
                    "INVALID_MESHING_PROFILE",
                    format!("Ο αριθμός γωνιακών υποδιαιρέσεων του '{}' πρέπει να είναι >= 12.", id),
                    format!("Angular divisions of circular feature '{}' must be >= 12.", id),
                ));
            }
        }
        CutoutShape::Rectangle { width, height, .. } => {
            if !width.is_finite() || !height.is_finite() || *width < 1e-3 || *height < 1e-3 {
                return Err(CanonicalError::new(
                    "DEGENERATE_POLYGON_AREA",
                    format!("Οι διαστάσεις του ορθογωνίου '{}' πρέπει να είναι >= 10⁻³ m.", id),
                    format!("Dimensions of rectangular feature '{}' must be >= 10⁻³ m.", id),
                ));
            }
        }
        CutoutShape::Polygon { vertices } => {
            close_and_normalize_loop(id, vertices, is_hole)?;
        }
    }
    Ok(())
}

/// Validates an entire ContinuumDomain against topological and clearance rules.
pub fn validate_domain(domain: &ContinuumDomain) -> Result<DomainValidationResponse, CanonicalError> {
    let diagnostics = Vec::new();

    // 1. Validate Outer Boundary
    let outer_vertices = &domain.outer_boundary.vertices;
    let outer_loop = close_and_normalize_loop("L_OUTER", outer_vertices, false)?;
    for seg in &domain.outer_boundary.segments {
        if let Some(h_loc) = seg.local_element_size {
            if !h_loc.is_finite() || h_loc <= 0.0 {
                return Err(CanonicalError::new(
                    "INVALID_MESHING_PROFILE",
                    format!("Το τοπικό μέγεθος στοιχείου στο τμήμα '{}' πρέπει να είναι > 0.", seg.id),
                    format!("Local element size on segment '{}' must be > 0.", seg.id),
                ));
            }
        }
    }

    let outer_poly: Vec<(f64, f64)> = outer_loop.vertices.iter().map(|v| (v.x, v.y)).collect();
    let num_outer = outer_poly.len();

    let h = domain.meshing_profile.target_element_size.max(1e-3);
    let alpha = domain.meshing_profile.hole_refinement_factor.clamp(0.1, 1.0);
    let d_min = 1e-3_f64.max(0.1 * h * alpha);

    // Collect all features (Cutouts and Inclusions)
    struct FeatureRecord {
        id: String,
        is_cutout: bool,
        pts: Vec<(f64, f64)>,
        bounding_radius: f64,
        center: (f64, f64),
    }

    let mut feature_records = Vec::new();

    for c in &domain.cutouts {
        validate_feature_shape(&c.id, &c.shape, true)?;
        if let Some(h_loc) = c.local_element_size {
            if !h_loc.is_finite() || h_loc <= 0.0 {
                return Err(CanonicalError::new(
                    "INVALID_MESHING_PROFILE",
                    format!("Το τοπικό μέγεθος στοιχείου στην οπή '{}' πρέπει να είναι > 0.", c.id),
                    format!("Local element size on cutout '{}' must be > 0.", c.id),
                ));
            }
        }
        let pts = discretize_cutout_shape(&c.shape, 32);
        let mut cx = 0.0;
        let mut cy = 0.0;
        for p in &pts {
            cx += p.0;
            cy += p.1;
        }
        cx /= pts.len() as f64;
        cy /= pts.len() as f64;
        let r_max = pts.iter().map(|p| ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt()).fold(0.0_f64, f64::max);
        feature_records.push(FeatureRecord {
            id: c.id.clone(),
            is_cutout: true,
            pts,
            bounding_radius: r_max,
            center: (cx, cy),
        });
    }

    for inc in &domain.inclusions {
        validate_feature_shape(&inc.id, &inc.shape, false)?;
        if let Some(e_mod) = inc.youngs_modulus_override_gpa {
            if !e_mod.is_finite() || e_mod <= 0.0 {
                return Err(CanonicalError::new(
                    "INVALID_BOUNDARY_CONDITION",
                    format!("Μη έγκυρο μέτρο ελαστικότητας E στο ένθεμα '{}' (E > 0 GPa).", inc.id),
                    format!("Invalid Young's modulus override on inclusion '{}' (E > 0 GPa).", inc.id),
                ));
            }
        }
        if let Some(nu) = inc.poisson_ratio_override {
            if !nu.is_finite() || nu <= -1.0 || nu >= 0.5 {
                return Err(CanonicalError::new(
                    "INVALID_BOUNDARY_CONDITION",
                    format!("Μη έγκυρος λόγος Poisson ν στο ένθεμα '{}' (-1.0 < ν < 0.5).", inc.id),
                    format!("Invalid Poisson ratio override on inclusion '{}' (-1.0 < nu < 0.5).", inc.id),
                ));
            }
        }
        if let Some(h_loc) = inc.local_element_size {
            if !h_loc.is_finite() || h_loc <= 0.0 {
                return Err(CanonicalError::new(
                    "INVALID_MESHING_PROFILE",
                    format!("Το τοπικό μέγεθος στοιχείου στο ένθεμα '{}' πρέπει να είναι > 0.", inc.id),
                    format!("Local element size on inclusion '{}' must be > 0.", inc.id),
                ));
            }
        }
        let pts = discretize_cutout_shape(&inc.shape, 32);
        let mut cx = 0.0;
        let mut cy = 0.0;
        for p in &pts {
            cx += p.0;
            cy += p.1;
        }
        cx /= pts.len() as f64;
        cy /= pts.len() as f64;
        let r_max = pts.iter().map(|p| ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt()).fold(0.0_f64, f64::max);
        feature_records.push(FeatureRecord {
            id: inc.id.clone(),
            is_cutout: false,
            pts,
            bounding_radius: r_max,
            center: (cx, cy),
        });
    }

    // Check feature containment & clearance with outer boundary
    for feat in &feature_records {
        for pt in &feat.pts {
            if !is_point_inside_polygon(*pt, &outer_poly) {
                let details = serde_json::json!({
                    "feature_id": feat.id,
                    "violating_point": [pt.0, pt.1],
                });
                return Err(CanonicalError::new(
                    "HOLE_OUTSIDE_OUTER_BOUNDARY",
                    format!("Η οπή/ένθεμα '{}' εξέρχεται ή εφάπτεται στο εξωτερικό περίγραμμα.", feat.id),
                    format!("Hole/inclusion '{}' lies outside or touches the outer boundary.", feat.id),
                ).with_details(details));
            }

            // Check distance to outer boundary segments
            for i in 0..num_outer {
                let dist = distance_point_to_segment(*pt, outer_poly[i], outer_poly[(i + 1) % num_outer]);
                if dist < d_min - 1e-9 {
                    let details = serde_json::json!({
                        "feature_id": feat.id,
                        "actual_clearance": dist,
                        "required_clearance": d_min,
                    });
                    return Err(CanonicalError::new(
                        "CLEARANCE_BELOW_MINIMUM",
                        format!("Η απόσταση της οπής '{}' από το περίγραμμα ({:.3} m) είναι μικρότερη από το d_min ({:.3} m).", feat.id, dist, d_min),
                        format!("Clearance of hole '{}' to outer boundary ({:.3} m) is below minimum ({:.3} m).", feat.id, dist, d_min),
                    ).with_details(details));
                }
            }
        }
    }

    // Check mutual feature clearance & overlaps
    let num_feats = feature_records.len();
    for i in 0..num_feats {
        for j in (i + 1)..num_feats {
            let f1 = &feature_records[i];
            let f2 = &feature_records[j];

            let center_dist = ((f1.center.0 - f2.center.0).powi(2) + (f1.center.1 - f2.center.1).powi(2)).sqrt();
            if center_dist > f1.bounding_radius + f2.bounding_radius + d_min {
                continue; // Broad phase disjoint
            }

            // 1. Direct segment intersection check between f1 and f2
            let mut segments_intersect = false;
            for s1 in 0..f1.pts.len() {
                let a1 = f1.pts[s1];
                let b1 = f1.pts[(s1 + 1) % f1.pts.len()];
                for s2 in 0..f2.pts.len() {
                    let a2 = f2.pts[s2];
                    let b2 = f2.pts[(s2 + 1) % f2.pts.len()];
                    if segment_intersection(a1, b1, a2, b2, 1e-6).is_some() {
                        segments_intersect = true;
                        break;
                    }
                }
                if segments_intersect {
                    break;
                }
            }

            // 2. Point containment check (either vertex or center of one inside the other)
            let any_f1_in_f2 = f1.pts.iter().any(|&p| is_point_inside_polygon(p, &f2.pts)) || is_point_inside_polygon(f1.center, &f2.pts);
            let any_f2_in_f1 = f2.pts.iter().any(|&p| is_point_inside_polygon(p, &f1.pts)) || is_point_inside_polygon(f2.center, &f1.pts);

            if segments_intersect || any_f1_in_f2 || any_f2_in_f1 {
                let is_nested_in_void = (any_f1_in_f2 && f2.is_cutout) || (any_f2_in_f1 && f1.is_cutout);
                let details = serde_json::json!({
                    "feature_a": f1.id,
                    "feature_b": f2.id,
                    "nested_in_cutout": is_nested_in_void,
                });
                return Err(CanonicalError::new(
                    "HOLE_OVERLAPS_EXISTING_FEATURE",
                    if is_nested_in_void {
                        format!("Δεν επιτρέπεται η ένθεση του '{}' εντός της οπής '{}'.", f1.id, f2.id)
                    } else {
                        format!("Οι οπές/ενθέματα '{}' και '{}' επικαλύπτονται ή τέμνονται.", f1.id, f2.id)
                    },
                    if is_nested_in_void {
                        format!("Nesting feature inside cutout void between '{}' and '{}' is forbidden.", f1.id, f2.id)
                    } else {
                        format!("Features '{}' and '{}' overlap or intersect.", f1.id, f2.id)
                    },
                ).with_details(details));
            }

            // Compute minimum distance between points of f1 and segments of f2
            let mut min_feat_dist = f64::INFINITY;
            for p1 in &f1.pts {
                for k in 0..f2.pts.len() {
                    let d = distance_point_to_segment(*p1, f2.pts[k], f2.pts[(k + 1) % f2.pts.len()]);
                    min_feat_dist = min_feat_dist.min(d);
                }
            }
            for p2 in &f2.pts {
                for k in 0..f1.pts.len() {
                    let d = distance_point_to_segment(*p2, f1.pts[k], f1.pts[(k + 1) % f1.pts.len()]);
                    min_feat_dist = min_feat_dist.min(d);
                }
            }

            if min_feat_dist < 1e-4 {
                let details = serde_json::json!({
                    "feature_a": f1.id,
                    "feature_b": f2.id,
                });
                return Err(CanonicalError::new(
                    "HOLE_OVERLAPS_EXISTING_FEATURE",
                    format!("Οι οπές '{}' και '{}' τέμνονται ή εφάπτονται.", f1.id, f2.id),
                    format!("Cutouts '{}' and '{}' intersect or touch.", f1.id, f2.id),
                ).with_details(details));
            }

            if min_feat_dist < d_min - 1e-9 {
                let details = serde_json::json!({
                    "feature_a": f1.id,
                    "feature_b": f2.id,
                    "actual_clearance": min_feat_dist,
                    "required_clearance": d_min,
                });
                return Err(CanonicalError::new(
                    "CLEARANCE_BELOW_MINIMUM",
                    format!("Η απόσταση μεταξύ των οπών '{}' και '{}' ({:.3} m) είναι κάτω από το d_min ({:.3} m).", f1.id, f2.id, min_feat_dist, d_min),
                    format!("Clearance between '{}' and '{}' ({:.3} m) is below minimum ({:.3} m).", f1.id, f2.id, min_feat_dist, d_min),
                ).with_details(details));
            }
        }
    }

    // Validate MeshingProfile
    if domain.meshing_profile.target_element_size <= 0.0
        || domain.meshing_profile.hole_refinement_factor < 0.1
        || domain.meshing_profile.hole_refinement_factor > 1.0
        || domain.meshing_profile.min_angle_deg < 15.0
        || domain.meshing_profile.min_angle_deg > 32.0
    {
        return Err(CanonicalError::new(
            "INVALID_MESHING_PROFILE",
            "Μη έγκυρες παράμετροι πλέγματος (h > 0, α ∈ [0.1, 1.0], θ_min ∈ [15°, 32°]).",
            "Invalid meshing profile parameters (h > 0, alpha in [0.1, 1.0], theta_min in [15, 32]).",
        ));
    }

    // Validate boundary conditions
    for (tag, bc) in &domain.tag_conditions {
        match bc {
            BoundaryConditionSpec::DirichletDisplacement { ux_m, uy_m, .. } => {
                if !ux_m.is_finite() || !uy_m.is_finite() {
                    return Err(CanonicalError::new(
                        "INVALID_BOUNDARY_CONDITION",
                        format!("Μη πεπερασμένη μετατόπιση στην ετικέτα '{}'.", tag),
                        format!("Non-finite displacement on boundary tag '{}'.", tag),
                    ));
                }
            }
            BoundaryConditionSpec::NeumannTraction { tx_kn_m2, ty_kn_m2, .. } => {
                if !tx_kn_m2.is_finite() || !ty_kn_m2.is_finite() {
                    return Err(CanonicalError::new(
                        "INVALID_BOUNDARY_CONDITION",
                        format!("Μη πεπερασμένη τάση στην ετικέτα '{}'.", tag),
                        format!("Non-finite traction on boundary tag '{}'.", tag),
                    ));
                }
            }
            BoundaryConditionSpec::RobinImpedance { kn_kn_m3, ks_kn_m3, cn_kn_s_m3, cs_kn_s_m3 } => {
                if *kn_kn_m3 < 0.0 || *ks_kn_m3 < 0.0 || *cn_kn_s_m3 < 0.0 || *cs_kn_s_m3 < 0.0 {
                    return Err(CanonicalError::new(
                        "INVALID_BOUNDARY_CONDITION",
                        format!("Αρνητική δυσκαμψία ή απόσβεση Robin στην ετικέτα '{}'.", tag),
                        format!("Negative Robin spring stiffness or damping on boundary tag '{}'.", tag),
                    ));
                }
            }
            BoundaryConditionSpec::Free => {}
        }
    }

    Ok(DomainValidationResponse {
        is_valid_for_meshing: diagnostics.is_empty(),
        diagnostics,
    })
}
