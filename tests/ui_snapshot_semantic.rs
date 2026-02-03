mod common;
use common::{EntityBuilder, WorldBuilder};
use primordium_lib::app::state::App;
use ratatui::{backend::TestBackend, Terminal};

#[tokio::test]
async fn test_main_ui_render_semantic() {
    let (world, env) = WorldBuilder::new()
        .with_entity(EntityBuilder::new().at(10.0, 10.0).build())
        .with_entity(EntityBuilder::new().at(20.0, 20.0).build())
        .build();

    let snapshot = world.create_snapshot(None);
    let mut app = App::new().expect("Failed to create app");
    app.latest_snapshot = Some(snapshot);
    app.env = env;

    // 1. Setup Ratatui Test Backend
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    // 2. Render
    terminal
        .draw(|f| {
            app.draw(f);
        })
        .expect("Failed to render UI");

    // 3. Semantic Assertions on Buffer
    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer);

    assert!(
        content.contains("Pop: 2"),
        "Population count missing or incorrect"
    );
    assert!(
        content.contains("Tick: 0"),
        "Tick count missing or incorrect"
    );

    // Check if legends/rank grid headers are present (in the normal sidebar)
    // By default sidebar is hidden unless show_brain or show_ancestry is true.
    // Let's toggle show_brain to show the sidebar with Hall of Fame
    app.show_brain = true;
    terminal.draw(|f| app.draw(f)).unwrap();
    let content2 = format!("{:?}", terminal.backend().buffer());
    assert!(
        content2.contains("Hall of Fame"),
        "Hall of Fame section missing in sidebar"
    );
}

#[tokio::test]
async fn test_help_overlay_render() {
    let (world, env) = WorldBuilder::new().build();
    let snapshot = world.create_snapshot(None);
    let mut app = App::new().expect("Failed to create app");
    app.latest_snapshot = Some(snapshot);
    app.env = env;
    app.show_help = true; // Toggle help overlay

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            app.draw(f);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer);

    assert!(content.contains("CONTROLS"), "Help overlay title missing");
}
