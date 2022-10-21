use std::collections::HashMap;

use crate::audio_vec::{mod_int::ModInt998244353, ntt::Ntt, AudioVec};

use self::card_voice::CardVoiceIndex;

pub mod card_voice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InspectPoint {
    pub using_voice: CardVoiceIndex,
    pub delay: isize,
}

/// 損失関数のオブジェクト
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loss {
    /// 88 個の読み札の読み上げ音声
    card_voices: HashMap<CardVoiceIndex, AudioVec>,
    flipped_card_voices: HashMap<CardVoiceIndex, AudioVec>,
    partial_card_voice_norm: HashMap<InspectPoint, ModInt998244353>,
    /// 数論変換のための前計算オブジェクト
    ntt: Ntt,
}

impl Loss {
    pub fn new(card_voices: HashMap<CardVoiceIndex, AudioVec>, problem_voice_len: usize) -> Self {
        let mut partial_card_voice_norm = HashMap::new();
        for (&using, voice) in card_voices.iter() {
            let voice_len = voice.len() as isize;
            for delay in -voice_len..problem_voice_len as isize {
                let delayed = voice.delayed(delay);
                partial_card_voice_norm.insert(
                    InspectPoint {
                        using_voice: using,
                        delay,
                    },
                    delayed.squared_norm(),
                );
            }
        }
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
            partial_card_voice_norm,
            ntt: Ntt::new(),
        }
    }

    /// 2 乗ノルムを用いた損失関数
    ///
    /// `problem_voice` は `card_voices` のうちからいくつかが選ばれて, 時間をずらして重ね合わせたもの
    #[inline]
    pub fn evaluate(&self, problem_voice: &AudioVec, point: InspectPoint) -> u32 {
        (problem_voice.squared_norm()
            - ModInt998244353::new(2)
                * problem_voice
                    .convolution(&self.flipped_card_voices[&point.using_voice], &self.ntt)
                    [point.delay as usize]
            + self.partial_card_voice_norm[&InspectPoint {
                delay: problem_voice.len() as isize - point.delay - 1,
                ..point
            }]
            - self.partial_card_voice_norm[&InspectPoint {
                delay: -point.delay - 1,
                ..point
            }])
            .as_u32()
    }

    pub fn find_points(&self, solutions: usize, problem_voice: &AudioVec) -> Vec<InspectPoint> {
        let points_by_loss: Vec<_> = {
            let mut points: Vec<_> = self
                .card_voices
                .iter()
                .flat_map(|(&using, card_voice)| {
                    (-(card_voice.len() as isize)..problem_voice.len() as isize).map(move |delay| {
                        InspectPoint {
                            using_voice: using,
                            delay,
                        }
                    })
                })
                .collect();
            points.sort_by_key(|&p| self.evaluate(problem_voice, p));
            points
        };
        let first_answer = points_by_loss[..solutions].to_vec();

        // この最初に見つけた解が問題に一致するかどうか検算
        if self.validate(problem_voice, &first_answer) {
            return first_answer;
        }

        // 違うようなので, 最初の解から 1 つだけ取り除いて別の解を探す
        for &next_candidate in &points_by_loss[solutions..] {
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
