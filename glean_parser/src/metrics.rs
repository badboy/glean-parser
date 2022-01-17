use std::fmt;
use std::collections::BTreeMap;

use minijinja::value::{ArgType, Value as JinjaValue, Object};
use serde::Deserialize;
use serde::Serialize;

macro_rules! value {
    (o $val:expr) => {
        JinjaValue::from_object($val)
    };
    ($val:expr) => {
        JinjaValue::from_serializable(&$val)
    }
}

macro_rules! object {
    ($ty:ty) => {
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl Object for $ty { }
    }
}

macro_rules! extra {
    ($ty:ty, $($o:ident $extra:tt),+) => {
        impl $ty {
            fn extra(&self) -> BTreeMap<&'static str, JinjaValue> {
                BTreeMap::from([
                    $(
                        extra!(@expand self $o $extra)
                    ),+
                ])
            }
        }
    };
    (@expand $this:ident o $extra:tt) => {
        (stringify!($extra), value!(o $this.$extra.clone()))
    };
    (@expand $this:ident v $extra:tt) => {
        (stringify!($extra), value!($this.$extra))
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Lifetime {
    Ping,
    Application,
    User,
}
object!(Lifetime);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MemoryUnit {
    Byte,
    Kilobyte,
    Megabyte,
    Gigabyte,
}
object!(MemoryUnit);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TimeUnit {
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
}
object!(TimeUnit);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum HistogramType {
    Linear,
    Exponential,
}
object!(HistogramType);

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

    Timespan(Timespan),

    #[serde(rename = "timing_distribution")]
    TimingDistribution(TimingDistribution),

    #[serde(rename = "memory_distribution")]
    MemoryDistribution(MemoryDistribution),

    #[serde(rename = "custom_distribution")]
    CustomDistribution(CustomDistribution),

    Uuid(CommonMetricData),

    Url(CommonMetricData),

    Datetime(Datetime),

    Event(Event),

    Rate(CommonMetricData),

    #[serde(rename = "rate_external")]
    RateExternal(RateExternal),

    Text(CommonMetricData),

    Quantity(Quantity),
}

impl Metric {
    pub fn type_name(&self) -> &'static str {
        use Metric::*;
        match self {
            Counter(_) => "counter",
            LabeledCounter(_) => "labeled_counter",
            Boolean(_) => "boolean",
            LabeledBoolean(_) => "labeled_boolean",
            String(_) => "string",
            LabeledString(_) => "labeled_string",
            StringList(_) => "string_list",
            Timespan(_) => "timespan",
            TimingDistribution(_) => "timing_distribution",
            MemoryDistribution(_) => "memory_distribution",
            CustomDistribution(_) => "custom_distribution",
            Uuid(_) => "uuid",
            Url(_) => "url",
            Datetime(_) => "datetime",
            Event(_) => "event",
            Rate(_) | RateExternal(_) => "rate",
            Text(_) => "text",
            Quantity(_) => "quantity",
        }
    }

    pub fn common_metric_data(&self) -> BTreeMap<&'static str, JinjaValue> {
        let cm = match self {
            | Metric::Counter(cm)
            | Metric::LabeledCounter(LabeledData { common_metric_data: cm, .. })
            | Metric::Boolean(cm)
            | Metric::LabeledBoolean(LabeledData { common_metric_data: cm, .. })
            | Metric::String(cm)
            | Metric::LabeledString(LabeledData { common_metric_data: cm, .. })
            | Metric::StringList(cm)
            | Metric::Timespan(Timespan { common_metric_data: cm, .. })
            | Metric::TimingDistribution(TimingDistribution { common_metric_data: cm, .. })
            | Metric::MemoryDistribution(MemoryDistribution { common_metric_data: cm, .. })
            | Metric::CustomDistribution(CustomDistribution { common_metric_data: cm, .. })
            | Metric::Uuid(cm)
            | Metric::Url(cm)
            | Metric::Datetime(Datetime { common_metric_data: cm, .. })
            | Metric::Event(Event { common_metric_data: cm, .. })
            | Metric::Rate(cm)
            | Metric::RateExternal(RateExternal { common_metric_data: cm, .. })
            | Metric::Text(cm)
            | Metric::Quantity(Quantity {
                common_metric_data: cm,
                ..
            }) => cm,
        };
        BTreeMap::from([
            ("category", value!("")),
            ("name", value!("")),
            ("lifetime", value!(o cm.lifetime.clone())),
            ("description", value!(cm.description)),
            ("send_in_pings", value!(vec!["metrics".to_string()])),
            ("disabled", value!(false)),
        ])
    }

    pub fn extra(&self) -> BTreeMap<&'static str, JinjaValue> {
        match self {
            Metric::Timespan(o) => o.extra(),
            Metric::Datetime(o) => o.extra(),
            Metric::TimingDistribution(o) => o.extra(),
            Metric::MemoryDistribution(o) => o.extra(),
            Metric::CustomDistribution(o) => o.extra(),
            _ => BTreeMap::new(),
        }
    }
}

impl ArgType for Metric {
    fn from_value(value: Option<JinjaValue>) -> Result<Self, minijinja::Error> {
        let value = value.expect("value required");

        let f = format!("{}", value);
        let metric: Metric = serde_json::from_str(&f).unwrap();
        Ok(metric)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabeledData {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    labels: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Timespan {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    time_unit: TimeUnit,
}
extra!(Timespan, o time_unit);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Datetime {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    time_unit: TimeUnit,
}
extra!(Datetime, o time_unit);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,

    extra_keys: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimingDistribution {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    time_unit: TimeUnit,
}
extra!(TimingDistribution, o time_unit);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryDistribution {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    memory_unit: MemoryUnit,
}
extra!(MemoryDistribution, o memory_unit);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomDistribution {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,

    range_min: usize,
    range_max: usize,
    bucket_count: usize,
    histogram_type: HistogramType,
}
extra!(CustomDistribution, v range_min, v range_max, v bucket_count, o histogram_type);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateExternal {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,

    denominator_metric: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quantity {
    #[serde(flatten)]
    common_metric_data: CommonMetricData,
    unit: Option<String>,
}

pub type MetricList = BTreeMap<String, Metric>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Content {
    #[serde(rename = "$schema")]
    schema: String,

    #[serde(flatten)]
    pub categories: CategoryList,
}

pub type CategoryList = BTreeMap<String, MetricList>;
