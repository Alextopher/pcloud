use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
struct CreateRedirct {
    target: String,
    // How many seconds should the redirect be valid for?
    destory_after: u32,
    // How many times can the redirect be used?
    remaining_usage: Option<i64>,
}

#[derive(Deserialize, Debug)]
struct RedirectReponse {
    key: String,
}

// Parses a suffixed duration string to a number of seconds
fn parse_duration(duration: String) -> Result<u32, ()> {
    // Format: (?i)[0-9]+[smhdwy]
    let re = regex::Regex::new(r"(?i)([0-9]+)([smhdwy])").unwrap();

    match re.captures(duration.as_str()) {
        Some(caps) => {
            let num = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
            let unit = caps.get(2).unwrap().as_str();

            match unit {
                "s" => Ok(num),
                "m" => Ok(num * 60),
                "h" => Ok(num * 60 * 60),
                "d" => Ok(num * 60 * 60 * 24),
                "w" => Ok(num * 60 * 60 * 24 * 7),
                "y" => Ok(num * 60 * 60 * 24 * 365),
                _ => Err(()),
            }
        }
        None => Err(()),
    }
}

pub fn create_redirect(host: String, url: url::Url, usages: Option<u32>, duration: String) {
    let destory_after = parse_duration(duration).unwrap();
    let remaining_usage = match usages {
        Some(usages) => Some(usages as i64),
        None => None,
    };

    let redirect = CreateRedirct {
        target: url.to_string(),
        destory_after,
        remaining_usage,
    };

    let client = reqwest::blocking::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let res = client
        .post(format!("{host}/r"))
        .json(&redirect)
        .header("Content-Type", "application/json")
        .send();

    match res {
        Ok(res) => {
            let redirect_response = res.json::<RedirectReponse>().unwrap();
            println!("{host}/r/{}", redirect_response.key);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
