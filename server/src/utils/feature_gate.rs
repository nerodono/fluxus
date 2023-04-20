use galaxy_network::reader::ReadResult;

#[cfg(feature = "http")]
use crate::features::http::HttpFeature;

#[derive(Clone)]
pub struct FeatureGate {
    #[cfg(feature = "http")]
    http: HttpFeature,
}

impl FeatureGate {
    #[cfg(feature = "http")]
    pub const fn http(&self) -> ReadResult<&HttpFeature> {
        Ok(&self.http)
    }

    #[cfg(not(feature = "http"))]
    pub const fn http(&self) -> ReadResult<&HttpFeature> {
        Err(ReadError::NonCritical(NonCriticalError::FeatureIsDisabled))
    }

    pub const fn new(#[cfg(feature = "http")] http: HttpFeature) -> Self {
        Self {
            #[cfg(feature = "http")]
            http,
        }
    }
}
