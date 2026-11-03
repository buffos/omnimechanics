use super::types::{Envelope, EnvelopeStation, MemberEnvelope, ResultSet};

pub fn compute_envelopes(revision: u64, result_sets: &[ResultSet]) -> Option<Envelope> {
    if result_sets.is_empty() {
        return None;
    }

    // Map each element to its stations across combinations
    let first_rs = &result_sets[0];
    let mut member_envelopes = Vec::new();

    for mr in &first_rs.member_results {
        let el_id = &mr.element_id;
        let num_stations = mr.stations.len();

        let mut env_stations = Vec::with_capacity(num_stations);
        for st_idx in 0..num_stations {
            let x = mr.stations[st_idx].x_m;
            let mut max_m = -f64::INFINITY;
            let mut gov_max_m = String::new();
            let mut min_m = f64::INFINITY;
            let mut gov_min_m = String::new();

            let mut max_q = -f64::INFINITY;
            let mut gov_max_q = String::new();
            let mut min_q = f64::INFINITY;
            let mut gov_min_q = String::new();

            let mut max_n = -f64::INFINITY;
            let mut gov_max_n = String::new();
            let mut min_n = f64::INFINITY;
            let mut gov_min_n = String::new();

            for rs in result_sets {
                if let Some(target_mr) = rs.member_results.iter().find(|m| m.element_id == *el_id) {
                    if st_idx < target_mr.stations.len() {
                        let st = &target_mr.stations[st_idx];

                        if st.m_knm > max_m {
                            max_m = st.m_knm;
                            gov_max_m = rs.target_id.clone();
                        }
                        if st.m_knm < min_m {
                            min_m = st.m_knm;
                            gov_min_m = rs.target_id.clone();
                        }

                        if st.q_knm_or_q(st.q_kn) > max_q {
                            max_q = st.q_kn;
                            gov_max_q = rs.target_id.clone();
                        }
                        if st.q_kn < min_q {
                            min_q = st.q_kn;
                            gov_min_q = rs.target_id.clone();
                        }

                        if st.n_kn > max_n {
                            max_n = st.n_kn;
                            gov_max_n = rs.target_id.clone();
                        }
                        if st.n_kn < min_n {
                            min_n = st.n_kn;
                            gov_min_n = rs.target_id.clone();
                        }
                    }
                }
            }

            env_stations.push(EnvelopeStation {
                x_m: x,
                max_m,
                governing_max_m: gov_max_m,
                min_m,
                governing_min_m: gov_min_m,
                max_q,
                governing_max_q: gov_max_q,
                min_q,
                governing_min_q: gov_min_q,
                max_n,
                governing_max_n: gov_max_n,
                min_n,
                governing_min_n: gov_min_n,
            });
        }

        member_envelopes.push(MemberEnvelope {
            element_id: el_id.clone(),
            length_eff_m: mr.length_eff_m,
            stations: env_stations,
        });
    }

    Some(Envelope {
        revision,
        combinations_count: result_sets.len(),
        member_envelopes,
    })
}

trait HelperQ {
    fn q_knm_or_q(&self, val: f64) -> f64;
}

impl HelperQ for super::types::StationResult {
    fn q_knm_or_q(&self, val: f64) -> f64 {
        val
    }
}
