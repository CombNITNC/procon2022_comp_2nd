use std::collections::HashMap;

use log::info;
use num::Zero;

use crate::{
    audio_vec::{mod_int::ModInt998244353, ntt::Ntt, AudioVec},
    precalc::Precalculation,
};

use self::card_voice::CardVoiceIndex;

pub mod card_voice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InspectPoint {
    pub using_voice: CardVoiceIndex,
    pub delay: usize,
}

/// 損失関数のオブジェクト
#[derive(Debug)]
pub struct Loss {
    /// 88 個の読み札の読み上げ音声
    card_voices: HashMap<CardVoiceIndex, AudioVec>,
    flipped_card_voices: HashMap<CardVoiceIndex, AudioVec>,
    precalc: Precalculation,
    /// 数論変換のための前計算オブジェクト
    ntt: Ntt,
}

impl Loss {
    pub fn new(card_voices: HashMap<CardVoiceIndex, AudioVec>) -> Self {
        let precalc = Precalculation::new(&card_voices);
        let flipped_card_voices = card_voices
            .iter()
            .map(|(&idx, vec)| {
                let mut cloned = vec.clone();
                cloned.flip();
                (idx, cloned)
            })
            .collect();
        Self {
            card_voices,
            flipped_card_voices,
            precalc,
            ntt: Ntt::new(),
        }
    }

    /// 2 乗ノルムを用いた損失関数
    ///
    /// `problem_voice` は `card_voices` のうちからいくつかが選ばれて, 時間をずらして重ね合わせたもの
    #[inline]
    pub fn evaluate(&self, problem_voice: &AudioVec, using_voice: CardVoiceIndex) -> InspectPoint {
        let convolution =
            problem_voice.convolution(&self.flipped_card_voices[&using_voice], &self.ntt);
        let stride = self.card_voices[&using_voice].len() - problem_voice.len();
        let squared_norm = problem_voice.squared_norm();

        let mut min_score = u32::MAX;
        let mut min_delay = 0;
        for delay in 0..stride {
            let convolution_at = convolution
                .get(delay)
                .copied()
                .unwrap_or_else(ModInt998244353::zero);
            let score = (squared_norm - ModInt998244353::new(2) * convolution_at
                + self
                    .precalc
                    .get(using_voice, problem_voice.len() + delay - 1)
                - self.precalc.get(using_voice, delay - 1))
            .as_u32();
            if score < min_score {
                min_score = score;
                min_delay = delay
            }
        }
        info!("({min_score}, {min_delay}) using {using_voice}");
        InspectPoint {
            using_voice,
            delay: min_delay,
        }
    }

    pub fn find_points(&self, solutions: usize, problem_voice: &AudioVec) -> Vec<InspectPoint> {
        let points_by_loss: Vec<_> = CardVoiceIndex::all()
            .map(|index| self.evaluate(problem_voice, index))
            .collect();
        let first_answer = &points_by_loss[..solutions];

        info!("first answer is: {:?}", first_answer);

        // この最初に見つけた解が問題に一致するかどうか検算
        if self.validate(problem_voice, first_answer) {
            return first_answer.to_vec();
        }

        // 違うようなので, 最初の解から 1 つだけ取り除いて別の解を探す
        for &next_candidate in &points_by_loss[solutions..] {
            for to_remove in 0..first_answer.len() {
                let next_answer = {
                    let mut list = first_answer.to_vec();
                    list[to_remove] = next_candidate;
                    list
                };
                if self.validate(problem_voice, &next_answer) {
                    return next_answer;
                }
            }
        }

        todo!()
    }

    fn validate(&self, problem_voice: &AudioVec, answer: &[InspectPoint]) -> bool {
        let mut composed = AudioVec::default();
        composed.resize(problem_voice.len());
        for &InspectPoint { using_voice, delay } in answer {
            composed.add_assign(&self.card_voices[&using_voice].delayed(delay));
        }
        composed.clip();

        let composed_norm = problem_voice.sub(&composed).squared_norm();
        info!("validation : score of {answer:?} is\n\t{composed_norm:?}");
        let threshold = ModInt998244353::new(10);
        composed_norm < threshold
    }
}
