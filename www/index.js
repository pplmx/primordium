import init, { Simulation } from './pkg/primordium.js';

const canvas = document.getElementById('sim-canvas');
const ctx = canvas.getContext('2d');
const loading = document.getElementById('loading');
const uiPanel = document.getElementById('ui-panel');
const fpsEl = document.getElementById('fps');
const ticksEl = document.getElementById('ticks');
const entitiesEl = document.getElementById('entities');
const btnPause = document.getElementById('btn-pause');
const btnReset = document.getElementById('btn-reset');

// State
let simulation = null;
let animationId = null;
let lastTime = performance.now();
let frames = 0;
let isPaused = false;
let width = window.innerWidth;
let height = window.innerHeight;

// Configuration
const TARGET_FPS = 60;
const FRAME_TIME = 1000 / TARGET_FPS;

async function start() {
    try {
        console.log("Initializing WASM...");
        await init();

        // Setup canvas
        resize();
        window.addEventListener('resize', resize);

        console.log("Creating Simulation...");
        simulation = Simulation.new();

        loading.style.display = 'none';
        uiPanel.style.display = 'block';

        loop(performance.now());

    } catch (e) {
        console.error("Failed to start:", e);
        loading.innerText = "Initialization Failed: " + e;
        loading.style.color = "#ff4444";
    }
}

function resize() {
    width = window.innerWidth;
    height = window.innerHeight;
    canvas.width = width;
    canvas.height = height;
    // Potentially re-init renderer if it cached size, but ours passes size on draw
}

function loop(currentTime) {
    animationId = requestAnimationFrame(loop);

    if (isPaused) return;

    const deltaTime = currentTime - lastTime;

    if (deltaTime >= FRAME_TIME) {
        lastTime = currentTime - (deltaTime % FRAME_TIME);

        // Update physics
        try {
            simulation.tick();
        } catch (e) {
            console.error(e);
            cancelAnimationFrame(animationId);
            return;
        }

        // Draw
        simulation.draw(ctx, width, height);

        // Update UI
        updateStats();

        frames++;
    }
}

// FPS Counter
setInterval(() => {
    fpsEl.innerText = frames;
    frames = 0;
}, 1000);

function updateStats() {
    if (simulation) {
        const stats = simulation.get_stats();
        ticksEl.innerText = stats.tick;
        entitiesEl.innerText = stats.entities;
    }
}

// Controls
btnPause.addEventListener('click', () => {
    isPaused = !isPaused;
    btnPause.innerText = isPaused ? "Resume" : "Pause";
    btnPause.classList.toggle('active', isPaused);
});

btnReset.addEventListener('click', () => {
    simulation = Simulation.new();
});

start();
