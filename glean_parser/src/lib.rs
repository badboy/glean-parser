use std::error::Error;
use std::fs;

use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;
use yaml_merge_keys::merge_keys_serde;

mod metrics;

const METRICS_SCHEMA_V1_DATA: &str = include_str!("../schemas/metrics.2-0-0.schema.yaml");
static METRICS_SCHEMA_V1: Lazy<JSONSchema> = Lazy::new(|| {
    let data: JsonValue =
        serde_yaml::from_str(METRICS_SCHEMA_V1_DATA).expect("invalid schema data");
    JSONSchema::compile(&data).expect("invalid schema data")
});

pub fn validate(file: &str) -> Result<metrics::CategoryList, Box<dyn Error>> {
    let content = fs::read_to_string(file)?;
    let instance = serde_yaml::from_str(&content)?;
    let instance = merge_keys_serde(instance)?;
    let expanded_yaml = serde_yaml::to_string(&instance)?;
    let instance: JsonValue = serde_yaml::from_str(&expanded_yaml)?;

    {
        let result = METRICS_SCHEMA_V1.validate(&instance);
        if let Err(errors) = result {
            for error in errors {
                println!("Validation error at {}: {}", error.instance_path, error);
            }

            return Err("failed".into());
        }
    }

    let all_metrics: metrics::CategoryList = serde_yaml::from_str(&expanded_yaml)?;

    Ok(all_metrics)
}
