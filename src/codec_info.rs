//! コーデック情報の照会

/// コーデック種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodecType {
    /// AV1
    Av1,
}

impl VideoCodecType {
    /// すべてのコーデック種別を返す
    fn all() -> &'static [Self] {
        &[Self::Av1]
    }
}

/// コーデックごとの情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecInfo {
    /// コーデック種別
    pub codec: VideoCodecType,
    /// デコード情報
    pub decoding: DecodingInfo,
    /// エンコード情報
    pub encoding: EncodingInfo,
}

/// デコード情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingInfo {
    /// デコードが可能か
    pub supported: bool,
    /// ハードウェアアクセラレーションが利用可能か
    pub hardware_accelerated: bool,
}

/// エンコード情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingInfo {
    /// エンコードが可能か
    pub supported: bool,
    /// ハードウェアアクセラレーションが利用可能か
    pub hardware_accelerated: bool,
    /// コーデック固有のプロファイル情報
    pub profiles: EncodingProfiles,
}

/// コーデック固有のエンコードプロファイル情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodingProfiles {
    /// AV1 プロファイル一覧
    Av1(Vec<Av1EncodingProfile>),
}

/// AV1 エンコードプロファイル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Av1EncodingProfile {
    /// Main (8-bit/10-bit 4:2:0)
    Main,
}

/// このバックエンドで利用可能なコーデック情報の一覧を返す
///
/// SVT-AV1 はソフトウェアエンコーダー専用であるため、AV1 のエンコードのみ可能で、
/// デコードとハードウェアアクセラレーションは利用できない。
pub fn supported_codecs() -> Vec<CodecInfo> {
    VideoCodecType::all()
        .iter()
        .map(|&codec| CodecInfo {
            codec,
            decoding: decoding_info(),
            encoding: encoding_info(),
        })
        .collect()
}

/// デコード情報を返す
///
/// SVT-AV1 はエンコーダー専用であるため、デコードは常に非対応。
fn decoding_info() -> DecodingInfo {
    DecodingInfo {
        supported: false,
        hardware_accelerated: false,
    }
}

/// エンコード情報を返す
///
/// SVT-AV1 はソフトウェアエンコーダーであるため、supported は常に true、
/// hardware_accelerated は常に false になる。
/// I420 (8-bit) と I42010 (10-bit) の 4:2:0 に対応しているため Main プロファイルを返す。
fn encoding_info() -> EncodingInfo {
    EncodingInfo {
        supported: true,
        hardware_accelerated: false,
        profiles: EncodingProfiles::Av1(vec![Av1EncodingProfile::Main]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_codecs_returns_one_codec() {
        let codecs = supported_codecs();
        assert_eq!(codecs.len(), 1);
        assert_eq!(codecs[0].codec, VideoCodecType::Av1);
    }

    #[test]
    fn av1_codec_info() {
        let codecs = supported_codecs();
        let av1 = &codecs[0];
        assert_eq!(
            *av1,
            CodecInfo {
                codec: VideoCodecType::Av1,
                decoding: DecodingInfo {
                    supported: false,
                    hardware_accelerated: false,
                },
                encoding: EncodingInfo {
                    supported: true,
                    hardware_accelerated: false,
                    profiles: EncodingProfiles::Av1(vec![Av1EncodingProfile::Main]),
                },
            }
        );
    }
}
