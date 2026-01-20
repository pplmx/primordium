use clap::Parser;
use petgraph::graph::DiGraph;
use primordium_lib::model::history::{Legend, LiveEvent};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "logs/live.jsonl")]
    live_log: String,

    #[arg(short, long, default_value = "logs/legends.json")]
    legends_log: String,

    #[arg(short, long, default_value = "report.md")]
    output: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Analyzing Primordium Evolutionary History...");

    // 1. Build Family Tree
    let mut graph = DiGraph::<Uuid, ()>::new();
    let mut nodes = HashMap::new();

    let live_file = File::open(&args.live_log)?;
    let reader = BufReader::new(live_file);

    let mut birth_count = 0;
    let mut death_count = 0;
    let mut total_age = 0;
    let mut max_gen = 0;

    for line in reader.lines() {
        let l = line?;
        if let Ok(event) = serde_json::from_str::<LiveEvent>(&l) {
            match event {
                LiveEvent::Birth {
                    id, parent_id, gen, ..
                } => {
                    birth_count += 1;
                    max_gen = max_gen.max(gen);

                    let node = *nodes.entry(id).or_insert_with(|| graph.add_node(id));
                    if let Some(pid) = parent_id {
                        let p_node = *nodes.entry(pid).or_insert_with(|| graph.add_node(pid));
                        graph.add_edge(p_node, node, ());
                    }
                }
                LiveEvent::Death { age, .. } => {
                    death_count += 1;
                    total_age += age;
                }
                _ => {}
            }
        }
    }

    // 2. Load Legends
    let mut legends = Vec::new();
    if let Ok(legends_file) = File::open(&args.legends_log) {
        let reader = BufReader::new(legends_file);
        for line in reader.lines() {
            if let Ok(l) = line {
                if let Ok(legend) = serde_json::from_str::<Legend>(&l) {
                    legends.push(legend);
                }
            }
        }
    }

    // 3. Generate Report
    let avg_lifespan = if death_count > 0 {
        total_age as f64 / death_count as f64
    } else {
        0.0
    };

    let report = format!(
        "# Primordium Evolution Report\n\n\
        ## Summary\n\
        - **Total Births**: {}\n\
        - **Total Deaths**: {}\n\
        - **Average Lifespan**: {:.2} ticks\n\
        - **Max Generation**: {}\n\n\
        ## Legendary Organisms ({})\n\
        {}\n",
        birth_count,
        death_count,
        avg_lifespan,
        max_gen,
        legends.len(),
        legends
            .iter()
            .enumerate()
            .map(|(i, l)| {
                format!(
                    "{}. **{}** - Gen: {}, Lifespan: {}, Offspring: {}\n",
                    i + 1,
                    l.id.to_string()[..8].to_string(),
                    l.generation,
                    l.lifespan,
                    l.offspring_count
                )
            })
            .collect::<Vec<_>>()
            .join("")
    );

    std::fs::write(&args.output, report)?;
    println!("Report generated: {}", args.output);

    Ok(())
}
