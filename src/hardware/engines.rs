use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo {
    pub name: String,
    pub version: Option<String>,
    pub found: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InferenceEngines {
    pub engines: Vec<EngineInfo>,
}

impl InferenceEngines {
    pub fn display(&self) -> String {
        let found: Vec<&str> = self
            .engines
            .iter()
            .filter(|e| e.found)
            .map(|e| e.name.as_str())
            .collect();

        if found.is_empty() {
            "None detected".to_string()
        } else {
            found.join(", ")
        }
    }
}

pub fn detect() -> InferenceEngines {
    let engines = vec![
        detect_llama_cpp(),
        detect_mlx(),
        detect_ollama(),
        detect_vllm(),
        detect_exllamav2(),
    ];

    InferenceEngines { engines }
}

fn detect_llama_cpp() -> EngineInfo {
    let found = command_exists("llama-server")
        || command_exists("llama-cli")
        || command_exists("llama-bench");

    EngineInfo {
        name: "llama.cpp".to_string(),
        version: None,
        found,
    }
}

fn detect_mlx() -> EngineInfo {
    let found = python_package_exists("mlx_lm");
    let version = if found {
        python_package_version("mlx_lm")
    } else {
        None
    };

    EngineInfo {
        name: "MLX".to_string(),
        version,
        found,
    }
}

fn detect_ollama() -> EngineInfo {
    let found = command_exists("ollama");

    EngineInfo {
        name: "Ollama".to_string(),
        version: None,
        found,
    }
}

fn detect_vllm() -> EngineInfo {
    let found = command_exists("vllm") || python_package_exists("vllm");

    EngineInfo {
        name: "vLLM".to_string(),
        version: None,
        found,
    }
}

fn detect_exllamav2() -> EngineInfo {
    let found = python_package_exists("exllamav2");

    EngineInfo {
        name: "ExLlamaV2".to_string(),
        version: None,
        found,
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn python_package_exists(package: &str) -> bool {
    Command::new("python3")
        .args(["-c", &format!("import {}", package)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn python_package_version(package: &str) -> Option<String> {
    let output = Command::new("python3")
        .args([
            "-c",
            &format!(
                "from importlib.metadata import version; print(version('{}'))",
                package
            ),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if v.is_empty() { None } else { Some(v) }
    } else {
        None
    }
}
