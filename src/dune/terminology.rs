/// Tech term mapping
#[allow(dead_code)]
pub fn term<'a>(tech_name: &'a str) -> &'a str {
    match tech_name {
        "available_ram" => "Memory Reserves",
        "compatibility" => "Fit Status",
        "gpu_cores" => "GPU Cores",
        "context_window" => "Context Window",
        "tok_s" => "Token Rate",
        "swap_danger" => "OOM",
        "safe_to_run" => "Fits",
        "max_safe_context" => "Horizon Limit",
        "os_reserved" => "OS Reserved",
        "weight_memory" => "Weight Memory",
        "kv_cache" => "KV Cache",
        "overhead" => "Overhead",
        "total_memory" => "Total Memory",
        _ => tech_name,
    }
}
