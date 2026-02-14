use primordium_core::systems::audio::AudioEvent;
use primordium_lib::app::audio::engine::AudioEngine;
use primordium_lib::app::audio::entropy_synth::FMSynthesizer;
use primordium_lib::app::audio::event_sfx::EventSFXGenerator;
use primordium_lib::app::audio::spatial::SpatialAudio;
use std::time::Instant;

fn bench_single_sample_generation() {
    let mut synth = FMSynthesizer::new(44100.0);

    let start = Instant::now();
    let iter_count = 1000;
    for i in 0..iter_count {
        let _ = synth.render_sample((i as f32) / (iter_count as f32), 1000.0);
    }
    let elapsed = start.elapsed();

    let avg_ns = elapsed.as_nanos() / iter_count as u128;
    let avg_us = avg_ns as f64 / 1000.0;

    println!(
        "Single FM synthesis sample generation: {:.2} μs/sample",
        avg_us
    );
    println!("  Total for {} samples: {:?}", iter_count, elapsed);
}

fn bench_render_block() {
    let mut engine = AudioEngine::new().expect("Failed to create AudioEngine");

    let start = Instant::now();
    let block_size = 256;
    let mut buffer = vec![0.0f32; block_size];

    let iter_count = 100;
    for _ in 0..iter_count {
        engine.render_block(&mut buffer);
    }

    let elapsed = start.elapsed();
    let total_samples = block_size * iter_count;
    let avg_ns = elapsed.as_nanos() as f64 / total_samples as f64;
    let avg_us = avg_ns / 1000.0;

    println!(
        "Render block ({} samples) x{} times:",
        block_size, iter_count
    );
    println!("  Average: {:.2} μs/sample", avg_us);
    println!("  Total: {:?}", elapsed);
}

fn bench_event_sfx_generation() {
    let generator = EventSFXGenerator::new(44100.0);

    let start = Instant::now();
    let waveform = generator.generate_waveform(AudioEvent::Birth);
    let elapsed = start.elapsed();

    println!("Birth event SFX generation:");
    println!("  Waveform length: {} samples", waveform.len());
    println!("  Total time: {:?}", elapsed);
}

fn bench_spatial_calculations() {
    let (width, height) = (1000u16, 1000u16);
    let test_cases = [
        (0.0, 0.0, "top-left"),
        (500.0, 500.0, "center"),
        (1000.0, 1000.0, "bottom-right"),
        (250.0, 750.0, "top-right"),
    ];

    let start = Instant::now();
    let iterations = 10000;

    for _ in 0..iterations {
        for &(x, y, _) in &test_cases {
            let (left, right) = SpatialAudio::calculate_stereo_panning(x, y, width, height);
            let max_dist = ((width as f64).powf(2.0) + (height as f64).powf(2.0)).sqrt();
            let dx = x - width as f64 / 2.0;
            let dy = y - height as f64 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            let _ = SpatialAudio::apply_distance_attenuation(
                if left > right { left } else { right },
                distance,
                max_dist,
            );
        }
    }

    let elapsed = start.elapsed();
    let total_calls = iterations * test_cases.len();

    println!("Spatial audio calculations (panning + attenuation):");
    println!(
        "  {} calls × {} iterations = {} total",
        test_cases.len(),
        iterations,
        total_calls
    );
    println!(
        "  Average per call: {:.2} ns",
        elapsed.as_nanos() as f64 / total_calls as f64
    );
    println!("  Total time: {:?}", elapsed);
}

fn main() {
    println!("=== Audio Performance Benchmarks ===");
    println!();

    bench_single_sample_generation();
    println!();

    bench_render_block();
    println!();

    bench_event_sfx_generation();
    println!();

    bench_spatial_calculations();
    println!();

    println!("=== All benchmarks completed ===");
}
