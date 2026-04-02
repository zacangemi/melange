use serde::Deserialize;

use crate::hardware::engines::InferenceEngines;
use crate::models::ModelInfo;

#[derive(Debug, Clone, Deserialize)]
pub struct CompatDb {
    #[serde(rename = "warnings")]
    pub entries: Vec<CompatWarning>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompatWarning {
    pub id: String,

    // Match criteria (AND logic — all specified must match)
    pub model_type: Option<String>,
    pub architecture: Option<String>,
    pub model_family: Option<String>,
    pub is_moe: Option<bool>,

    // Engine this warning applies to
    pub engine: String,

    pub severity: WarningSeverity,
    pub summary: String,
    pub detail: Option<String>,
    #[serde(default)]
    pub workaround: Option<String>,
    #[serde(default)]
    pub fixed_in: Option<String>,
    #[serde(default)]
    pub references: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum WarningSeverity {
    // Ord: variants ordered highest-severity-first for sorting
    Breaking,
    Caution,
    Info,
}

impl WarningSeverity {
    pub fn icon(&self) -> &'static str {
        match self {
            WarningSeverity::Info => "ℹ",
            WarningSeverity::Caution => "⚠",
            WarningSeverity::Breaking => "✗",
        }
    }
}

/// Load the bundled compatibility database, merged with any user overrides.
pub fn load_compat_db() -> CompatDb {
    let bundled_toml = include_str!("../../assets/compat_warnings.toml");
    let mut db: CompatDb = toml::from_str(bundled_toml).unwrap_or(CompatDb { entries: vec![] });

    // Merge user overrides from ~/.config/melange/compat_warnings.toml
    if let Some(user_path) = user_compat_path() {
        if user_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&user_path) {
                if let Ok(user_db) = toml::from_str::<CompatDb>(&contents) {
                    for user_entry in user_db.entries {
                        // Replace by id if exists, otherwise append
                        if let Some(pos) = db.entries.iter().position(|e| e.id == user_entry.id) {
                            db.entries[pos] = user_entry;
                        } else {
                            db.entries.push(user_entry);
                        }
                    }
                }
            }
        }
    }

    db
}

/// Find model-specific warnings that apply to a given model + detected engines.
/// Excludes engine-wide wildcard warnings. Sorted by severity (breaking first).
pub fn find_warnings<'a>(
    db: &'a CompatDb,
    model: &ModelInfo,
    engines: &InferenceEngines,
) -> Vec<&'a CompatWarning> {
    let mut results: Vec<&CompatWarning> = db
        .entries
        .iter()
        .filter(|w| warning_matches(w, model, engines))
        .collect();

    results.sort_by(|a, b| a.severity.cmp(&b.severity));
    results
}

fn warning_matches(warning: &CompatWarning, model: &ModelInfo, engines: &InferenceEngines) -> bool {
    // Engine must be detected and found
    let engine_found = engines
        .engines
        .iter()
        .any(|e| e.found && e.name.to_lowercase() == warning.engine.to_lowercase());

    if !engine_found {
        return false;
    }

    // Skip engine-wide wildcard warnings — only show model-specific ones
    let is_wildcard = warning.architecture.as_deref() == Some("any")
        || warning.model_type.as_deref() == Some("*");

    if is_wildcard {
        return false;
    }

    // At least one model criterion must be specified
    let has_criterion = warning.model_type.is_some()
        || warning.architecture.is_some()
        || warning.model_family.is_some()
        || warning.is_moe.is_some();

    if !has_criterion {
        return false;
    }

    // All specified criteria must match (AND logic)
    if let Some(ref mt) = warning.model_type {
        if !model.model_type.to_lowercase().contains(&mt.to_lowercase()) {
            return false;
        }
    }

    if let Some(ref arch) = warning.architecture {
        if !model.architecture.to_lowercase().contains(&arch.to_lowercase()) {
            return false;
        }
    }

    if let Some(ref family) = warning.model_family {
        if !model.name.to_lowercase().starts_with(&family.to_lowercase()) {
            return false;
        }
    }

    if let Some(moe) = warning.is_moe {
        if model.is_moe != moe {
            return false;
        }
    }

    true
}

fn user_compat_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("melange").join("compat_warnings.toml"))
}
