use primordium_lib::model::history::Legend;
use primordium_lib::model::infra::lineage_tree::AncestryTree;
use primordium_lib::model::state::entity::Entity;
use uuid::Uuid;

#[test]
fn test_ancestry_tree_linking() {
    let p_id = Uuid::new_v4();
    let c_id = Uuid::new_v4();

    // 1. Create a parent (Legend)
    let parent = Legend {
        id: p_id,
        parent_id: None,
        lineage_id: p_id,
        birth_tick: 0,
        death_tick: 100,
        lifespan: 100,
        generation: 1,
        offspring_count: 1,
        peak_energy: 100.0,
        birth_timestamp: "".to_string(),
        death_timestamp: "".to_string(),
        genotype: primordium_lib::model::state::entity::Genotype::new_random(),
        color_rgb: (255, 0, 0),
    };

    // 2. Create a child (Living Entity)
    let mut child = Entity::new(10.0, 10.0, 100);
    child.id = c_id;
    child.parent_id = Some(p_id);
    child.metabolism.generation = 2;
    child.metabolism.lineage_id = p_id;

    // 3. Build tree
    let tree = AncestryTree::build(&[parent], &[child]);

    // 4. Verify graph structure
    assert_eq!(tree.graph.node_count(), 2);
    assert_eq!(tree.graph.edge_count(), 1);

    let dot = tree.to_dot();
    assert!(dot.contains(&p_id.to_string()));
    assert!(dot.contains(&c_id.to_string()));
}
