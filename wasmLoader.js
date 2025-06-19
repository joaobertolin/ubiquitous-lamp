
// wasmLoader.js
let wasmModule = null;
let particleSimulation = null;

export async function initWasm() {
    try {
        const wasmPkg = await import('./pkg/particle_sim_wasm.js');
        await wasmPkg.default();
        wasmModule = wasmPkg;
        console.log('WASM module loaded successfully');
        return true;
    } catch (error) {
        console.error('Failed to load WASM module:', error);
        return false;
    }
}

export function createSimulation(particleCount, numTypes, canvasWidth, canvasHeight) {
    if (!wasmModule) {
        throw new Error('WASM module not loaded');
    }
    particleSimulation = new wasmModule.ParticleSimulation(particleCount, numTypes, canvasWidth, canvasHeight);
    return particleSimulation;
}

export function updateParams(params) {
    if (!particleSimulation) return;
    particleSimulation.update_params(
        params.radius,
        params.delta_t,
        params.friction,
        params.repulsion,
        params.attraction,
        params.k,
        params.balance,
        params.ratio,
        params.force_multiplier,
        params.max_expected_neighbors
    );
}

export function updateForceTable(forceTable) {
    if (!particleSimulation) return;
    particleSimulation.update_force_table(forceTable);
}

export function updateRadioByType(radioByType) {
    if (!particleSimulation) return;
    particleSimulation.update_radio_by_type(radioByType);
}

export function simulateStep() {
    if (!particleSimulation) return;
    particleSimulation.simulate_step();
}

export function getParticleData() {
    if (!particleSimulation) return null;
    return particleSimulation.get_particle_data();
}

export function resetParticles() {
    if (!particleSimulation) return;
    particleSimulation.reset_particles();
}

export function setCanvasSize(width, height) {
    if (!particleSimulation) return;
    particleSimulation.set_canvas_size(width, height);
}

export function isWasmAvailable() {
    return wasmModule !== null;
}
