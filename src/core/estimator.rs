use std::convert::TryFrom;
use std::thread;
use std::time::{Duration, Instant};

use rand::Rng;

use crate::core::ProxyServer;

const TEST_URIS: [&str; 3] = [
    "https://www.google.com",
    "https://www.twitter.com",
    "https://www.instagram.com",
];

const ROUNDS: usize = 5;

#[derive(Clone)]
pub struct Estimator<'a> {
    pub proxy_server: &'a ProxyServer
}

impl<'a> Estimator<'a>
    where 'a: 'static
{
    pub fn start(self) {
        thread::spawn(move || loop {
            self.clone().estimate();
            let secs = rand::thread_rng().gen_range(5..30);
            thread::sleep(Duration::from_secs(secs));
        });
    }

    fn estimate(self) {
        let proxy = reqwest::Proxy::all(&self.proxy_server.format())
            .expect("Invalid proxy server");

        let client = reqwest::blocking::Client::builder()
            .proxy(proxy)
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Can't build a http client");

        let mut total: u128 = 0;

        for _ in 1..ROUNDS {
            for uri in TEST_URIS.iter() {
                let now = Instant::now();
                let result = client.head(*uri).send();
                let elapsed = match result {
                    Ok(_) => now.elapsed().as_millis(),
                    Err(e) => {
                        println!("{:?}", e);
                        10000
                    }
                };
                total = total + elapsed;
            }

            let millis = rand::thread_rng().gen_range(100..901);
            thread::sleep(Duration::from_millis(millis));
        }

        let x = u128::try_from(ROUNDS * TEST_URIS.len()).unwrap();
        let y = total / x;

        let rating = self.proxy_server.latency_guard();
        rating.set((rating.get() * 3 + y * 7) / 10)
    }
}
