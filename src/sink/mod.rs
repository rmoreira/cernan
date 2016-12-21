use hopper;
use metric::{Event, LogLine, Metric};
use std::sync;
use time;

mod console;
mod federation_transmitter;
mod firehose;
mod null;
mod wavefront;

pub use self::console::{Console, ConsoleConfig};
pub use self::federation_transmitter::{FederationTransmitter, FederationTransmitterConfig};
pub use self::firehose::{Firehose, FirehoseConfig};
pub use self::null::{Null, NullConfig};
pub use self::wavefront::{Wavefront, WavefrontConfig};

pub enum Valve {
    Open,
    Closed,
}

/// A 'sink' is a sink for metrics.
pub trait Sink {
    fn flush(&mut self) -> ();
    fn valve_state(&self) -> Valve;
    fn deliver(&mut self, point: sync::Arc<Metric>) -> ();
    fn deliver_line(&mut self, line: sync::Arc<LogLine>) -> ();
    fn run(&mut self, mut recv: hopper::Receiver<Event>) {
        let mut attempts = 0;
        loop {
            time::delay(attempts);
            match recv.next() {
                None => attempts += 1,
                Some(event) => {
                    attempts = 0;
                    match self.valve_state() {
                        Valve::Open => {
                            match event {
                                Event::TimerFlush => self.flush(),
                                Event::Telemetry(metric) => {
                                    self.deliver(metric);
                                }

                                Event::Log(line) => {
                                    self.deliver_line(line);
                                }
                            }
                        }                            
                        Valve::Closed => {
                            attempts += 1;
                            continue;
                        }
                    }
                }
            }
        }
    }
}
