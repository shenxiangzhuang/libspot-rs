/// Configuration for initializing a SPOT detector
#[derive(Debug, Clone)]
pub struct SpotConfig {
    /// Decision probability (SPOT will flag extreme events with probability lower than this)
    pub q: f64,
    /// Lower tail mode (false for upper tail, true for lower tail)
    pub low_tail: bool,
    /// Do not include anomalies in the model
    pub discard_anomalies: bool,
    /// Excess level (high quantile that delimits the tail)
    pub level: f64,
    /// Maximum number of data points kept to analyze the tail
    pub max_excess: u64,
}

impl Default for SpotConfig {
    fn default() -> Self {
        SpotConfig {
            q: 0.0001,
            low_tail: false,
            discard_anomalies: true,
            level: 0.998,
            max_excess: 200,
        }
    }
} 