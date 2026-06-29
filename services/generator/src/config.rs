use std::env;

#[derive(Debug)]
pub struct Config {
    pub redis_host: String,
    pub redis_port: u16,
    pub queue_name: String,
    pub results_queue: String,
    pub templates_dir: String,
    pub dpi: f32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let redis_host = env::var("REDIS_HOST").map_err(|_| "REDIS_HOST environment variable is not set".to_string())?;
        
        let redis_port = env::var("REDIS_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(6379);

        let queue_name = env::var("QUEUE_NAME").unwrap_or_else(|_| "generate:jobs".to_string());
        
        let results_queue = env::var("RESULTS_QUEUE").unwrap_or_else(|_| "generate:results".to_string());
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
            redis_host,
            redis_port,
            queue_name,
            results_queue,
            templates_dir,
            dpi
        })
    }
}
