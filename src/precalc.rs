use std::{collections::HashMap, fs::File, io, path::PathBuf};

use log::info;

use crate::{
    audio_vec::owned::{pixel::Pixel, Owned},
    solve::card_voice::CardVoiceIndex,
};

pub fn load_all_jk() -> io::Result<HashMap<CardVoiceIndex, Owned>> {
    let mut map = HashMap::new();
    for idx in CardVoiceIndex::all() {
        let path: PathBuf = ["assets".into(), "jk".into(), format!("{}.wav", idx)]
            .into_iter()
            .collect();
        let data = wav::read(&mut File::open(&path)?)?.1;
        let pcm = data
            .try_into_sixteen()
            .expect("input audio bit-depth must be 16-bit");
        map.insert(idx, Owned::from_pcm(&pcm));
        info!("loaded speech voice: {}", path.display());
    }
    Ok(map)
}

/// 読み札の音声ごとに, その音声を 2 乗したものの累積和を前計算して格納する.
///
/// すなわち, `f(x) = Σ_{t = 0}^{x} R_t^2` を提供する.
#[derive(Debug)]
pub struct Precalculation {
    table: HashMap<CardVoiceIndex, Vec<Pixel>>,
}

impl Precalculation {
    pub fn new(card_voices: &HashMap<CardVoiceIndex, Owned>) -> Precalculation {
        let mut table = HashMap::new();
        for (&using, voice) in card_voices.iter() {
            let squared = voice.squared();
            let partial_sum: Vec<_> = squared
                .scan(Pixel::default(), |acc, x| {
                    *acc += x;
                    Some(*acc)
                })
                .collect();
            table.insert(using, partial_sum);
            info!("pre-calculated: {:?}", using);
        }
        Self { table }
    }

    pub fn get(&self, using: CardVoiceIndex, delay: isize) -> Pixel {
        (0 <= delay)
            .then(|| self.table[&using].get(delay as usize).copied())
            .flatten()
            .unwrap_or_default()
    }
}
