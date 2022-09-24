use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dns_parser::RData;
use lazy_static::lazy_static;

use crate::mdns;

pub type MappedEndpoints = HashMap<IpAddr, mdns::Endpoint>;
pub type SharedEndpoints = Arc<Mutex<MappedEndpoints>>;

// https://datatracker.ietf.org/doc/html/rfc6763#section-9
pub const DNS_ENUMERATION_SERVICE_NAME: &str = "_services._dns-sd._udp.local";

lazy_static! {
    pub static ref KNOWN_SERVICES: HashMap<&'static str, &'static str> = {
        let mut services = HashMap::new();

        services.insert("_services._dns-sd.", "mDNS Enumeration Service");
        services.insert("_osc.", "MIDI OSC Bridge");
        services.insert("_apple-midi.", "Apple MIDI Network Driver");
        services.insert("_adisk.", "Time Capsule Backups");
        services.insert("_afpovertcp.", "AppleTalk Filing Protocol (AFP)");
        services.insert("_airdroid.", "AirDroid App");
        services.insert("_airdrop.", "OSX AirDrop");
        services.insert("_airplay.", "Apple TV");
        services.insert("_airport.", "AirPort Base Station");
        services.insert("_amzn-wplay.", "Amazon Devices");
        services.insert("_sub._apple-mobdev2.", "OSX Wi-Fi Sync");
        services.insert("_apple-mobdev2.", "OSX Wi-Fi Sync");
        services.insert("_apple-sasl.", "Apple Password Server");
        services.insert("_appletv-v2.", "Apple TV Home Sharing");
        services.insert("_atc.", "Apple Shared iTunes Library");
        services.insert("_sketchmirror.", "Sketch App");
        services.insert("_bcbonjour.", "Sketch App");
        services.insert("_companion-link.", "Airplay 2");
        services.insert("_cloud.", "Cloud by Dapile");
        services.insert("_daap.", "Digital Audio Access Protocol (DAAP)");
        services.insert("_device-info.", "OSX Device Info");
        services.insert("_distcc.", "Distributed Compiler");
        services.insert("_dpap.", "Digital Photo Access Protocol (DPAP)");
        services.insert("_eppc.", "Remote AppleEvents");
        services.insert("_esdevice.", "ES File Share App");
        services.insert("_esfileshare.", "ES File Share App");
        services.insert("_ftp.", "File Transfer Protocol (FTP)");
        services.insert("_googlecast.", "Google Cast (Chromecast)");
        services.insert("_googlezone.", "Google Zone (Chromecast)");
        services.insert("_hap.", "Apple HomeKit - HomeKit Accessory Protocol");
        services.insert("_homekit.", "Apple HomeKit");
        services.insert("_home-sharing.", "iTunes Home Sharing");
        services.insert("_http.", "Hypertext Transfer Protocol (HTTP)");
        services.insert("_hudson.", "Jenkins App");
        services.insert("_hue.", "Philips Hue Smart Bulbs");
        services.insert("_ica-networking.", "Image Capture Sharing");
        services.insert("_ichat.", "iChat Instant Messaging Protocol");
        services.insert("_print._sub._ipp.", "Printers (AirPrint)");
        services.insert("_cups._sub._ipps.", "Printers");
        services.insert("_print._sub._ipps.", "Printers");
        services.insert("_jenkins.", "Jenkins App");
        services.insert("_apple-lgremote.", "Apple Logic Remote");
        services.insert("_KeynoteControl.", "OSX Keynote");
        services.insert("_keynotepair.", "OSX Keynote");
        services.insert("_mediaremotetv.", "Apple TV Media Remote");
        services.insert("_nfs.", "Network File System (NFS)");
        services.insert("_nvstream.", "NVIDIA Shield Game Streaming");
        services.insert("_androidtvremote.", "Nvidia Shield / Android TV");
        services.insert("_omnistate.", "OmniGroup (OmniGraffle and other apps)");
        services.insert("_pdl-datastream.", "PDL Data Stream (Port 9100)");
        services.insert("_photoshopserver.", "Adobe Photoshop Nav");
        services.insert("_printer.", "Printers - Line Printer Daemon (LPD/LPR)");
        services.insert("_raop.", "AirPlay - Remote Audio Output Protocol");
        services.insert("_readynas.", "Netgear ReadyNAS");
        services.insert("_rfb.", "OSX Screen Sharing");
        services.insert("_physicalweb.", "Physical Web");
        services.insert("_riousbprint.", "Remote I/O USB Printer Protocol");
        services.insert("_rsp.", "Roku Server Protocol");
        services.insert("_scanner.", "Scanners");
        services.insert("_servermgr.", "Server Admin");
        services.insert("_sftp-ssh.", "Protocol - SFTP");
        services.insert("_sleep-proxy.", "Wake-on-Network / Bonjour Sleep Proxy");
        services.insert("_smb.", "Protocol - SMB");
        services.insert("_spotify-connect.", "Spotify Connect");
        services.insert("_ssh.", "Protocol - SSH");
        services.insert("_teamviewer.", "TeamViewer");
        services.insert("_telnet.", "Remote Login (TELNET)");
        services.insert("_touch-able.", "Apple TV Remote App (iOS devices)");
        services.insert("_tunnel.", "Tunnel");
        services.insert("_udisks-ssh.", "Ubuntu / Raspberry Pi Advertisement");
        services.insert("_webdav.", "WebDAV File System (WEBDAV)");
        services.insert("_workstation.", "Workgroup Manager");
        services.insert("_xserveraid.", "Xserve RAID");
        services
    };
}

