use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use dns_lookup::getnameinfo;
use dns_parser::RData;
use serde::{Deserialize, Serialize};

pub mod discovery;

mod fingerprint;

pub type PropertyValues = Vec<String>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Properties(pub HashMap<String, PropertyValues>);

impl Properties {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, key: &str, value: String) {
        if let Some(ref mut prop) = self.0.get_mut(key) {
            if !prop.contains(&value) {
                prop.push(value);
            }
        } else {
            self.0.insert(key.to_string(), vec![value]);
        }
    }

    pub fn merge(&mut self, props: &Properties) {
        for (key, values) in &props.0 {
            for value in values {
                self.add(key, value.to_string());
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&PropertyValues> {
        self.0.get(key)
    }

    pub fn has_ip(&self) -> bool {
        self.get("ipv4").is_some() || self.get("ipv6").is_some()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub description: Option<String>,
    pub properties: Properties,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Fingerprint {
    pub vendor: String,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub name: Option<String>,
    pub address: IpAddr,
    pub local: bool,
    pub services: HashMap<String, Service>,
    pub fingerprint: Option<Fingerprint>,
}

impl Endpoint {
    pub fn with_services<'b>(
        address: SocketAddr,
        records: impl Iterator<Item = &'b dns_parser::ResourceRecord<'b>>,
    ) -> Endpoint {
        let name = match getnameinfo(&address, 0) {
            Ok((name, _)) => {
                if name == address.ip().to_string() {
                    None
                } else {
                    Some(name)
                }
            }
            Err(_) => None,
        };

        let mut local = false;
        for iface in interfaces::Interface::get_all().expect("could not get network interfaces") {
            for addr in iface.addresses.iter() {
                if let Some(ip) = addr.addr {
                    if ip.ip() == address.ip() {
                        local = true;
                        break;
                    }
                }
            }
        }

        let mut endpoint = Endpoint {
            name,
            local,
            address: address.ip(),
            services: HashMap::new(),
            fingerprint: None,
        };
        endpoint.add_services(records);
        endpoint
    }

    fn add_ip_property(properties: &mut Properties, ip: String) {
        properties.add(if ip.contains(':') { "ipv6" } else { "ipv4" }, ip);
    }

    fn parse_properties<'b>(data: &'b RData<'b>) -> Properties {
        let mut properties = Properties::new();

        match data {
            RData::AAAA(ip) => Self::add_ip_property(&mut properties, ip.0.to_string()),
            RData::A(ip) => Self::add_ip_property(&mut properties, ip.0.to_string()),
            RData::PTR(name) => properties.add("name", name.0.to_string()),
            RData::SRV(server) => {
                properties.add("server", format!("{}:{}", server.target, server.port))
            }
            RData::Unknown(typ, raw) => properties.add(&format!("{:?}", typ), format!("{:?}", raw)),
            RData::TXT(txt) => {
                for chunk in txt.iter() {
                    if let Ok(str) = String::from_utf8(chunk.to_vec()) {
                        properties.add("text", str);
                    } else if !chunk.is_empty() {
                        properties.add("text", format!("{:?}", chunk));
                    }
                }
            }
            _ => properties.add("???", format!("{:?}", data)),
        }

        properties
    }

    pub fn add_services<'b>(
        &mut self,
        records: impl Iterator<Item = &'b dns_parser::ResourceRecord<'b>>,
    ) {
        // for every answer
        for rec in records {
            // println!("{:?} - {:?}", self.address, rec);
            // if this is not a mdns enumeration descriptor (which has already been parsed by the agent)
            let svc_name = rec.name.to_string();
            if svc_name != discovery::DNS_ENUMERATION_SERVICE_NAME {
                // parse record data into properties
                let properties = Self::parse_properties(&rec.data);
                // if this endpoint still has no name, check if this record can be used for it
                if self.name.is_none() && properties.has_ip() {
                    self.name = Some(svc_name.clone());
                }

                if let Some(service) = self.services.get_mut(&svc_name) {
                    // known service, update properties
                    service.properties.merge(&properties);
                } else {
                    // new service
                    let name = svc_name.to_owned();
                    let description = discovery::get_service_description(&name);
                    self.services.insert(
                        name.to_owned(),
                        Service {
                            name,
                            description,
                            properties,
                        },
                    );
                }

                // attempt fingerprinting
                if self.fingerprint.is_none() {
                    self.fingerprint = fingerprint::get(self);
                }
            }
        }
    }
}
