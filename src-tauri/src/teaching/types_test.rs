use super::*;

#[test]
fn test_types_creation() {
    let node = Node2D {
        id: 0,
        x: 0.0,
        y: 0.0,
        label: "A".to_string(),
    };
    assert_eq!(node.id, 0);
    assert_eq!(node.label, "A");

    let support = Support2D {
        node_id: 0,
        condition: SupportCondition::Pinned,
    };
    assert_eq!(support.condition, SupportCondition::Pinned);
}
