use std::collections::HashMap;

use crate::audio_vec::{mod_int::ModInt998244353, ntt::Ntt, AudioVec};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InspectPoint {
    pub using_voice: usize,
    pub delay: isize,
}

/// 損失関数のオブジェクト
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loss {
    /// 88 個の読み札の読み上げ音声
    card_voices: Vec<AudioVec>,
    partial_card_voice_norm: HashMap<InspectPoint, ModInt998244353>,
    /// 数論変換のための前計算オブジェクト
    ntt: Ntt,
}

impl Loss {
    pub fn new(card_voices: Vec<AudioVec>, problem_voice_len: usize) -> Self {
        let mut partial_card_voice_norm = HashMap::new();
        for (using, voice) in card_voices.iter().enumerate() {
            let voice_len = voice.len() as isize;
            for delay in -voice_len..problem_voice_len as isize {
                let delayed = card_voices[using].delayed(delay);
                partial_card_voice_norm.insert(
                    InspectPoint {
                        using_voice: using,
                        delay,
                    },
                    delayed.squared_norm(),
                );
            }
        }
        Self {
            card_voices,
            partial_card_voice_norm,
            ntt: Ntt::new(),
        }
    }

    /// 2 乗ノルムを用いた損失関数
    ///
    /// `problem_voice` は `card_voices` のうちからいくつかが選ばれて, 時間をずらして重ね合わせたもの
    pub fn evaluate(&self, problem_voice: &AudioVec, point: InspectPoint) -> u32 {
        (problem_voice.squared_norm()
            - ModInt998244353::new(2)
                * problem_voice.convolution(&self.card_voices[point.using_voice], &self.ntt)
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
}
