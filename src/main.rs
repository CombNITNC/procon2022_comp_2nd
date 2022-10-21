use std::{collections::HashMap, fs::File, io, path::PathBuf};

use self::{
    audio_vec::AudioVec,
    request::{net::NetRequester, Answer, Requester},
    solve::{card_voice::CardVoiceIndex, Loss},
};

mod audio_vec;
mod request;
mod solve;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    let endpoint = std::env::var("ENDPOINT")?;
    let token = std::env::var("TOKEN")?;
    let debug = std::env::var("DEBUG")?;

    let all_jk = load_all_jk()?;
    const MIN_VOICE_LEN: usize = 48000 / 2;
    let loss = Loss::new(all_jk, MIN_VOICE_LEN);

    if debug.as_str().trim() == "True" {
        todo!()
    } else {
        let requester = NetRequester::new(&endpoint, &token);
        run_solver(&loss, &requester)
    }
}

fn load_all_jk() -> io::Result<HashMap<CardVoiceIndex, AudioVec>> {
    let mut map = HashMap::new();
    for idx in CardVoiceIndex::all() {
        let path: PathBuf = ["assets".into(), "jk".into(), format!("{}.wav", idx)]
            .into_iter()
            .collect();
        let data = wav::read(&mut File::open(path)?)?.1;
        let pcm = data
            .try_into_sixteen()
            .expect("input audio bit-depth must be 16-bit");
        map.insert(idx, AudioVec::from_pcm(&pcm));
    }
    Ok(map)
}

fn run_solver(loss: &Loss, requester: &impl Requester) -> anyhow::Result<()> {
    let problem_info = requester.get_problem()?;
    let chunks = requester.get_chunks(1)?;
    let chunk = &chunks[0];

    let points = loss.find_points(problem_info.data as usize, chunk);

    requester.post_answer(&Answer {
        problem_id: problem_info.id,
        answers: points
            .into_iter()
            .map(|p| p.using_voice.into_answer_string())
            .collect(),
    })?;

    Ok(())
}
