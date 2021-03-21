pub mod linux;
pub mod pc_common;
pub mod windows;
use std::time::Duration;

use futures::never::Never;

use crate::{config::TimetrackrsConfig, prelude::*};

#[enum_dispatch]
#[derive(Debug)]
pub enum CaptureArgs {
    /// Capture open window information from a (linux) X11 server
    X11(X11CaptureArgs),
    Windows(WindowsCaptureArgs),
}

pub struct CaptureConfig {
    pub interval: Duration,
    pub args: CaptureArgs,
}

#[enum_dispatch(CaptureArgs)]
pub trait CapturerCreator {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>>;
}

pub trait Capturer: Send {
    fn capture(&mut self) -> anyhow::Result<EventData>;
}

pub async fn capture_loop(db: DatyBasy, config: CaptureConfig) -> anyhow::Result<Never> {
    let CaptureConfig { args, interval } = config;
    let mut c = args
        .create_capturer()
        .with_context(|| format!("Could not create capturer from {:?}", &args))?;

    let idgen = libxid::new_generator();

    let mut interval = tokio::time::interval(config.interval);
    loop {
        log::info!("sleeping {}s", config.interval.as_secs());
        interval.tick().await;

        let data = c.capture()?;
        let act = CreateNewDbEvent {
            id: idgen.new_id().unwrap().encode(),
            timestamp: Utc::now(),
            duration_ms: config.interval.as_millis() as i64,
            data,
        };
        let ins: NewDbEvent = act.try_into()?;

        db.insert_events(vec![ins])
            .await
            .context("Could not insert captured event")?;
    }
}
