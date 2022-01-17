use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum Lifetime {
    Ping,
    Application,
    User
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum MemoryUnit {
    Byte,
    Kilobyte,
    Megabyte,
    Gigabyte,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum TimeUnit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
enum HistogramType {
    Linear,
    Exponential,
}

impl Default for Lifetime {
    fn default() -> Self {
        Lifetime::Ping
    }
}


#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct CommonMetricData {
    lifetime: Lifetime,
    description: String,
    bugs: Vec<String>,
    data_reviews: Vec<String>,
    notification_emails: Vec<String>,
    expires: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Metric {
    Boolean(CommonMetricData),

    #[serde(rename = "labeled_boolean")]
    LabeledBoolean(LabeledData),

    Counter(CommonMetricData),

    #[serde(rename = "labeled_counter")]
    LabeledCounter(LabeledData),

    String(CommonMetricData),

    #[serde(rename = "labeled_string")]
    LabeledString(LabeledData),

    #[serde(rename = "string_list")]
    StringList(CommonMetricData),

    #[serde(rename = "timing_distribution")]
    TimingDistribution(TimingDistribution),

    #[serde(rename = "memory_distribution")]
    MemoryDistribution(MemoryDistribution),

    #[serde(rename = "custom_distribution")]
    CustomDistribution(CustomDistribution),

    Quantity(Quantity),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabeledData {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    labels: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimingDistribution {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    time_unit: TimeUnit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryDistribution {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    memory_unit: MemoryUnit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomDistribution {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,

    range_min: usize,
    range_max: usize,
    bucket_count: usize,
    histogram_type: HistogramType
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quantity {
    #[serde(flatten)]
    pub common_metric_data: CommonMetricData,
    pub unit: Option<String>
}

pub type MetricList = BTreeMap<String, Metric>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryList {
    #[serde(rename = "$schema")]
    schema: String,

    #[serde(flatten)]
    categories: BTreeMap<String, MetricList>,
}
