use std::{fs::File, path::PathBuf};

use serde::Deserialize;

use crate::audio_vec::owned::Owned;

use super::Requester;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct InformationText {
    nspeech: u8,
    speech: String,
    offset: String,
    nsplit: u8,
    duration: String,
}

impl InformationText {
    fn speeches(&self) -> Vec<String> {
        self.speech.split(',').map(|s| s.to_owned()).collect()
    }

    fn durations(&self) -> Vec<u64> {
        self.duration
            .split(',')
            .map(|duration| duration.trim().parse().unwrap())
            .collect()
    }
}

#[derive(Debug)]
pub struct MockRequester {
    using_path: PathBuf,
    speeches: Vec<String>,
    durations: Vec<u64>,
}

impl MockRequester {
    pub fn new(using_path: PathBuf) -> Self {
        let file = File::open(using_path.join("information.txt")).unwrap();
        let info: InformationText = serde_yaml::from_reader(file).unwrap();
        Self {
            using_path,
            speeches: info.speeches(),
            durations: info.durations(),
        }
    }
}

impl Requester for MockRequester {
    fn get_match(&self) -> anyhow::Result<super::Match> {
        Ok(super::Match {
            problems: 1,
            bonus_factor: vec![2.0, 1.5, 1.0],
            penalty: 10,
            change_penalty: 5,
            wrong_penalty: 10,
            correct_point: 40,
        })
    }

    fn get_problem(&self) -> anyhow::Result<super::Problem> {
        Ok(super::Problem {
            id: self.using_path.display().to_string(),
            chunks: self.durations.len() as u32,
            start_at: 1667005344340,
            time_limit: 20000000,
            data: self.speeches.len() as u32,
        })
    }

    fn get_chunks(&self, _using_chunks: u8) -> anyhow::Result<Vec<Owned>> {
        let data = wav::read(&mut File::open(self.using_path.join("problem1.wav"))?)?.1;
        let pcm = data
            .try_into_sixteen()
            .expect("input audio bit-depth must be 16-bit");
        Ok(vec![Owned::from_pcm(&pcm)])
    }

    fn post_answer(&self, answer: &super::Answer) -> anyhow::Result<super::AnswerResponse> {
        assert_eq!(self.speeches, answer.answers);
        Ok(super::AnswerResponse {
            problem_id: answer.problem_id.clone(),
            answers: answer.answers.clone(),
            accepted_at: 0,
        })
    }
}
