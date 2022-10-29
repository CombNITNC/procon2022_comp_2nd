pub mod audio_vec;
pub mod request;
pub mod solve;

use std::{collections::HashMap, fs::File, io, path::PathBuf};

use log::info;

use self::{
    audio_vec::{mod_int::ModInt998244353, AudioVec},
    solve::card_voice::CardVoiceIndex,
};

pub fn load_all_jk() -> io::Result<HashMap<CardVoiceIndex, AudioVec>> {
    let mut map = HashMap::new();
    for idx in CardVoiceIndex::all() {
        let path: PathBuf = ["assets".into(), "jk".into(), format!("{}.wav", idx)]
            .into_iter()
            .collect();
        let data = wav::read(&mut File::open(&path)?)?.1;
        let pcm = data
            .try_into_sixteen()
            .expect("input audio bit-depth must be 16-bit");
        map.insert(idx, AudioVec::from_pcm(&pcm));
        info!("loaded speech voice: {}", path.display());
    }
    Ok(map)
}

pub fn precalculate(
    card_voices: &HashMap<CardVoiceIndex, AudioVec>,
) -> HashMap<CardVoiceIndex, Vec<ModInt998244353>> {
    let mut partial_card_voice_norm = HashMap::new();
    for (&using, voice) in card_voices.iter() {
        let squared = voice.squared();
        let partial_sum: Vec<_> = squared
            .scan(ModInt998244353::new(0), |acc, x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
        partial_card_voice_norm.insert(using, partial_sum);
        info!("pre-calculated: {:?}", using);
    }
    partial_card_voice_norm
}
