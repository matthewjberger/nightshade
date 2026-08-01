fn main() -> Result<(), Box<dyn std::error::Error>> {
    boids_gpu::launch()
}
