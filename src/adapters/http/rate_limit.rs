use std::sync::Arc;

use governor::middleware::NoOpMiddleware;
use tower_governor::{
    GovernorLayer,
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
};

/// Per-IP rate limiter using X-Forwarded-For / X-Real-IP with peer IP fallback.
pub type IpRateLayer = GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware>;

/// Create a per-IP rate limiter.
///
/// - `per_second`: refill interval in seconds (1 token every N seconds)
/// - `burst_size`: maximum burst capacity
pub fn per_ip(per_second: u64, burst_size: u32) -> IpRateLayer {
    GovernorLayer {
        config: Arc::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(per_second)
                .burst_size(burst_size)
                .finish()
                .expect("invalid rate limiter config"),
        ),
    }
}