pub const ADDR_ANY: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
pub const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const MULTICAST_PORT: u16 = 5353;

const RECV_BUFFER_SIZE: usize = 4096;

pub fn get_service_description(svc_name: &str) -> Option<String> {
    let svc_name = svc_name.to_ascii_lowercase();
    for (known_name, known_desc) in &*KNOWN_SERVICES {
        if svc_name.contains(&known_name.to_ascii_lowercase()) {
            return Some(known_desc.to_string());
        }
    }
    None
}
pub struct Question {
    services: Vec<String>,
}

impl Question {
    pub fn new() -> Self {
        let services = vec![DNS_ENUMERATION_SERVICE_NAME.to_owned()];
        Self { services }
    }

    pub fn query(&self) -> Vec<u8> {
        let mut builder = dns_parser::Builder::new_query(0, false);

        for svc in &self.services {
            builder.add_question(
                svc,
                false,
                dns_parser::QueryType::PTR,
                dns_parser::QueryClass::IN,
            );
        }

        builder.build().unwrap()
    }

    pub fn add_services<'a>(
        &mut self,
        records: impl Iterator<Item = &'a dns_parser::ResourceRecord<'a>>,
    ) -> Option<Vec<u8>> {
        let mut changed = false;

        for rec in records {
            let svc_name = rec.name.to_string();
            if svc_name == DNS_ENUMERATION_SERVICE_NAME {
                if let RData::PTR(name) = rec.data {
                    let data_name = name.0.to_string();
                    if !self.services.contains(&data_name) {
                        self.services.push(data_name);
                        changed = true;
                    }
                }
            }
        }

        if changed {
            Some(self.query())
        } else {
            None
        }
    }
}

pub struct Channel {
    passive: bool,
    address: SocketAddr,
    socket: std::net::UdpSocket,
    recv_buffer: Vec<u8>,

    query_time: Duration,
    last_query: Option<Instant>,
    question: Question,
    query_data: Vec<u8>,
}

#[cfg(not(target_os = "windows"))]
fn create_socket() -> io::Result<std::net::UdpSocket> {
    use net2::unix::UnixUdpBuilderExt;

    net2::UdpBuilder::new_v4()?
        .reuse_address(true)?
        .reuse_port(true)?
        .bind((ADDR_ANY, MULTICAST_PORT))
}

