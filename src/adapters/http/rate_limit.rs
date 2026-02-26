use std::sync::Arc;

use governor::middleware::NoOpMiddleware;
use tower_governor::{
    GovernorLayer,
    governor::GovernorConfigBuilder,
    key_extractor::PeerIpKeyExtractor,
};

/// Per-IP rate limiter layer type (used by routers that need to name the type).
pub type IpRateLayer = GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware>;

/// Create a per-IP rate limiter.
///
/// - `per_second`: refill interval in seconds (1 token every N seconds)
/// - `burst_size`: maximum burst capacity
pub fn per_ip(per_second: u64, burst_size: u32) -> IpRateLayer {
    GovernorLayer {
        config: Arc::new(
            GovernorConfigBuilder::default()
                .per_second(per_second)
                .burst_size(burst_size)
                .finish()
                .expect("invalid rate limiter config"),
        ),
    }
}
