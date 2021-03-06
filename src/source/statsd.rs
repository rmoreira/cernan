use metric;
use protocols::statsd::parse_statsd;
use source::Source;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::str;
use std::sync;
use std::thread;
use util;
use util::send;

pub struct Statsd {
    chans: util::Channel,
    port: u16,
    tags: sync::Arc<metric::TagMap>,
}

#[derive(Debug,Clone)]
pub struct StatsdConfig {
    pub ip: String,
    pub port: u16,
    pub tags: metric::TagMap,
    pub forwards: Vec<String>,
    pub config_path: String,
    pub delete_gauges: bool,
}

impl Default for StatsdConfig {
    fn default() -> StatsdConfig {
        StatsdConfig {
            ip: String::from(""),
            port: 8125,
            tags: metric::TagMap::default(),
            forwards: Vec::new(),
            config_path: "sources.statsd".to_string(),
            delete_gauges: false,
        }
    }
}

impl Statsd {
    pub fn new(chans: util::Channel, config: StatsdConfig) -> Statsd {
        Statsd {
            chans: chans,
            port: config.port,
            tags: sync::Arc::new(config.tags),
        }
    }
}

fn handle_udp(mut chans: util::Channel, tags: sync::Arc<metric::TagMap>, socket: UdpSocket) {
    let mut buf = [0; 8192];
    let mut metrics = Vec::new();
    let basic_metric = sync::Arc::new(Some(metric::Telemetry::default()
        .overlay_tags_from_map(&tags)));
    loop {
        let (len, _) = match socket.recv_from(&mut buf) {
            Ok(r) => r,
            Err(_) => panic!("Could not read UDP socket."),
        };
        match str::from_utf8(&buf[..len]) {
            Ok(val) => {
                if parse_statsd(val, &mut metrics, basic_metric.clone()) {
                    for m in metrics.drain(..) {
                        send("statsd", &mut chans, metric::Event::new_telemetry(m));
                    }
                    let mut metric = metric::Telemetry::new("cernan.statsd.packet", 1.0).aggr_sum();
                    metric = metric.overlay_tags_from_map(&tags);
                    send("statsd", &mut chans, metric::Event::new_telemetry(metric));
                } else {
                    let mut metric = metric::Telemetry::new("cernan.statsd.bad_packet", 1.0)
                        .aggr_sum();
                    metric = metric.overlay_tags_from_map(&tags);
                    send("statsd", &mut chans, metric::Event::new_telemetry(metric));
                    error!("BAD PACKET: {:?}", val);
                }
            }
            Err(e) => {
                error!("Payload not valid UTF-8: {:?}", e);
            }
        }
    }
}

impl Source for Statsd {
    fn run(&mut self) {
        let mut joins = Vec::new();

        let addr_v6 = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), self.port, 0, 0);
        let socket_v6 = UdpSocket::bind(addr_v6).expect("Unable to bind to UDP V6 socket");
        let chans_v6 = self.chans.clone();
        let tags_v6 = self.tags.clone();
        info!("server started on ::1 {}", self.port);
        joins.push(thread::spawn(move || handle_udp(chans_v6, tags_v6, socket_v6)));

        let addr_v4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), self.port);
        let socket_v4 = UdpSocket::bind(addr_v4).expect("Unable to bind to UDP socket");
        let chans_v4 = self.chans.clone();
        let tags_v4 = self.tags.clone();
        info!("server started on 127.0.0.1:{}", self.port);
        joins.push(thread::spawn(move || handle_udp(chans_v4, tags_v4, socket_v4)));

        for jh in joins {
            // TODO Having sub-threads panic will not cause a bubble-up if that
            // thread is not the currently examined one. We're going to have to have
            // some manner of sub-thread communication going on.
            jh.join().expect("Uh oh, child thread paniced!");
        }
    }
}
