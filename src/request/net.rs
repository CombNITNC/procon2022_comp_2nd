use std::io::Cursor;

use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
    Url,
};

use crate::audio_vec::AudioVec;

use super::Requester;

pub struct NetRequester {
    endpoint: Url,
    client: Client,
}

impl NetRequester {
    pub fn new(endpoint: &str, token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "procon-token",
            HeaderValue::from_str(token).expect("invalid character in token"),
        );
        Self {
            endpoint: Url::parse(endpoint).expect("invalid endpoint"),
            client: Client::builder()
                .default_headers(headers)
                .build()
                .expect("invalid header map"),
        }
    }
}

impl Requester for NetRequester {
    fn get_match(&self) -> anyhow::Result<super::Match> {
        let res = self
            .client
            .get(self.endpoint.join("/match").unwrap())
            .send()?;
        let json: super::Match = res.json()?;
        Ok(json)
    }

    fn get_problem(&self) -> anyhow::Result<super::Problem> {
        let res = self
            .client
            .get(self.endpoint.join("/problem").unwrap())
            .send()?;
        let json: super::Problem = res.json()?;
        Ok(json)
    }

    fn get_chunks(&self, using_chunks: u8) -> anyhow::Result<Vec<AudioVec>> {
        let chunks_url = self.endpoint.join("/problem/chunks").unwrap();
        let res = self
            .client
            .post(chunks_url.clone())
            .query(&[("n", using_chunks)])
            .send()?;
        let json: super::Chunks = res.json()?;
        json.chunks
            .into_iter()
            .map(|chunk| {
                let res = self.client.get(chunks_url.join(&chunk).unwrap()).send()?;
                let bytes = res.bytes()?;
                let mut cursor = Cursor::new(bytes);
                let pcm = wav::read(&mut cursor)?
                    .1
                    .try_into_sixteen()
                    .expect("expected 16-bit depth wav file");
                Ok(AudioVec::from_pcm(&pcm))
            })
            .collect()
    }

    fn post_answer(&self, answer: &super::Answer) -> anyhow::Result<super::AnswerResponse> {
        let res = self
            .client
            .post(self.endpoint.join("/problem").unwrap())
            .json(answer)
            .send()?;
        let json: super::AnswerResponse = res.json()?;
        Ok(json)
    }
}