#[cfg(target_os = "windows")]
fn create_socket() -> io::Result<std::net::UdpSocket> {
    net2::UdpBuilder::new_v4()?
        .reuse_address(true)?
        .bind((ADDR_ANY, MULTICAST_PORT))
}

impl Channel {
    pub fn new(query_time_secs: u64, passive: bool) -> Result<Self, String> {
        let address = SocketAddr::new(MULTICAST_ADDR.into(), MULTICAST_PORT);
        let recv_buffer = vec![0; RECV_BUFFER_SIZE];

        let socket = create_socket().map_err(|e| e.to_string())?;
        socket
            .set_multicast_loop_v4(false)
            .map_err(|e| e.to_string())?;
        socket
            .join_multicast_v4(&MULTICAST_ADDR, &ADDR_ANY)
            .map_err(|e| e.to_string())?;

        let query_time = Duration::from_secs(query_time_secs);
        let last_query = None;
        let question = Question::new();
        let query_data = question.query();

        Ok(Self {
            passive,
            address,
            socket,
            recv_buffer,
            query_time,
            last_query,
            question,
            query_data,
        })
    }

    pub fn send_query_if_needed(&mut self) {
        if !self.passive
            && (self.last_query.is_none() || self.last_query.unwrap().elapsed() >= self.query_time)
        {
            if let Err(e) = self.socket.send_to(&self.query_data, self.address) {
                println!("error sending multicast query: {:?}", e);
            } else {
                self.last_query = Some(Instant::now());
            }
        }
    }

    pub fn read_packet(&mut self) -> Option<(SocketAddr, dns_parser::Packet)> {
        let (count, source) = self.socket.recv_from(&mut self.recv_buffer).unwrap();
        if count > 0 {
            let parsed = dns_parser::Packet::parse(&self.recv_buffer[..count]);
            if let Ok(packet) = parsed {
                // check new services to discover
                let records = packet.answers.iter().chain(packet.additional.iter());
                let new_query = self.question.add_services(records);
                if let Some(new_query) = new_query {
                    // trigger new query
                    self.query_data = new_query;
                    self.last_query = None;
                }

                // only interested in responses
                if !packet.header.query {
                    return Some((source, packet));
                }
            } else {
                println!("error parsing packet: {:?}", parsed);
            }
        }
        None
    }
}

pub struct Agent {
    channel: Channel,
    endpoints: SharedEndpoints,
    filter_for: Option<String>,
}

impl Agent {
    pub fn new(
        query_time_secs: u64,
        passive: bool,
        filter_for: Option<String>,
    ) -> Result<Self, String> {
        let channel = Channel::new(query_time_secs, passive)?;
        let endpoints = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            channel,
            endpoints,
            filter_for,
        })
    }

    pub fn start(&mut self, cb: impl Fn(SharedEndpoints)) {
        println!(
            "started in {} mode ...",
            if self.channel.passive {
                "passive"
            } else {
                "active"
            }
        );

        loop {
            // send query if interval has elapsed and we're not in passive mode
            self.channel.send_query_if_needed();

            // wait for a response packet
            if let Some((source, packet)) = self.channel.read_packet() {
                // skip if we need to filter by address and this is not it
                if let Some(ref address) = self.filter_for {
                    if source.ip().to_string() != *address {
                        continue;
                    }
                }

                // check if we have any answers
                if !packet.answers.is_empty() || !packet.additional.is_empty() {
                    // collect answers + additional records
                    let records = packet.answers.iter().chain(packet.additional.iter());
                    let source_ip = source.ip();
                    // update endpoints
                    if let Ok(mut guard) = self.endpoints.lock() {
                        if let Some(endpoint) = guard.get_mut(&source_ip) {
                            // known endpoint, update services and properties
                            endpoint.add_services(records)
                        } else {
                            // new endpoint
                            guard.insert(source_ip, mdns::Endpoint::with_services(source, records));
                        }
                    }
                    // pass to callback
                    cb(self.endpoints.clone());
                }
            }
        }
    }
}
