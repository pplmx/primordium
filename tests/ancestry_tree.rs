mod common;
use common::EntityBuilder;
use primordium_core::lineage_tree::AncestryTree;
use primordium_data::Legend;
use primordium_lib::model::brain;
use uuid::Uuid;

#[tokio::test]
async fn test_ancestry_tree_linking() {
    let p_id = Uuid::new_v4();
    let c_id = Uuid::new_v4();

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
        genotype: brain::create_genotype_random_with_rng(&mut rand::thread_rng()),
        color_rgb: (255, 0, 0),
    };

    let mut child = EntityBuilder::new().at(10.0, 10.0).lineage(p_id).build();
    child.identity.id = c_id;
    child.identity.parent_id = Some(p_id);
    child.metabolism.generation = 2;

    let tree = AncestryTree::build(&[parent], &[child]);

    assert_eq!(tree.graph.node_count(), 2);
    assert_eq!(tree.graph.edge_count(), 1);

    let dot = tree.to_dot();
    assert!(dot.contains(&p_id.to_string()));
    assert!(dot.contains(&c_id.to_string()));
}
