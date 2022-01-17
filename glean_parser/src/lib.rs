use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

use heck::{ToUpperCamelCase, ToLowerCamelCase};
use jsonschema::JSONSchema;
use minijinja::{context, Environment, value::Value as JinjaValue, value::ValueKind};
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;
use yaml_merge_keys::merge_keys_serde;

use metrics::Metric;

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

    let all_metrics: metrics::Content = serde_yaml::from_str(&expanded_yaml)?;
    Ok(all_metrics.categories)
}

const METRICS_TEMPLATE: &str = include_str!("../templates/metrics.jinja2");

fn upper_camelize(_state: &minijinja::State, value: String) -> Result<String, minijinja::Error> {
    Ok(value.to_upper_camel_case())
}

fn lower_camelize(_state: &minijinja::State, value: String) -> Result<String, minijinja::Error> {
    Ok(value.to_lower_camel_case())
}

fn type_name(_state: &minijinja::State, metric: Metric) -> Result<String, minijinja::Error> {
    Ok(metric.type_name().to_upper_camel_case())
}

fn common_metric_data(_state: &minijinja::State, metric: Metric) -> Result<BTreeMap<&'static str, JinjaValue>, minijinja::Error> {
    Ok(metric.common_metric_data())
}

fn extra_data(_state: &minijinja::State, metric: Metric) -> Result<BTreeMap<&'static str, JinjaValue>, minijinja::Error> {
    Ok(metric.extra())
}

fn swift_ty(_state: &minijinja::State, value: JinjaValue) -> Result<String, minijinja::Error> {
    if let Some(s) = value.as_str() {
        return Ok(format!("{:?}", s))
    }

    if value.kind() == ValueKind::Number {
        return Ok(value.to_string())
    }

    if let Some(s) = value.downcast_object_ref::<metrics::Lifetime>() {
        return Ok(format!(".{}", s.to_string().to_lower_camel_case()))
    }

    if let Some(s) = value.downcast_object_ref::<metrics::MemoryUnit>() {
        return Ok(format!(".{}", s.to_string().to_lower_camel_case()))
    }

    if let Some(s) = value.downcast_object_ref::<metrics::TimeUnit>() {
        return Ok(format!(".{}", s.to_string().to_lower_camel_case()))
    }

    if let Some(s) = value.downcast_object_ref::<metrics::HistogramType>() {
        return Ok(format!(".{}", s.to_string().to_lower_camel_case()))
    }

    if value.kind() == ValueKind::Seq {
        let len = value.len().unwrap();
        let mut values = Vec::with_capacity(len);
        for i in 0..len {
            let item = match value.get_item(&i.into()) {
                Ok(item) => item,
                _ => break
            };
            values.push(swift_ty(_state, item)?);
        }

        return Ok(format!("[{}]", values.join(", ")));
    }

    dbg!(value);
    todo!()
}

pub fn generate(metrics: metrics::CategoryList) {
    let mut env = Environment::new();
    env.set_debug(true);
    env.add_template("metrics", METRICS_TEMPLATE).unwrap();
    env.add_filter("Camelize", upper_camelize);
    env.add_filter("camelize", lower_camelize);
    env.add_filter("type_name", type_name);
    env.add_filter("common_metric_data", common_metric_data);
    env.add_filter("extra", extra_data);
    env.add_filter("swift", swift_ty);
    let tmpl = env.get_template("metrics").unwrap();
    let ctx = context!(categories => metrics);

    println!("{}", tmpl.render(ctx).unwrap());
}
