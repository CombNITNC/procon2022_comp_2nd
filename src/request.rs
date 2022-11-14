use anyhow::Result;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::audio_vec::owned::Owned;

pub mod mock;
pub mod net;

#[derive(Debug, Deserialize)]
pub struct Match {
    pub problems: u32,
    pub bonus_factor: Vec<f64>,
    pub penalty: u32,
    pub change_penalty: u32,
    pub wrong_penalty: u32,
    pub correct_point: u32,
}

#[derive(Debug, Deserialize)]
pub struct Problem {
    pub id: String,
    pub chunks: u32,
    pub start_at: u64,
    pub time_limit: u64,
    pub data: u32,
}

#[derive(Debug, Deserialize)]
pub struct Chunks {
    pub chunks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Answer {
    pub problem_id: String,
    pub answers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AnswerResponse {
    pub problem_id: String,
    pub answers: Vec<String>,
    pub accepted_at: u64,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid token")]
    InvalidToken,
    #[error("the match not started")]
    AccessTime,
    #[error("invalid request format")]
    Format,
    #[error("file {0} not found")]
    NotFound(String),
    #[error("request body was too large")]
    TooLargeRequest,
    #[error("unknown http error: {0}")]
    Unknown(StatusCode),
}

pub trait Requester {
    fn get_match(&self) -> Result<Match>;

    fn get_problem(&self) -> Result<Problem>;

    fn get_chunks(&self, using_chunks: u8) -> Result<Vec<Owned>>;

    fn post_answer(&self, answer: &Answer) -> Result<AnswerResponse>;
}
