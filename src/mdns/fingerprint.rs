use lazy_static::lazy_static;

use crate::mdns::{Endpoint, Fingerprint};

lazy_static! {
    static ref CHECKS: Vec<(&'static str, Fingerprint)> = {
        vec![
            (
                "_googlecast.",
                Fingerprint {
                    vendor: "google".to_string(),
                    kind: "chromecast".to_string(),
                },
            ),
            (
                "_adisk.",
                Fingerprint {
                    vendor: "".to_string(),
                    kind: "disk".to_string(),
                },
            ),
            (
                "_hue.",
                Fingerprint {
                    vendor: "philips".to_string(),
                    kind: "light".to_string(),
                },
            ),
            (
                "_device-info.",
                Fingerprint {
                    vendor: "apple".to_string(),
                    kind: "osx".to_string(),
                },
            ),
            (
                "_apple",
                Fingerprint {
                    vendor: "apple".to_string(),
                    kind: "apple".to_string(),
                },
            ),
        ]
    };
}

pub fn get(endpoint: &Endpoint) -> Option<Fingerprint> {
    for service in endpoint.services.values() {
        for (name, finger) in &*CHECKS {
            if service.name.contains(name) {
                return Some(finger.clone());
            }
        }
    }
    None
}
