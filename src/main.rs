use std::{collections::HashMap, fs::File, io, path::PathBuf};

use self::{audio_vec::AudioVec, solve::card_voice::CardVoiceIndex};

mod audio_vec;
mod solve;

fn main() -> io::Result<()> {
    let all_jk = load_all_jk()?;

    Ok(())
}

fn load_all_jk() -> io::Result<HashMap<CardVoiceIndex, AudioVec>> {
    let mut map = HashMap::new();
    for idx in CardVoiceIndex::all() {
        let path: PathBuf = ["assets".into(), "jk".into(), idx.to_string()]
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
