use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::{MappedRwLockReadGuard, RwLock};
use tracing::{Event, Level, Subscriber};
#[cfg(feature = "log")]
use tracing_log::NormalizeEvent;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::event::CollectedEvent;

#[derive(Clone, Debug)]
pub enum AllowedTargets {
    All,
    Selected(HashSet<String>),
}

#[derive(Debug, Clone)]
pub struct EventCollector {
    allowed_targets: AllowedTargets,
    level: Level,
    events: Arc<RwLock<Vec<CollectedEvent>>>,
}

impl EventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(self, level: Level) -> Self {
        Self { level, ..self }
    }

    pub fn allowed_targets(self, allowed_targets: AllowedTargets) -> Self {
        Self {
            allowed_targets,
            ..self
        }
    }

    pub fn events(&self) -> Vec<CollectedEvent> {
        self.events.read().clone()
    }

    pub fn get_events(
        &self,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, Vec<CollectedEvent>>
    {
        self.events.read()
    }

    pub fn clear(&self) {
        self.events.write().clear();
    }

    fn collect(&self, event: CollectedEvent) {
        if event.level <= self.level {
            let should_collect = match self.allowed_targets {
                AllowedTargets::All => true,
                AllowedTargets::Selected(ref selection) => selection.contains(&event.target),
            };

            if should_collect {
                self.events.write().push(event);
            }
        }
    }
}

impl Default for EventCollector {
    fn default() -> Self {
        Self {
            allowed_targets: AllowedTargets::All,
            events: Arc::new(RwLock::new(Vec::new())),
            level: Level::TRACE, // capture everything by default.
        }
    }
}

impl<S> Layer<S> for EventCollector
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        #[cfg(feature = "log")]
        let normalized_meta = event.normalized_metadata();
        #[cfg(feature = "log")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "log"))]
        let meta = event.metadata();

        self.collect(CollectedEvent::new(event, meta));
    }
}
