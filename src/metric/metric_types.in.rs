#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct LogLine {
    pub time: i64,
    pub path: String,
    pub value: String,
    pub tags: TagMap,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct Metric {
    pub kind: MetricKind,
    pub time: i64,
    pub created_time: i64,
    pub name: String,
    pub tags: sync::Arc<TagMap>,
    value: MetricValue,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    Telemetry(sync::Arc<Option<Metric>>),
    Log(sync::Arc<Option<LogLine>>),
    TimerFlush,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum MetricValueKind {
    Single,
    Many,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct MetricValue {
    kind: MetricValueKind,
    single: Option<f64>,
    many: Option<CKMS<f64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum MetricKind {
    Counter,
    Gauge,
    DeltaGauge,
    Timer,
    Histogram,
    Raw,
}
