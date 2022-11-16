use log::info;

use crate::{
    precalc::load_all_jk,
    request::{mock::MockRequester, net::NetRequester, Answer},
    solve::Loss,
};

use self::request::Requester;

mod audio_vec;
mod precalc;
mod request;
mod solve;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let endpoint = std::env::var("ENDPOINT")?;
    let token = std::env::var("TOKEN")?;
    let debug = std::env::var("DEBUG")?;

    let all_jk = load_all_jk()?;
    let loss = Loss::new(all_jk);

    info!("setup complete");

    if debug.as_str().trim() == "True" {
        let requester =
            MockRequester::new(["assets", "sample", "sample_Q_E01"].into_iter().collect());
        run_solver(loss, &requester)
    } else {
        let requester = NetRequester::new(&endpoint, &token);
        run_solver(loss, &requester)
    }
}

fn run_solver(loss: Loss, requester: &impl Requester) -> anyhow::Result<()> {
    let problem_info = requester.get_problem()?;

    info!("got problem: {:?}", problem_info);

    let chunks = requester.get_chunks(1)?;
    let chunk = &chunks[0];

    let solutions = problem_info.data as usize;
    let points_by_loss = loss.find_points(chunk);

    let first_answer = &points_by_loss[..solutions];

    info!("first answer is: {:?}", first_answer);

    // この最初に見つけた解が問題に一致するかどうか検算
    if loss.validate(chunk, first_answer) {
        requester.post_answer(&Answer {
            problem_id: problem_info.id,
            answers: first_answer
                .iter()
                .map(|p| p.using_voice.into_answer_string())
                .collect(),
        })?;
        return Ok(());
    }

    // 違うようなので, 最初の解から 1 つだけ取り除いて別の解を探す
    for &next_candidate in &points_by_loss[solutions..] {
        for to_remove in 0..first_answer.len() {
            let next_answer = {
                let mut list = first_answer.to_vec();
                list[to_remove] = next_candidate;
                list
            };
            if loss.validate(chunk, &next_answer) {
                requester.post_answer(&Answer {
                    problem_id: problem_info.id,
                    answers: next_answer
                        .into_iter()
                        .map(|p| p.using_voice.into_answer_string())
                        .collect(),
                })?;
                return Ok(());
            }
        }
    }

    todo!()
}
