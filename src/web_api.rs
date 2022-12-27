use reqwest::{blocking::Client, cookie::Jar, Url};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

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

    fn question_input_path(&self, question: usize) -> Result<PathBuf, Box<dyn Error>> {
        let base_name = PathBuf::from_str(&format!("{}.in", question))?;
        Ok(self.input_cache.join(base_name))
    }

    #[allow(dead_code)]
    pub fn query_question_input(&self, question: usize) -> Result<String, Box<dyn Error>> {
        let cache_path = self.question_input_path(question)?;
        if cache_path.is_file() {
            Ok(fs::read_to_string(cache_path)?)
        } else {
            let text = self
                .web_client
                .get(format!("{}{}/input", self.base_url, question))
                .send()?
                .text()?;
            fs::write(cache_path, text.clone())?;
            Ok(text)
        }
    }
}
