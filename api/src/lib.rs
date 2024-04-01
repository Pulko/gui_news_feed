use std::io;

use reqwest;
use serde::Deserialize;
use thiserror::Error;
use ureq;
use url;

const BASE_URL: &str = "https://newsapi.org/v2/";

#[derive(Error, Debug)]
pub enum NewsApiError {
    #[error("Failed to fetch data form API")]
    RequestFailed(#[from] ureq::Error),
    #[error("Failed to parse data")]
    ParseFailed(#[from] serde_json::Error),
    #[error("Failed to convert response to string")]
    ResponseToStringFailed(#[from] io::Error),
    #[error("Failed to parse URL")]
    UrlParseFailed(#[from] url::ParseError),
    #[error("Failed to prepare URL")]
    UrlPreparingFailed,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Failed to make async request")]
    #[cfg(feature = "async")]
    AsyncRequestFailed(#[from] reqwest::Error),
    #[error("Failed to make async request")]
    #[cfg(not(feature = "async"))]
    AsyncRequestFailed(#[from] reqwest::Error),
}

#[derive(Deserialize, Debug)]
pub struct NewsApiResponse {
    pub status: String,
    pub articles: Vec<Article>,
    pub code: Option<String>,
}

impl NewsApiResponse {
    pub fn get_articles(&self) -> &Vec<Article> {
        &self.articles
    }
}

#[derive(Deserialize, Debug)]
pub struct Article {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
}

impl Article {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn desciption(&self) -> &str {
        match &self.description {
            Some(desc) => desc,
            None => "",
        }
    }
}

impl Default for Article {
    fn default() -> Self {
        Article {
            title: "".to_string(),
            url: "".to_string(),
            description: None,
        }
    }
}

pub struct NewsApi {
    api_key: String,
    endpoint: Endpoint,
    country: Country,
}

pub enum Country {
    Us,
}

impl ToString for Country {
    fn to_string(&self) -> String {
        match self {
            Country::Us => String::from("us"),
        }
    }
}

pub enum Endpoint {
    TopHeadlines,
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Endpoint::TopHeadlines => "top-headlines".to_string(),
        }
    }
}

impl NewsApi {
    pub fn new(api_key: &str) -> Self {
        let api = NewsApi {
            api_key: api_key.to_string(),
            endpoint: Endpoint::TopHeadlines,
            country: Country::Us,
        };

        api
    }

    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = endpoint;
        self
    }

    pub fn country(&mut self, country: Country) -> &mut Self {
        self.country = country;
        self
    }

    fn prepare_url(&self) -> Result<String, NewsApiError> {
        let mut url = url::Url::parse(BASE_URL).map_err(|e| NewsApiError::UrlParseFailed(e))?;

        url.path_segments_mut()
            .map_err(|_| NewsApiError::UrlPreparingFailed)?
            .push(&self.endpoint.to_string());

        let country = format!("country={}", self.country.to_string());
        url.set_query(Some(&country));

        Ok(url.to_string())
    }

    pub fn fetch(&self) -> Result<NewsApiResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let req = ureq::get(&url).set("Authorization", &self.api_key);
        let resp: NewsApiResponse = req.call()?.into_json()?;

        self.parse_resp(resp)
    }

    #[cfg(feature = "async")]
    pub async fn fetch_async(&self) -> Result<NewsApiResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let client = reqwest::Client::new();
        let request = client
            .request(reqwest::Method::GET, url)
            .header("Authorization", &self.api_key)
            .build()
            .map_err(|e| NewsApiError::AsyncRequestFailed(e))?;

        let response: NewsApiResponse = client
            .execute(request)
            .await?
            .json()
            .await
            .map_err(|e| NewsApiError::AsyncRequestFailed(e))?;

        self.parse_resp(response)
    }

    fn parse_resp(&self, response: NewsApiResponse) -> Result<NewsApiResponse, NewsApiError> {
        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(self.map_response_err(response.code)),
        }
    }

    fn map_response_err(&self, code: Option<String>) -> NewsApiError {
        if let Some(code) = code {
            match code.as_str() {
                "apiKeyDisabled" => {
                    return NewsApiError::BadRequest("API key is disabled".to_string())
                }
                "apiKeyExhausted" => {
                    return NewsApiError::BadRequest(
                        "API key has no more requests available".to_string(),
                    )
                }
                "apiKeyInvalid" => {
                    return NewsApiError::BadRequest("API key is invalid".to_string())
                }
                "apiKeyMissing" => {
                    return NewsApiError::BadRequest("API key is missing".to_string())
                }
                "parameterInvalid" => {
                    return NewsApiError::BadRequest("Parameters are invalid".to_string())
                }
                "parametersMissing" => {
                    return NewsApiError::BadRequest("Parameters are missing".to_string())
                }
                "rateLimited" => {
                    return NewsApiError::BadRequest("Request is rate limited".to_string())
                }
                _ => return NewsApiError::BadRequest("Unknown error".to_string()),
            }
        } else {
            return NewsApiError::BadRequest("Unknown error".to_string());
        }
    }
}
