use colored::Colorize;

use crate::mdns::Endpoint;

pub fn endpoint(endpoint: &Endpoint) {
    if endpoint.name.is_none() {
        print!("<{}>\r\n", endpoint.address);
    } else {
        print!(
            "<{}> ({})\r\n",
            endpoint.address,
            endpoint.name.as_ref().unwrap()
        );
    }

    for service in endpoint.services.values() {
        if let Some(desc) = &service.description {
            print!("  {} {}\r\n", service.name.green(), desc.yellow());
        } else {
            print!("  {}\r\n", service.name.green());
        }

        for (key, values) in &service.properties.0 {
            for value in values {
                if key == "server" {
                    println!("    server: {}", value.bright_red());
                } else {
                    println!("    {}: {}", key, value.bright_blue());
                }
            }
        }
    }
    println!();
}
