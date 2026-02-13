use std::net::ToSocketAddrs;

/// Resolve a domain to its IPv4 address
pub fn resolve_ipv4(domain: &str) -> Option<String> {
    let addr = format!("{}:80", domain);
    match addr.to_socket_addrs() {
        Ok(addrs) => {
            for a in addrs {
                if a.is_ipv4() {
                    return Some(a.ip().to_string());
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Get the list of GitHub domains from embedded JSON
pub fn get_github_domains() -> Vec<String> {
    let json_str = include_str!("../assets/domains.json");
    serde_json::from_str(json_str).unwrap_or_default()
}

/// Resolve all GitHub domains and return (ip, domain) pairs
pub fn fetch_hosts(domains: &[String]) -> Vec<(String, String)> {
    let mut hosts = Vec::new();
    for domain in domains {
        if let Some(ip) = resolve_ipv4(domain) {
            hosts.push((ip, domain.clone()));
        }
    }
    hosts
}

/// Format hosts entries into a text string
pub fn format_hosts_text(hosts: &[(String, String)], now: &str) -> String {
    let mut result = String::from("# fetch-github-hosts begin\n");
    for (ip, domain) in hosts {
        result.push_str(&format!("{:<28}{}\n", ip, domain));
    }
    result.push_str(&format!("# last fetch time: {}\n", now));
    result.push_str("# update url: https://hosts.gitcdn.top/hosts.txt\n");
    result.push_str("# fetch-github-hosts end\n\n");
    result
}
