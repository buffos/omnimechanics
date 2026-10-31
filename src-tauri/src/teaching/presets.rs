//! Educational Presets for Structural Mechanics Studio.

use super::types::*;
use super::isostatic_beam::*;
use super::truss::*;
use super::cross_kani::*;

pub fn solve_preset(config: &TeachingInputConfig) -> Result<TeachingSolutionDto, String> {
    let lang = config.language.as_deref().unwrap_or("en");
    match config.preset_name.as_str() {
        "simply_supported_distributed" => {
            let len = config.length.unwrap_or(6.0);
            let q = config.load_q.unwrap_or(12.0);
            let supports = vec![
                (0.0, SupportCondition::Pinned),
                (len, SupportCondition::RollerX),
            ];
            let loads = BeamLoadSpec {
                point_loads: vec![],
                dist_loads: vec![(0.0, len, q)],
                point_moments: vec![],
            };
            solve_isostatic_beam_system(len, &supports, &loads, lang)
        }
        "simply_supported_point" => {
            let len = config.length.unwrap_or(8.0);
            let p = config.load_p.unwrap_or(40.0);
            let p_pos = config.load_p_pos.unwrap_or(len / 2.0).clamp(0.0, len);
            let supports = vec![
                (0.0, SupportCondition::Pinned),
                (len, SupportCondition::RollerX),
            ];
            let loads = BeamLoadSpec {
                point_loads: vec![(p_pos, p)],
                dist_loads: vec![],
                point_moments: vec![],
            };
            solve_isostatic_beam_system(len, &supports, &loads, lang)
        }
        "cantilever" => {
            let len = config.length.unwrap_or(4.0);
            let p = config.load_p.unwrap_or(15.0);
            let p_pos = config.load_p_pos.unwrap_or(len).clamp(0.0, len);
            let q = config.load_q.unwrap_or(5.0);
            let supports = vec![(0.0, SupportCondition::Clamped)];
            let loads = BeamLoadSpec {
                point_loads: vec![(p_pos, p)],
                dist_loads: vec![(0.0, len, q)],
                point_moments: vec![],
            };
            solve_isostatic_beam_system(len, &supports, &loads, lang)
        }
        "overhanging_beam" => {
            let l_main = config.length.unwrap_or(6.0);
            let l_overhang = config.height.unwrap_or(2.0); // Use height as overhang length
            let total_len = l_main + l_overhang;
            let q = config.load_q.unwrap_or(10.0);
            let p = config.load_p.unwrap_or(20.0);
            let p_pos = config.load_p_pos.unwrap_or(total_len).clamp(0.0, total_len);
            let supports = vec![
                (0.0, SupportCondition::Pinned),
                (l_main, SupportCondition::RollerX),
            ];
            let loads = BeamLoadSpec {
                point_loads: vec![(p_pos, p)],
                dist_loads: vec![(0.0, l_main, q)],
                point_moments: vec![],
            };
            solve_isostatic_beam_system(total_len, &supports, &loads, lang)
        }
        "pratt_truss" => {
            let span = config.length.unwrap_or(12.0);
            let height = config.height.unwrap_or(3.0);
            let panel_len = span / 4.0;
            let p = config.load_p.unwrap_or(20.0);

            let mut nodes = Vec::new();
            // Bottom chord nodes 0..4
            for i in 0..=4 {
                nodes.push(Node2D {
                    id: i,
                    x: (i as f64) * panel_len,
                    y: 0.0,
                    label: format!("B{}", i),
                });
            }
            // Top chord nodes 5..7
            for i in 1..=3 {
                nodes.push(Node2D {
                    id: 4 + i,
                    x: (i as f64) * panel_len,
                    y: height,
                    label: format!("T{}", i),
                });
            }

            let mut members = Vec::new();
            let mut mid = 0;
            // Bottom chord bars
            for i in 0..4 {
                members.push(Member2D {
                    id: mid,
                    start_node: i,
                    end_node: i + 1,
                    length: panel_len,
                    ea: 1e7,
                    ei: 0.0,
                    label: format!("B{}-B{}", i, i + 1),
                });
                mid += 1;
            }
            // Top chord bars
            for i in 5..7 {
                members.push(Member2D {
                    id: mid,
                    start_node: i,
                    end_node: i + 1,
                    length: panel_len,
                    ea: 1e7,
                    ei: 0.0,
                    label: format!("T{}-T{}", i - 4, i - 3),
                });
                mid += 1;
            }
            // Verticals: 1-5, 2-6, 3-7
            for i in 1..=3 {
                members.push(Member2D {
                    id: mid,
                    start_node: i,
                    end_node: 4 + i,
                    length: height,
                    ea: 1e7,
                    ei: 0.0,
                    label: format!("V{}", i),
                });
                mid += 1;
            }
            // Diagonals: 0-5, 1-6, 3-6, 4-7 (Pratt: tension diagonals slope towards center)
            let diag_len = (panel_len * panel_len + height * height).sqrt();
            let diags = vec![(0, 5), (1, 6), (3, 6), (4, 7)];
            for (sn, en) in diags {
                members.push(Member2D {
                    id: mid,
                    start_node: sn,
                    end_node: en,
                    length: diag_len,
                    ea: 1e7,
                    ei: 0.0,
                    label: format!("D{}-{}", sn, en),
                });
                mid += 1;
            }

            let supports = vec![
                Support2D { node_id: 0, condition: SupportCondition::Pinned },
                Support2D { node_id: 4, condition: SupportCondition::RollerX },
            ];

            // Point loads at bottom joints
            let loads = vec![
                PointLoad2D { node_id: 1, fx: 0.0, fy: -p, mz: 0.0 },
                PointLoad2D { node_id: 2, fx: 0.0, fy: -p, mz: 0.0 },
                PointLoad2D { node_id: 3, fx: 0.0, fy: -p, mz: 0.0 },
            ];

            let truss = TrussSystem { nodes, members, supports, loads };
            solve_teaching_truss(&truss, lang)
        }
        "cross_two_span" => {
            let l1 = config.length.unwrap_or(6.0);
            let l2 = config.height.unwrap_or(4.0); // Use height as span 2
            let q1 = config.load_q.unwrap_or(15.0);
            let p2 = config.load_p.unwrap_or(30.0);

            let cb_config = ContinuousBeamConfig {
                l1,
                l2,
                q1,
                p2,
                left_fixed: false,
                right_fixed: false,
            };
            solve_continuous_beam_cross(&cb_config, lang)
        }
        _ => Err(format!("Unknown teaching preset: {}", config.preset_name)),
    }
}

#[cfg(test)]
#[path = "presets_test.rs"]
mod tests;
