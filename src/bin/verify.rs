use clap::Parser;
use primordium_lib::model::blockchain::AnchorRecord;
use primordium_lib::model::history::{HistoryLogger, Legend};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "logs/legends.json")]
    input: String,

    #[arg(short, long, default_value = "logs/anchors.jsonl")]
    anchors: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Verifying Primordium Evolutionary History...");

    // 1. Read all legends and compute current hash
    let file = File::open(&args.input)?;
    let reader = BufReader::new(file);
    let mut legends = Vec::new();
    for l in reader.lines().map_while(Result::ok) {
        if let Ok(legend) = serde_json::from_str::<Legend>(&l) {
            legends.push(legend);
        }
    }

    if legends.is_empty() {
        println!("No legends found in {}. Nothing to verify.", args.input);
        return Ok(());
    }

    let current_hash = HistoryLogger::compute_legends_hash(&legends)?;
    println!("Current Legends Hash: {}", current_hash);

    // 2. Find matching anchor
    let anchor_file = File::open(&args.anchors)?;
    let anchor_reader = BufReader::new(anchor_file);
    let mut found = false;

    for l in anchor_reader.lines().map_while(Result::ok) {
        if let Ok(record) = serde_json::from_str::<AnchorRecord>(&l) {
            if record.hash == current_hash {
                println!("\n✅ VERIFICATION SUCCESSFUL!");
                println!("Provider: {}", record.provider);
                println!("Timestamp: {}", record.timestamp);
                println!("Proof ID: {}", record.tx_id);
                found = true;
                break;
            }
        }
    }

    if !found {
        println!("\n❌ VERIFICATION FAILED!");
        println!("No matching anchor found for the current legends data.");
        println!("The data may have been tampered with or not yet anchored.");
    }

    Ok(())
}
