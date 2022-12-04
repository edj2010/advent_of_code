use reqwest::{blocking::Client, cookie::Jar, Url};
use std::error::Error;
use std::sync::Arc;

const BASE_URL: &str = "https://adventofcode.com/";

pub struct AdventOfCode {
    web_client: Client,
    base_url: String,
}

impl AdventOfCode {
    #[allow(dead_code)]
    pub fn init(year: &str, session_id: &str) -> Result<Self, Box<dyn Error>> {
        let jar = Jar::default();
        jar.add_cookie_str(
            &format!("session={}", session_id),
            &BASE_URL.parse::<Url>()?,
        );

        Ok(AdventOfCode {
            web_client: Client::builder().cookie_provider(Arc::new(jar)).build()?,
            base_url: format!("{}{}/day/", BASE_URL, year),
        })
    }

    #[allow(dead_code)]
    pub fn query_question_input(&self, question: usize) -> Result<String, Box<dyn Error>> {
        Ok(self
            .web_client
            .get(format!("{}{}/input", self.base_url, question))
            .send()?
            .text()?)
    }
}
