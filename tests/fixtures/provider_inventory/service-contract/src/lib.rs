use std::convert::Infallible;

use spi::AsyncServiceSpec;
use spi::ServiceSpec;
use spi::SyncServiceSpec;

/// Service family shared by the cross-crate provider fixture.
pub struct FixtureSpec;

impl ServiceSpec for FixtureSpec {
    type Config = String;
    type Error = Infallible;
}

impl SyncServiceSpec for FixtureSpec {
    type Output = String;
}

impl AsyncServiceSpec for FixtureSpec {
    type Output = String;
}

spi::declare_sync_provider_inventory! {
    pub mod sync_providers {
        spec = crate::FixtureSpec;
    }
}

spi::declare_async_provider_inventory! {
    pub mod async_providers {
        spec = crate::FixtureSpec;
    }
}
