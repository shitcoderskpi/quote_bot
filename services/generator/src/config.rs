use std::env;

#[derive(Debug)]
pub struct Config {
    pub nats_url: String,
    pub queue_name: String,
    pub results_queue: String,
    pub templates_dir: String,
    pub dpi: f32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let nats_url = env::var("NATS_URL").unwrap_or("nats://127.0.0.1:4222".to_string());
        
        let queue_name = env::var("QUEUE_NAME").unwrap_or("generate:jobs".to_string());
        
        let results_queue = env::var("RESULTS_QUEUE").unwrap_or("generate:results".to_string());
        let templates_dir = env::var("TEMPLATE").map_err(|_| "TEMPLATE environment variable is not set".to_string())?;
        
        let md = std::fs::metadata(&templates_dir)
            .map_err(|e| format!("Failed to access TEMPLATE path '{}': {}", templates_dir, e))?;
        if !md.is_dir() {
            return Err(format!("TEMPLATE path '{}' is not a directory", templates_dir));
        }
        
        let dpi = env::var("DPI")
            .ok()
            .and_then(|p| p.parse::<f32>().ok())
            .unwrap_or(300.0);

        Ok(Self {
            nats_url,
            queue_name,
            results_queue,
            templates_dir,
            dpi
        })
    }
}
