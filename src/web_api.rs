use crate::day::Day;
use reqwest::{blocking::Client, cookie::Jar, Url};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

const BASE_URL: &str = "https://adventofcode.com/";

pub struct AdventOfCode {
    web_client: Client,
    base_url: String,
    input_cache: PathBuf,
}

impl AdventOfCode {
    #[allow(dead_code)]
    pub fn init(year: &str, session_id: &str, input_cache: &Path) -> Result<Self, Box<dyn Error>> {
        let jar = Jar::default();
        jar.add_cookie_str(
            &format!("session={}", session_id),
            &BASE_URL.parse::<Url>()?,
        );

        Ok(AdventOfCode {
            web_client: Client::builder().cookie_provider(Arc::new(jar)).build()?,
            base_url: format!("{}{}/day/", BASE_URL, year),
            input_cache: input_cache.to_path_buf(),
        })
    }

    fn question_input_path(&self, day: Day) -> PathBuf {
        self.input_cache.join(day.to_filename())
    }

    fn query_question_input(&self, day: Day) -> Result<String, reqwest::Error> {
        self.web_client
            .get(day.to_web_input_path(&self.base_url))
            .send()?
            .text()
    }

    #[allow(dead_code)]
    pub fn load_question_input(&self, day: Day) -> String {
        let cache_path = self.question_input_path(day);
        fs::read_to_string(cache_path.clone()).unwrap_or_else(|_| {
            let text = self
                .query_question_input(day)
                .expect("Failed to query question input from server");
            fs::write(cache_path, text.clone()).expect("Failed to write text to file");
            text
        })
    }
}

pub fn load_question_input(year: &str, cookie_path: &str, day: Day) -> String {
    AdventOfCode::init(
        year,
        &fs::read_to_string(cookie_path).expect("Failed to read session id"),
        Path::new("inputs"),
    )
    .expect("Failed to initialize client")
    .load_question_input(day)
}
