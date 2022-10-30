use std::{cell::RefCell, collections::HashMap};

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
    pub delay: isize,
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
    convolution_cache: RefCell<HashMap<CardVoiceIndex, Vec<ModInt998244353>>>,
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
            convolution_cache: HashMap::new().into(),
        }
    }

    /// 2 乗ノルムを用いた損失関数
    ///
    /// `problem_voice` は `card_voices` のうちからいくつかが選ばれて, 時間をずらして重ね合わせたもの
    #[inline]
    pub fn evaluate(&self, problem_voice: &AudioVec, point: InspectPoint) -> u32 {
        let mut borrowed_cache = self.convolution_cache.borrow_mut();
        let convolution = borrowed_cache.entry(point.using_voice).or_insert_with(|| {
            problem_voice.convolution(&self.flipped_card_voices[&point.using_voice], &self.ntt)
        });
        let convolution_at = if point.delay < 0 {
            ModInt998244353::zero()
        } else {
            convolution
                .get(point.delay as usize)
                .copied()
                .unwrap_or_else(ModInt998244353::zero)
        };
        (problem_voice.squared_norm() - ModInt998244353::new(2) * convolution_at
            + self.precalc.get(
                point.using_voice,
                problem_voice.len() as isize - point.delay - 1,
            )
            - self.precalc.get(point.using_voice, -point.delay - 1))
        .as_u32()
    }

    pub fn find_points(&mut self, solutions: usize, problem_voice: &AudioVec) -> Vec<InspectPoint> {
        let points_by_loss: Vec<_> = {
            let points = self.card_voices.iter().flat_map(|(&using, card_voice)| {
                let stride = card_voice.len() - problem_voice.len();
                (-(stride as isize)..0).map(move |delay| InspectPoint {
                    using_voice: using,
                    delay,
                })
            });
            let mut point_scores: Vec<_> = points
                .map(|p| (p, self.evaluate(problem_voice, p)))
                .collect();
            point_scores.sort_unstable_by_key(|&p| p.1);
            point_scores
        };
        let first_answer: Vec<_> = points_by_loss[..solutions]
            .iter()
            .map(|(p, _)| p)
            .copied()
            .collect();

        info!("first answer is: {:?}", first_answer);

        // この最初に見つけた解が問題に一致するかどうか検算
        if self.validate(problem_voice, &first_answer) {
            return first_answer;
        }

        // 違うようなので, 最初の解から 1 つだけ取り除いて別の解を探す
        for &(next_candidate, _) in &points_by_loss[solutions..] {
            for to_remove in 0..first_answer.len() {
                let next_answer = {
                    let mut list = first_answer.clone();
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

    fn validate(&self, problem_voice: &AudioVec, first_answer: &[InspectPoint]) -> bool {
        let mut composed = AudioVec::default();
        composed.resize(problem_voice.len());
        for &InspectPoint { using_voice, delay } in first_answer {
            composed.add_assign(&self.card_voices[&using_voice].delayed(delay));
        }
        composed.clip();
        const THRESHOLD: u32 = 100;
        problem_voice.sub(&composed).squared_norm().as_u32() < THRESHOLD
    }
}
