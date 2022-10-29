use log::info;
use procon2022_comp_2nd::{
    load_all_jk,
    request::{mock::MockRequester, net::NetRequester, Answer, Requester},
    solve::Loss,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let endpoint = std::env::var("ENDPOINT")?;
    let token = std::env::var("TOKEN")?;
    let debug = std::env::var("DEBUG")?;

    let all_jk = load_all_jk()?;
    const MIN_VOICE_LEN: usize = 48000 / 2;
    let loss = Loss::new(all_jk, MIN_VOICE_LEN);

    info!("setup complete");

    if debug.as_str().trim() == "True" {
        let requester =
            MockRequester::new(["assets", "sample", "sample_Q_E01"].into_iter().collect());
        run_solver(&loss, &requester)
    } else {
        let requester = NetRequester::new(&endpoint, &token);
        run_solver(&loss, &requester)
    }
}

fn run_solver(loss: &Loss, requester: &impl Requester) -> anyhow::Result<()> {
    let problem_info = requester.get_problem()?;

    info!("got problem: {:?}", problem_info);

    let chunks = requester.get_chunks(1)?;
    let chunk = &chunks[0];

    let points = loss.find_points(problem_info.data as usize, chunk);

    info!("found points: {:?}", points);

    requester.post_answer(&Answer {
        problem_id: problem_info.id,
        answers: points
            .into_iter()
            .map(|p| p.using_voice.into_answer_string())
            .collect(),
    })?;

    Ok(())
}
