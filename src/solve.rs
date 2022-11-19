use std::collections::HashMap;

use log::info;

use crate::{
    audio_vec::{
        owned::{ntt::Ntt, Owned},
        AudioVec,
    },
    precalc::Precalculation,
};

use self::card_voice::CardVoiceIndex;

pub mod card_voice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InspectPoint {
    pub using_voice: CardVoiceIndex,
    pub delay: isize,
    pub score: u64,
}

/// 損失関数のオブジェクト
#[derive(Debug)]
pub struct Loss {
    /// 88 個の読み札の読み上げ音声
    card_voices: HashMap<CardVoiceIndex, Owned>,
    flipped_card_voices: HashMap<CardVoiceIndex, Owned>,
    precalc: Precalculation,
    /// 数論変換のための前計算オブジェクト
    ntt: (Ntt<924844033>, Ntt<998244353>),
}

impl Loss {
    pub fn new(card_voices: HashMap<CardVoiceIndex, Owned>) -> Self {
        let precalc = Precalculation::new(&card_voices);
        let flipped_card_voices = card_voices
            .iter()
            .map(|(&idx, vec)| (idx, vec.clone().flip().to_owned(vec.len())))
            .collect();
        Self {
            card_voices,
            flipped_card_voices,
            precalc,
            ntt: (Ntt::new(), Ntt::new()),
        }
    }

    /// 2 乗ノルムを用いた損失関数
    ///
    /// `problem_voice` は `card_voices` のうちからいくつかが選ばれて, 時間をずらして重ね合わせたもの
    #[inline]
    pub fn evaluate(&self, problem_voice: &Owned, using_voice: CardVoiceIndex) -> InspectPoint {
        let convolution = problem_voice.convolution(
            &self.flipped_card_voices[&using_voice],
            (&self.ntt.0, &self.ntt.1),
        );
        let stride = (self.card_voices[&using_voice].len() - problem_voice.len()) as isize;
        let squared_norm = problem_voice.squared_norm().as_u64();

        let mut min_score = u64::MAX;
        let mut min_delay = 0;
        for delay in -(problem_voice.len() as isize)..stride {
            // R : using voice
            // T : problem voice length
            // x : problem voice
            // w : how long delayed
            // f(w) = |x - R.delayed(w).clip()|^2
            // = |x|^2 - 2 * x * R.delayed(w).clip() + |R.delayed(w)|^2
            // = |x|^2 - 2 * x * R.delayed(w) + |R.delayed(w)|^2
            // = |x|^2 - 2 * Σ_t (x_t * R_{t - w}) + Σ_{t = 0}^{T - 1} R_{t - w}^2
            // = |x|^2 - 2 * Σ_t (x_t * R_{w - t}.flip()) + Σ_{t = -w}^{T - w - 1} R_t^2
            // = |x|^2 - 2 * x.convolution(R.flip())_w + Σ_{t = 0}^{T - w - 1} R_t^2 - Σ_{t = 0}^{-w - 1} R_t^2
            let convolution_at = (0 <= delay)
                .then(|| convolution.get(delay as usize).copied())
                .flatten()
                .unwrap_or_default();
            let score = squared_norm - 2 * convolution_at.as_u64()
                + self
                    .precalc
                    .get(using_voice, (problem_voice.len() as isize) - delay - 1)
                    .as_u64()
                - self.precalc.get(using_voice, -delay - 1).as_u64();
            if score < min_score {
                min_score = score;
                min_delay = delay;
            }
        }
        info!("({min_score}, {min_delay}) using {using_voice}");
        InspectPoint {
            using_voice,
            delay: min_delay,
            score: min_score,
        }
    }

    pub fn find_points(&self, problem_voice: &Owned) -> Vec<InspectPoint> {
        let mut points_by_loss: Vec<_> = CardVoiceIndex::all()
            .map(|index| self.evaluate(problem_voice, index))
            .collect();
        points_by_loss.sort_unstable_by_key(|point| point.score);
        points_by_loss
    }

    pub fn validate(&self, problem_voice: &Owned, answer: &[InspectPoint]) -> bool {
        let len = problem_voice.len();
        let mut composed = Owned::new();
        for &InspectPoint {
            using_voice, delay, ..
        } in answer
        {
            composed = composed
                .add(self.card_voices[&using_voice].clone().delay(delay))
                .to_owned(len);
        }

        let composed_norm = problem_voice
            .clone()
            .sub(composed.clip(len))
            .to_owned(len)
            .squared_norm()
            .as_u64()
            / len as u64;
        info!("validation : score of {answer:?} is\n\t{composed_norm:?}");
        let threshold = 10;
        composed_norm < threshold
    }
}

#[test]
fn validate_e01() -> anyhow::Result<()> {
    use crate::{load_all_jk, MockRequester, Requester};

    // E01 + E02 + E03 = Q_E01
    let loss = Loss::new(load_all_jk()?);

    let requester = MockRequester::new(["assets", "sample", "sample_Q_E01"].into_iter().collect());
    let chunks = requester.get_chunks(1)?;
    let chunk = &chunks[0];

    assert!(loss.validate(
        chunk,
        &[
            InspectPoint {
                delay: 0,
                score: 0,
                using_voice: CardVoiceIndex::new(0)
            },
            InspectPoint {
                delay: 0,
                score: 0,
                using_voice: CardVoiceIndex::new(1)
            },
            InspectPoint {
                delay: 0,
                score: 0,
                using_voice: CardVoiceIndex::new(2)
            }
        ]
    ));

    Ok(())
}
