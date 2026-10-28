use super::types::{LocalMaterialProperties, ReferenceMaterial};

/// Open-Closed Principle Interface for Spatial Inhomogeneity Fields.
/// New constitutive models, damage fields, or analytical profiles can be added
/// as new implementations of this trait without modifying solver core routines.
pub trait MaterialFieldStrategy: Send + Sync {
    /// Evaluates the complete local material state at coordinate point x = [x, y, z].
    fn evaluate(&self, point: [f64; 3], reference: &ReferenceMaterial) -> LocalMaterialProperties;
}
