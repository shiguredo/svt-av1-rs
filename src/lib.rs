//! [SVT-AV1] の Rust バインディング
//!
//! [SVT-AV1]: https://gitlab.com/AOMediaCodec/SVT-AV1
#![deny(missing_docs)]

use std::{mem::MaybeUninit, num::NonZeroUsize, sync::Mutex};

mod codec_info;
mod sys;

pub use codec_info::*;

/// ビルド時に参照したリポジトリ URL
pub const BUILD_REPOSITORY: &str = sys::BUILD_METADATA_REPOSITORY;

/// ビルド時に参照したリポジトリのバージョン（タグ）
pub const BUILD_VERSION: &str = sys::BUILD_METADATA_VERSION;

const ENV_KEY_SVT_LOG: &str = "SVT_LOG";
const ENV_VALUE_SVT_LOG_LEVEL: &str = "1"; // 1 は error (必要に応じて調整する）

// SVT-AV1 のエンコーダー初期化処理を複数スレッドで同時に実行すると
// 大量のエラーログが出力されることがあるのでロックを使用している
static GLOBAL_LOCK: Mutex<()> = Mutex::new(());

/// エラー
#[derive(Debug)]
pub struct Error {
    function: &'static str,
    code: sys::EbErrorType,
}

impl Error {
    fn check(code: sys::EbErrorType, function: &'static str) -> Result<(), Self> {
        if code == sys::EbErrorType_EB_ErrorNone {
            Ok(())
        } else {
            Err(Self { function, code })
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}() failed: code={}", self.function, self.code)
    }
}

impl std::error::Error for Error {}

/// カラーフォーマット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// YUV 4:2:0 planar 8-bit (3 プレーン: Y, U, V)
    I420,
    /// YUV 4:2:0 planar 10-bit (3 プレーン: Y, U, V、各ピクセル 2 バイト)
    I42010,
}

/// 入力フレームデータ
#[derive(Debug)]
pub enum FrameData<'a> {
    /// I420 (3 プレーン: Y, U, V)
    I420 {
        /// Y プレーン
        y: &'a [u8],
        /// U プレーン
        u: &'a [u8],
        /// V プレーン
        v: &'a [u8],
    },
    /// I42010 (3 プレーン: Y, U, V、各ピクセル 2 バイト)
    I42010 {
        /// Y プレーン (16-bit リトルエンディアン)
        y: &'a [u8],
        /// U プレーン (16-bit リトルエンディアン)
        u: &'a [u8],
        /// V プレーン (16-bit リトルエンディアン)
        v: &'a [u8],
    },
}

/// レート制御モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcMode {
    /// CQP/CRF (Constant Rate Factor) - 品質ベース
    CqpOrCrf,
    /// VBR (Variable Bit Rate)
    Vbr,
    /// CBR (Constant Bit Rate)
    Cbr,
}

/// カラープライマリ (H.273)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    /// BT.709 / sRGB
    Bt709,
    /// 未指定
    Unspecified,
    /// BT.470M
    Bt470M,
    /// BT.470BG
    Bt470Bg,
    /// BT.601
    Bt601,
    /// SMPTE 240M
    Smpte240,
    /// Generic Film
    GenericFilm,
    /// BT.2020 / BT.2100
    Bt2020,
    /// XYZ
    Xyz,
    /// SMPTE 431 (DCI-P3)
    Smpte431,
    /// SMPTE 432 (Display P3)
    Smpte432,
    /// EBU 3213
    Ebu3213,
}

/// 伝達特性 (H.273)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferCharacteristics {
    /// BT.709
    Bt709,
    /// 未指定
    Unspecified,
    /// BT.470M
    Bt470M,
    /// BT.470BG
    Bt470Bg,
    /// BT.601
    Bt601,
    /// SMPTE 240M
    Smpte240,
    /// Linear
    Linear,
    /// IEC 61966
    Iec61966,
    /// BT.1361
    Bt1361,
    /// sRGB
    Srgb,
    /// BT.2020 10-bit
    Bt202010Bit,
    /// BT.2020 12-bit
    Bt202012Bit,
    /// SMPTE 2084 (PQ)
    Pq,
    /// SMPTE 428
    Smpte428,
    /// HLG
    Hlg,
}

/// マトリクス係数 (H.273)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixCoefficients {
    /// Identity
    Identity,
    /// BT.709
    Bt709,
    /// 未指定
    Unspecified,
    /// FCC
    Fcc,
    /// BT.470BG
    Bt470Bg,
    /// BT.601
    Bt601,
    /// SMPTE 240M
    Smpte240,
    /// YCgCo
    Ycgco,
    /// BT.2020 Non-Constant Luminance
    Bt2020Ncl,
    /// BT.2020 Constant Luminance
    Bt2020Cl,
    /// SMPTE 2085
    Smpte2085,
    /// Chromaticity-derived Non-Constant Luminance
    ChromatNcl,
    /// Chromaticity-derived Constant Luminance
    ChromatCl,
    /// ICtCp
    Ictcp,
}

/// カラーレンジ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRange {
    /// スタジオレンジ (Y: 16-235, UV: 16-240)
    Studio,
    /// フルレンジ (0-255)
    Full,
}

/// クロマサンプル位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSamplePosition {
    /// 不明
    Unknown,
    /// 垂直 (left)
    Vertical,
    /// コロケーテッド (top-left)
    Colocated,
}

/// HDR マスタリングディスプレイ情報
#[derive(Debug, Clone, Copy)]
pub struct MasteringDisplayInfo {
    /// 赤の色度座標 (x, y) — 0.16 固定小数点
    pub r: (u16, u16),
    /// 緑の色度座標 (x, y) — 0.16 固定小数点
    pub g: (u16, u16),
    /// 青の色度座標 (x, y) — 0.16 固定小数点
    pub b: (u16, u16),
    /// 白色点の色度座標 (x, y) — 0.16 固定小数点
    pub white_point: (u16, u16),
    /// 最大輝度 (cd/m^2) — 24.8 固定小数点
    pub max_luminance: u32,
    /// 最小輝度 (cd/m^2) — 18.14 固定小数点
    pub min_luminance: u32,
}

/// HDR コンテンツ輝度レベル
#[derive(Debug, Clone, Copy)]
pub struct ContentLightLevel {
    /// 最大コンテンツ輝度レベル (cd/m^2)
    pub max_cll: u16,
    /// 最大フレーム平均輝度レベル (cd/m^2)
    pub max_fall: u16,
}

/// 品質チューニングメトリクス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tune {
    /// VQ (Visual Quality)
    Vq,
    /// PSNR
    Psnr,
    /// SSIM
    Ssim,
    /// IQ (Image Quality) — v4.0.0 以降
    Iq,
    /// MS-SSIM
    MsSsim,
}

/// イントラリフレッシュタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntraRefreshType {
    /// Forward Key Frame Refresh (Open GOP)
    FwdkfRefresh,
    /// Key Frame Refresh (Closed GOP)
    KfRefresh,
}

/// フレームタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureType {
    /// インターフレーム
    Inter,
    /// Alt-Ref フレーム
    AltRef,
    /// イントラオンリーフレーム
    IntraOnly,
    /// キーフレーム
    Key,
    /// 非参照フレーム
    NonRef,
    /// Forward キーフレーム
    ForwardKey,
    /// Show Existing フレーム
    ShowExisting,
    /// スイッチフレーム
    Switch,
    /// 不明
    Unknown,
}

/// フレームごとのエンコードオプション
#[derive(Debug, Clone, Default)]
pub struct EncodeOptions {
    /// キーフレームを強制する
    pub force_keyframe: bool,
}

/// エンコーダーに指定する設定
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// 入出力画像の幅
    pub width: usize,

    /// 入出力画像の高さ
    pub height: usize,

    /// 入力画像フォーマット
    pub color_format: ColorFormat,

    /// FPS の分子
    pub fps_numerator: usize,

    /// FPS の分母
    pub fps_denominator: usize,

    /// エンコードビットレート (bps 単位)
    pub target_bit_rate: usize,

    /// レート制御モード
    pub rate_control_mode: RcMode,

    /// エンコードプリセット (0-13, 0=最高品質・最遅, 13=最低品質・最速)
    pub enc_mode: u8,

    /// 最小量子化パラメータ (0-63)
    pub min_qp_allowed: Option<u8>,

    /// 最大量子化パラメータ (0-63)
    pub max_qp_allowed: Option<u8>,

    /// 量子化パラメータ (0-63, CRF モード時に使用)
    pub qp: Option<u8>,

    /// キーフレーム間隔 (フレーム数)
    pub intra_period_length: Option<NonZeroUsize>,

    /// タイル列数 (並列処理用)
    pub tile_columns: Option<NonZeroUsize>,

    /// タイル行数 (並列処理用)
    pub tile_rows: Option<NonZeroUsize>,

    /// 先読み距離 (フレーム数)
    pub look_ahead_distance: Option<usize>,

    /// シーンチェンジ検出
    pub scene_change_detection: bool,

    /// カラープライマリ
    pub color_primaries: Option<ColorPrimaries>,

    /// 伝達特性
    pub transfer_characteristics: Option<TransferCharacteristics>,

    /// マトリクス係数
    pub matrix_coefficients: Option<MatrixCoefficients>,

    /// カラーレンジ
    pub color_range: Option<ColorRange>,

    /// クロマサンプル位置
    pub chroma_sample_position: Option<ChromaSamplePosition>,

    /// HDR マスタリングディスプレイ情報
    pub mastering_display: Option<MasteringDisplayInfo>,

    /// HDR コンテンツ輝度レベル
    pub content_light_level: Option<ContentLightLevel>,

    /// CBR 初期バッファレベル (ミリ秒)
    pub starting_buffer_level_ms: Option<u64>,

    /// CBR 目標バッファレベル (ミリ秒)
    pub optimal_buffer_level_ms: Option<u64>,

    /// CBR 最大バッファサイズ (ミリ秒)
    pub maximum_buffer_size_ms: Option<u64>,

    /// 品質チューニングメトリクス
    pub tune: Option<Tune>,

    /// per-frame の品質メトリクス計算を有効にする
    pub stat_report: bool,

    /// デコーダー速度最適化 (0=無効, 1=中程度, 2=最大)
    pub fast_decode: Option<u8>,

    /// フィルムグレインデノイズ強度 (0=無効, 1-50=強度)
    pub film_grain_denoise_strength: Option<u32>,

    /// フィルムグレインデノイズ適用 (0=なし, 1=フル適用)
    pub film_grain_denoise_apply: Option<u8>,

    /// 適応的フィルムグレインブロックサイズ
    pub adaptive_film_grain: Option<bool>,

    /// シャープネス (-7 to 7)
    pub sharpness: Option<i8>,

    /// 適応的量子化モード (0=CQP, 1=variance, 2=CRF)
    pub aq_mode: Option<u8>,

    /// 最大ビットレート (bps 単位、Capped CRF 用)
    pub max_bit_rate: Option<usize>,

    /// スクリーンコンテンツモード (0=無効, 1=検出, 2=強制, 3=拡張検出)
    pub screen_content_mode: Option<u8>,

    /// イントラリフレッシュタイプ
    pub intra_refresh_type: Option<IntraRefreshType>,

    /// RTC (Real-Time Coding) モード
    pub rtc: Option<bool>,

    /// テンポラルレイヤーの階層数 (0-5, テンポラルレイヤー数 = hierarchical_levels + 1)
    pub hierarchical_levels: Option<u32>,

    /// S-frame の挿入間隔 (フレーム数)
    pub sframe_dist: Option<i32>,

    /// S-frame の挿入モード (1=STRICT, 2=NEAREST)
    pub sframe_mode: Option<u32>,

    /// スーパーレゾリューションモード (0=無効, 1=固定, 2=ランダム, 3=QThreshold, 4=自動)
    pub superres_mode: Option<u8>,

    /// スーパーレゾリューション ダウンスケール分母 (8-16, 8=スケーリングなし)
    pub superres_denom: Option<u8>,

    /// キーフレーム用スーパーレゾリューション ダウンスケール分母 (8-16)
    pub superres_kf_denom: Option<u8>,

    /// スーパーレゾリューション QThreshold モードの閾値
    pub superres_qthres: Option<u8>,

    /// キーフレーム用スーパーレゾリューション QThreshold
    pub superres_kf_qthres: Option<u8>,

    /// ロスレスエンコード
    pub lossless: Option<bool>,

    /// Variance Boost の有効化
    pub enable_variance_boost: Option<bool>,

    /// Variance Boost 強度
    pub variance_boost_strength: Option<u8>,

    /// 分散オクタイル
    pub variance_octile: Option<u8>,

    /// Variance Boost カーブ
    pub variance_boost_curve: Option<u8>,

    /// VBR 最小セクションレート (ターゲットビットレートに対する割合)
    pub vbr_min_section_pct: Option<u32>,

    /// VBR 最大セクションレート (ターゲットビットレートに対する割合)
    pub vbr_max_section_pct: Option<u32>,

    /// ターゲットビットレートに対するアンダーシュート許容割合
    pub under_shoot_pct: Option<u32>,

    /// ターゲットビットレートに対するオーバーシュート許容割合
    pub over_shoot_pct: Option<u32>,

    /// 最大ビットレートに対するオーバーシュート許容割合
    pub mbr_over_shoot_pct: Option<u32>,

    /// リコードループ制御 (0=無効, 1=キーフレームのみ, 2=全フレーム)
    pub recode_loop: Option<u32>,

    /// リサイズモード (0=無効, 1=固定, 2=ランダム, 3=動的)
    pub resize_mode: Option<u8>,

    /// リサイズ分母 (8-16)
    pub resize_denom: Option<u8>,

    /// キーフレーム用リサイズ分母 (8-16)
    pub resize_kf_denom: Option<u8>,

    /// AVIF 静止画エンコードモード
    pub avif: Option<bool>,

    /// 量子化マトリクスの有効化
    pub enable_qm: Option<bool>,

    /// 最小 QM レベル (0-15)
    pub min_qm_level: Option<u8>,

    /// 最大 QM レベル (0-15)
    pub max_qm_level: Option<u8>,

    /// クロマ用最小 QM レベル (0-15)
    pub min_chroma_qm_level: Option<u8>,

    /// クロマ用最大 QM レベル (0-15)
    pub max_chroma_qm_level: Option<u8>,

    /// デブロッキングループフィルター (0=無効, 1=有効, 2=高精度)
    pub enable_dlf_flag: Option<u8>,

    /// CDEF レベル (-1=自動, 0=無効, 1-5=レベル)
    pub cdef_level: Option<i32>,

    /// リストレーションフィルタリング (-1=自動, 0=無効, 1=有効)
    pub enable_restoration_filtering: Option<i32>,

    /// テンポラルフィルター (0=無効, 1=有効, 2=適応的)
    pub enable_tf: Option<u8>,

    /// テンポラルフィルター強度
    pub tf_strength: Option<u8>,

    /// オーバーレイフレーム有効化
    pub enable_overlays: Option<bool>,

    /// Motion Field Motion Vector (-1=自動, 0=無効, 1=有効)
    pub enable_mfmv: Option<i32>,

    /// データ駆動型 GOP (Data-driven GOP)
    pub enable_dg: Option<bool>,

    /// GOP 単位レート制御制約
    pub gop_constraint_rc: Option<bool>,

    /// キーフレーム間隔を秒単位として扱う
    pub multiply_keyint: Option<bool>,

    /// スーパーレゾリューション自動検索タイプ
    pub superres_auto_search_type: Option<u8>,

    /// スタートアップ MG (Minigroup) サイズ
    pub startup_mg_size: Option<u8>,

    /// スタートアップ GOP の QP オフセット
    pub startup_qp_offset: Option<i8>,

    /// 輝度 QP バイアス
    pub luminance_qp_bias: Option<u8>,

    /// QP スケール圧縮強度
    pub qp_scale_compress_strength: Option<u8>,

    /// 拡張 CRF QIndex オフセット
    pub extended_crf_qindex_offset: Option<u8>,

    /// S-frame の QP 値
    pub sframe_qp: Option<u8>,

    /// S-frame の QP オフセット
    pub sframe_qp_offset: Option<i8>,

    /// 最大トランスフォームサイズ
    pub max_tx_size: Option<u8>,

    /// 高周波誤差バイアス (テクスチャ保持)
    pub ac_bias: Option<f64>,

    /// 並列化レベル
    pub level_of_parallelism: Option<u32>,
}

impl EncoderConfig {
    /// 必須パラメータを指定して設定を生成する
    pub fn new(width: usize, height: usize, color_format: ColorFormat) -> Self {
        Self {
            width,
            height,
            color_format,
            fps_numerator: 30,
            fps_denominator: 1,
            target_bit_rate: 2_000_000,
            rate_control_mode: RcMode::Vbr,
            enc_mode: 8,
            min_qp_allowed: None,
            max_qp_allowed: None,
            qp: None,
            intra_period_length: None,
            tile_columns: None,
            tile_rows: None,
            look_ahead_distance: None,
            scene_change_detection: true,
            color_primaries: None,
            transfer_characteristics: None,
            matrix_coefficients: None,
            color_range: None,
            chroma_sample_position: None,
            mastering_display: None,
            content_light_level: None,
            starting_buffer_level_ms: None,
            optimal_buffer_level_ms: None,
            maximum_buffer_size_ms: None,
            tune: None,
            stat_report: false,
            fast_decode: None,
            film_grain_denoise_strength: None,
            film_grain_denoise_apply: None,
            adaptive_film_grain: None,
            sharpness: None,
            aq_mode: None,
            max_bit_rate: None,
            screen_content_mode: None,
            intra_refresh_type: None,
            rtc: None,
            hierarchical_levels: None,
            sframe_dist: None,
            sframe_mode: None,
            superres_mode: None,
            superres_denom: None,
            superres_kf_denom: None,
            superres_qthres: None,
            superres_kf_qthres: None,
            lossless: None,
            enable_variance_boost: None,
            variance_boost_strength: None,
            variance_octile: None,
            variance_boost_curve: None,
            vbr_min_section_pct: None,
            vbr_max_section_pct: None,
            under_shoot_pct: None,
            over_shoot_pct: None,
            mbr_over_shoot_pct: None,
            recode_loop: None,
            resize_mode: None,
            resize_denom: None,
            resize_kf_denom: None,
            avif: None,
            enable_qm: None,
            min_qm_level: None,
            max_qm_level: None,
            min_chroma_qm_level: None,
            max_chroma_qm_level: None,
            enable_dlf_flag: None,
            cdef_level: None,
            enable_restoration_filtering: None,
            enable_tf: None,
            tf_strength: None,
            enable_overlays: None,
            enable_mfmv: None,
            enable_dg: None,
            gop_constraint_rc: None,
            multiply_keyint: None,
            superres_auto_search_type: None,
            startup_mg_size: None,
            startup_qp_offset: None,
            luminance_qp_bias: None,
            qp_scale_compress_strength: None,
            extended_crf_qindex_offset: None,
            sframe_qp: None,
            sframe_qp_offset: None,
            max_tx_size: None,
            ac_bias: None,
            level_of_parallelism: None,
        }
    }
}

/// AV1 エンコーダー
#[derive(Debug)]
pub struct Encoder {
    handle: EncoderHandle,
    buffer_header: sys::EbBufferHeaderType,
    buffer: Box<sys::EbSvtIOFormat>,
    input_yuv: Vec<u8>,
    extra_data: Vec<u8>,
    frame_count: u64,
    received_count: u64,
    width: usize,
    color_format: ColorFormat,
    eos: bool,
}

impl Encoder {
    /// エンコーダーインスタンスを生成する
    pub fn new(config: EncoderConfig) -> Result<Self, Error> {
        Self::with_log_level(&config, ENV_VALUE_SVT_LOG_LEVEL)
    }

    fn validate_config(config: &EncoderConfig) -> Result<(), Error> {
        let bad = |name: &'static str| Error::check(sys::EbErrorType_EB_ErrorBadParameter, name);

        if config.width == 0 || config.height == 0 {
            bad("EncoderConfig: width/height must be > 0")?;
        }
        if config.width > u32::MAX as usize || config.height > u32::MAX as usize {
            bad("EncoderConfig: width/height must fit in u32")?;
        }
        if config.fps_numerator > u32::MAX as usize
            || config.fps_denominator > u32::MAX as usize
            || config.target_bit_rate > u32::MAX as usize
        {
            bad("EncoderConfig: fps_numerator/fps_denominator/target_bit_rate must fit in u32")?;
        }
        // plane_sizes の合計が u32 に収まることを検証する
        // (C 側へ n_filled_len として渡すため)
        let (y, u, v) =
            Self::plane_sizes(config.width, config.height, config.color_format).ok_or(Error {
                function: "EncoderConfig: plane size overflow",
                code: sys::EbErrorType_EB_ErrorBadParameter,
            })?;
        if y.checked_add(u)
            .and_then(|s| s.checked_add(v))
            .is_none_or(|total| total > u32::MAX as usize)
        {
            bad("EncoderConfig: total plane size must fit in u32")?;
        }
        if config.fps_denominator == 0 {
            bad("EncoderConfig: fps_denominator must be > 0")?;
        }
        if config.enc_mode > 13 {
            bad("EncoderConfig: enc_mode must be 0-13")?;
        }
        if let Some(qp) = config.qp
            && qp > 63
        {
            bad("EncoderConfig: qp must be 0-63")?;
        }
        if let Some(min) = config.min_qp_allowed
            && min > 63
        {
            bad("EncoderConfig: min_qp_allowed must be 0-63")?;
        }
        if let Some(max) = config.max_qp_allowed
            && max > 63
        {
            bad("EncoderConfig: max_qp_allowed must be 0-63")?;
        }
        if let (Some(min), Some(max)) = (config.min_qp_allowed, config.max_qp_allowed)
            && min > max
        {
            bad("EncoderConfig: min_qp_allowed must be <= max_qp_allowed")?;
        }
        if let Some(fd) = config.fast_decode
            && fd > 2
        {
            bad("EncoderConfig: fast_decode must be 0-2")?;
        }
        if let Some(s) = config.sharpness
            && !(-7..=7).contains(&s)
        {
            bad("EncoderConfig: sharpness must be -7 to 7")?;
        }
        if let Some(aq) = config.aq_mode
            && aq > 2
        {
            bad("EncoderConfig: aq_mode must be 0-2")?;
        }
        if let Some(scm) = config.screen_content_mode
            && scm > 3
        {
            bad("EncoderConfig: screen_content_mode must be 0-3")?;
        }
        if config.rate_control_mode == RcMode::CqpOrCrf && config.target_bit_rate > 0 {
            bad("EncoderConfig: target_bit_rate must be 0 for CRF mode")?;
        }

        Ok(())
    }

    fn with_log_level(config: &EncoderConfig, log_level: &str) -> Result<Self, Error> {
        Self::validate_config(config)?;

        let mut handle = std::ptr::null_mut();
        let buffer = MaybeUninit::<sys::EbBufferHeaderType>::zeroed();
        let buffer_format = MaybeUninit::<sys::EbSvtIOFormat>::zeroed();
        let mut svt_config = MaybeUninit::<sys::EbSvtAv1EncConfiguration>::zeroed();
        unsafe {
            // 念の為に、複数エンコーダーの同時初期化を防止するためのロックを獲得する
            let _guard = GLOBAL_LOCK.lock().inspect_err(|e| {
                // 基本はここに来ることはないはず。
                // またロック確保はあくまでも保険的なもので失敗しても致命的なものではないので、
                // ログを出すだけに留めておく
                log::warn!("failed to acquire the global lock for SVT-AV1: {e}");
            });

            // SVT-AV1 は環境変数経由でログレベルを指定するので、まず最初に設定しておく
            // この設定ができなくても致命的な問題は発生しないので、結果は単に無視する
            std::env::set_var(ENV_KEY_SVT_LOG, log_level);

            let code = sys::svt_av1_enc_init_handle(&mut handle, svt_config.as_mut_ptr());
            Error::check(code, "svt_av1_enc_init_handle")?;

            // FFI 境界の防御: 成功コードでも null が返った場合に備える
            if handle.is_null() {
                return Err(Error {
                    function: "svt_av1_enc_init_handle (null handle)",
                    code: sys::EbErrorType_EB_ErrorBadParameter,
                });
            }

            let mut handle = EncoderHandle {
                inner: handle,
                initialized: false,
            };

            let mut svt_config = svt_config.assume_init();

            // === 基本設定 ===
            svt_config.source_width = config.width as u32;
            svt_config.source_height = config.height as u32;
            svt_config.frame_rate_numerator = config.fps_numerator as u32;
            svt_config.frame_rate_denominator = config.fps_denominator as u32;
            // CRF モードでは target_bit_rate を設定しない (SVT-AV1 がエラーを返すため)
            if config.rate_control_mode != RcMode::CqpOrCrf {
                svt_config.target_bit_rate = config.target_bit_rate as u32;
            }

            // === 品質・速度制御 ===
            svt_config.enc_mode = config.enc_mode as i8;
            if let Some(qp) = config.qp {
                svt_config.qp = qp as u32;
            }
            if let Some(min_qp) = config.min_qp_allowed {
                svt_config.min_qp_allowed = min_qp as u32;
            }
            if let Some(max_qp) = config.max_qp_allowed {
                svt_config.max_qp_allowed = max_qp as u32;
            }

            // === レート制御 ===
            svt_config.rate_control_mode = match config.rate_control_mode {
                RcMode::CqpOrCrf => sys::SvtAv1RcMode_SVT_AV1_RC_MODE_CQP_OR_CRF,
                RcMode::Vbr => sys::SvtAv1RcMode_SVT_AV1_RC_MODE_VBR,
                RcMode::Cbr => sys::SvtAv1RcMode_SVT_AV1_RC_MODE_CBR,
            } as u8;

            if let Some(max_br) = config.max_bit_rate {
                svt_config.max_bit_rate = max_br as u32;
            }

            // === CBR バッファ設定 ===
            if let Some(v) = config.starting_buffer_level_ms {
                svt_config.starting_buffer_level_ms = v as i64;
            }
            if let Some(v) = config.optimal_buffer_level_ms {
                svt_config.optimal_buffer_level_ms = v as i64;
            }
            if let Some(v) = config.maximum_buffer_size_ms {
                svt_config.maximum_buffer_size_ms = v as i64;
            }

            // === GOP・フレーム構造 ===
            if let Some(interval) = config.intra_period_length {
                svt_config.intra_period_length = interval.get() as i32;
            }
            if let Some(irt) = config.intra_refresh_type {
                svt_config.intra_refresh_type = match irt {
                    IntraRefreshType::FwdkfRefresh => {
                        sys::SvtAv1IntraRefreshType_SVT_AV1_FWDKF_REFRESH
                    }
                    IntraRefreshType::KfRefresh => sys::SvtAv1IntraRefreshType_SVT_AV1_KF_REFRESH,
                };
            }
            // pred_structure はレート制御モードに応じて自動設定する
            svt_config.pred_structure = match config.rate_control_mode {
                RcMode::CqpOrCrf => 2, // ランダムアクセス
                RcMode::Vbr => 2,      // VBR はランダムアクセスのみサポート
                RcMode::Cbr => 1,      // CBR は低遅延のみサポート
            };
            svt_config.scene_change_detection = config.scene_change_detection as u32;
            if let Some(lad) = config.look_ahead_distance {
                svt_config.look_ahead_distance = lad as u32;
            }

            // === 並列処理 ===
            if let Some(tc) = config.tile_columns {
                svt_config.tile_columns = tc.get() as i32;
            }
            if let Some(tr) = config.tile_rows {
                svt_config.tile_rows = tr.get() as i32;
            }

            // === 画像フォーマット ===
            let (bit_depth, color_format) = match config.color_format {
                ColorFormat::I420 => (8u32, sys::EbColorFormat_EB_YUV420),
                ColorFormat::I42010 => (10u32, sys::EbColorFormat_EB_YUV420),
            };
            svt_config.encoder_bit_depth = bit_depth;
            svt_config.encoder_color_format = color_format;

            // === カラー情報 ===
            if let Some(cp) = config.color_primaries {
                svt_config.color_primaries = match cp {
                    ColorPrimaries::Bt709 => sys::EbColorPrimaries_EB_CICP_CP_BT_709,
                    ColorPrimaries::Unspecified => sys::EbColorPrimaries_EB_CICP_CP_UNSPECIFIED,
                    ColorPrimaries::Bt470M => sys::EbColorPrimaries_EB_CICP_CP_BT_470_M,
                    ColorPrimaries::Bt470Bg => sys::EbColorPrimaries_EB_CICP_CP_BT_470_B_G,
                    ColorPrimaries::Bt601 => sys::EbColorPrimaries_EB_CICP_CP_BT_601,
                    ColorPrimaries::Smpte240 => sys::EbColorPrimaries_EB_CICP_CP_SMPTE_240,
                    ColorPrimaries::GenericFilm => sys::EbColorPrimaries_EB_CICP_CP_GENERIC_FILM,
                    ColorPrimaries::Bt2020 => sys::EbColorPrimaries_EB_CICP_CP_BT_2020,
                    ColorPrimaries::Xyz => sys::EbColorPrimaries_EB_CICP_CP_XYZ,
                    ColorPrimaries::Smpte431 => sys::EbColorPrimaries_EB_CICP_CP_SMPTE_431,
                    ColorPrimaries::Smpte432 => sys::EbColorPrimaries_EB_CICP_CP_SMPTE_432,
                    ColorPrimaries::Ebu3213 => sys::EbColorPrimaries_EB_CICP_CP_EBU_3213,
                };
            }
            if let Some(tc) = config.transfer_characteristics {
                svt_config.transfer_characteristics = match tc {
                    TransferCharacteristics::Bt709 => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_709
                    }
                    TransferCharacteristics::Unspecified => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_UNSPECIFIED
                    }
                    TransferCharacteristics::Bt470M => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_470_M
                    }
                    TransferCharacteristics::Bt470Bg => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_470_B_G
                    }
                    TransferCharacteristics::Bt601 => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_601
                    }
                    TransferCharacteristics::Smpte240 => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_SMPTE_240
                    }
                    TransferCharacteristics::Linear => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_LINEAR
                    }
                    TransferCharacteristics::Iec61966 => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_IEC_61966
                    }
                    TransferCharacteristics::Bt1361 => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_1361
                    }
                    TransferCharacteristics::Srgb => sys::EbTransferCharacteristics_EB_CICP_TC_SRGB,
                    TransferCharacteristics::Bt202010Bit => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_2020_10_BIT
                    }
                    TransferCharacteristics::Bt202012Bit => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_BT_2020_12_BIT
                    }
                    TransferCharacteristics::Pq => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_SMPTE_2084
                    }
                    TransferCharacteristics::Smpte428 => {
                        sys::EbTransferCharacteristics_EB_CICP_TC_SMPTE_428
                    }
                    TransferCharacteristics::Hlg => sys::EbTransferCharacteristics_EB_CICP_TC_HLG,
                };
            }
            if let Some(mc) = config.matrix_coefficients {
                svt_config.matrix_coefficients = match mc {
                    MatrixCoefficients::Identity => sys::EbMatrixCoefficients_EB_CICP_MC_IDENTITY,
                    MatrixCoefficients::Bt709 => sys::EbMatrixCoefficients_EB_CICP_MC_BT_709,
                    MatrixCoefficients::Unspecified => {
                        sys::EbMatrixCoefficients_EB_CICP_MC_UNSPECIFIED
                    }
                    MatrixCoefficients::Fcc => sys::EbMatrixCoefficients_EB_CICP_MC_FCC,
                    MatrixCoefficients::Bt470Bg => sys::EbMatrixCoefficients_EB_CICP_MC_BT_470_B_G,
                    MatrixCoefficients::Bt601 => sys::EbMatrixCoefficients_EB_CICP_MC_BT_601,
                    MatrixCoefficients::Smpte240 => sys::EbMatrixCoefficients_EB_CICP_MC_SMPTE_240,
                    MatrixCoefficients::Ycgco => sys::EbMatrixCoefficients_EB_CICP_MC_SMPTE_YCGCO,
                    MatrixCoefficients::Bt2020Ncl => {
                        sys::EbMatrixCoefficients_EB_CICP_MC_BT_2020_NCL
                    }
                    MatrixCoefficients::Bt2020Cl => sys::EbMatrixCoefficients_EB_CICP_MC_BT_2020_CL,
                    MatrixCoefficients::Smpte2085 => {
                        sys::EbMatrixCoefficients_EB_CICP_MC_SMPTE_2085
                    }
                    MatrixCoefficients::ChromatNcl => {
                        sys::EbMatrixCoefficients_EB_CICP_MC_CHROMAT_NCL
                    }
                    MatrixCoefficients::ChromatCl => {
                        sys::EbMatrixCoefficients_EB_CICP_MC_CHROMAT_CL
                    }
                    MatrixCoefficients::Ictcp => sys::EbMatrixCoefficients_EB_CICP_MC_ICTCP,
                };
            }
            if let Some(cr) = config.color_range {
                svt_config.color_range = match cr {
                    ColorRange::Studio => sys::EbColorRange_EB_CR_STUDIO_RANGE,
                    ColorRange::Full => sys::EbColorRange_EB_CR_FULL_RANGE,
                };
            }
            if let Some(csp) = config.chroma_sample_position {
                svt_config.chroma_sample_position = match csp {
                    ChromaSamplePosition::Unknown => sys::EbChromaSamplePosition_EB_CSP_UNKNOWN,
                    ChromaSamplePosition::Vertical => sys::EbChromaSamplePosition_EB_CSP_VERTICAL,
                    ChromaSamplePosition::Colocated => sys::EbChromaSamplePosition_EB_CSP_COLOCATED,
                };
            }

            // === HDR メタデータ ===
            if let Some(md) = config.mastering_display {
                svt_config.mastering_display.r.x = md.r.0;
                svt_config.mastering_display.r.y = md.r.1;
                svt_config.mastering_display.g.x = md.g.0;
                svt_config.mastering_display.g.y = md.g.1;
                svt_config.mastering_display.b.x = md.b.0;
                svt_config.mastering_display.b.y = md.b.1;
                svt_config.mastering_display.white_point.x = md.white_point.0;
                svt_config.mastering_display.white_point.y = md.white_point.1;
                svt_config.mastering_display.max_luma = md.max_luminance;
                svt_config.mastering_display.min_luma = md.min_luminance;
            }
            if let Some(cll) = config.content_light_level {
                svt_config.content_light_level.max_cll = cll.max_cll;
                svt_config.content_light_level.max_fall = cll.max_fall;
            }

            // === 品質チューニング ===
            if let Some(tune) = config.tune {
                svt_config.tune = match tune {
                    Tune::Vq => 0,
                    Tune::Psnr => 1,
                    Tune::Ssim => 2,
                    Tune::Iq => 3,
                    Tune::MsSsim => 4,
                };
            }
            svt_config.stat_report = config.stat_report as u32;
            if let Some(fd) = config.fast_decode {
                svt_config.fast_decode = fd;
            }
            if let Some(fgds) = config.film_grain_denoise_strength {
                svt_config.film_grain_denoise_strength = fgds;
            }
            if let Some(fgda) = config.film_grain_denoise_apply {
                svt_config.film_grain_denoise_apply = fgda;
            }
            if let Some(afg) = config.adaptive_film_grain {
                svt_config.adaptive_film_grain = afg;
            }
            if let Some(s) = config.sharpness {
                svt_config.sharpness = s;
            }
            if let Some(aq) = config.aq_mode {
                svt_config.aq_mode = aq;
            }
            if let Some(scm) = config.screen_content_mode {
                svt_config.screen_content_mode = scm as u32;
            }
            if let Some(rtc) = config.rtc {
                svt_config.rtc = rtc;
            }

            // === テンポラルレイヤー ===
            if let Some(hl) = config.hierarchical_levels {
                svt_config.hierarchical_levels = hl;
            }

            // === S-frame ===
            if let Some(sd) = config.sframe_dist {
                svt_config.sframe_dist = sd;
            }
            if let Some(sm) = config.sframe_mode {
                svt_config.sframe_mode = sm as _;
            }

            // === スーパーレゾリューション ===
            if let Some(v) = config.superres_mode {
                svt_config.superres_mode = v;
            }
            if let Some(v) = config.superres_denom {
                svt_config.superres_denom = v;
            }
            if let Some(v) = config.superres_kf_denom {
                svt_config.superres_kf_denom = v;
            }
            if let Some(v) = config.superres_qthres {
                svt_config.superres_qthres = v;
            }
            if let Some(v) = config.superres_kf_qthres {
                svt_config.superres_kf_qthres = v;
            }

            // === ロスレス ===
            if let Some(v) = config.lossless {
                svt_config.lossless = v;
            }

            // === Variance Boost ===
            if let Some(v) = config.enable_variance_boost {
                svt_config.enable_variance_boost = v;
            }
            if let Some(v) = config.variance_boost_strength {
                svt_config.variance_boost_strength = v;
            }
            if let Some(v) = config.variance_octile {
                svt_config.variance_octile = v;
            }
            if let Some(v) = config.variance_boost_curve {
                svt_config.variance_boost_curve = v;
            }

            // === VBR セクションレート ===
            if let Some(v) = config.vbr_min_section_pct {
                svt_config.vbr_min_section_pct = v;
            }
            if let Some(v) = config.vbr_max_section_pct {
                svt_config.vbr_max_section_pct = v;
            }

            // === アンダーシュート / オーバーシュート ===
            if let Some(v) = config.under_shoot_pct {
                svt_config.under_shoot_pct = v;
            }
            if let Some(v) = config.over_shoot_pct {
                svt_config.over_shoot_pct = v;
            }
            if let Some(v) = config.mbr_over_shoot_pct {
                svt_config.mbr_over_shoot_pct = v;
            }

            // === リコードループ ===
            if let Some(v) = config.recode_loop {
                svt_config.recode_loop = v;
            }

            // === リサイズ ===
            if let Some(v) = config.resize_mode {
                svt_config.resize_mode = v;
            }
            if let Some(v) = config.resize_denom {
                svt_config.resize_denom = v;
            }
            if let Some(v) = config.resize_kf_denom {
                svt_config.resize_kf_denom = v;
            }

            // === AVIF ===
            if let Some(v) = config.avif {
                svt_config.avif = v;
            }

            // === 量子化マトリクス ===
            if let Some(v) = config.enable_qm {
                svt_config.enable_qm = v;
            }
            if let Some(v) = config.min_qm_level {
                svt_config.min_qm_level = v;
            }
            if let Some(v) = config.max_qm_level {
                svt_config.max_qm_level = v;
            }
            if let Some(v) = config.min_chroma_qm_level {
                svt_config.min_chroma_qm_level = v;
            }
            if let Some(v) = config.max_chroma_qm_level {
                svt_config.max_chroma_qm_level = v;
            }

            // === フィルター制御 ===
            if let Some(v) = config.enable_dlf_flag {
                svt_config.enable_dlf_flag = v;
            }
            if let Some(v) = config.cdef_level {
                svt_config.cdef_level = v;
            }
            if let Some(v) = config.enable_restoration_filtering {
                svt_config.enable_restoration_filtering = v;
            }

            // === テンポラルフィルター ===
            if let Some(v) = config.enable_tf {
                svt_config.enable_tf = v;
            }
            if let Some(v) = config.tf_strength {
                svt_config.tf_strength = v;
            }
            if let Some(v) = config.enable_overlays {
                svt_config.enable_overlays = v;
            }

            // === Motion Field Motion Vector ===
            if let Some(v) = config.enable_mfmv {
                svt_config.enable_mfmv = v;
            }

            // === GOP 制御 ===
            if let Some(v) = config.enable_dg {
                svt_config.enable_dg = v;
            }
            if let Some(v) = config.gop_constraint_rc {
                svt_config.gop_constraint_rc = v;
            }
            if let Some(v) = config.multiply_keyint {
                svt_config.multiply_keyint = v;
            }

            // === スーパーレゾリューション自動検索 ===
            if let Some(v) = config.superres_auto_search_type {
                svt_config.superres_auto_search_type = v;
            }

            // === スタートアップ制御 ===
            if let Some(v) = config.startup_mg_size {
                svt_config.startup_mg_size = v;
            }
            if let Some(v) = config.startup_qp_offset {
                svt_config.startup_qp_offset = v;
            }

            // === QP 制御 ===
            if let Some(v) = config.luminance_qp_bias {
                svt_config.luminance_qp_bias = v;
            }
            if let Some(v) = config.qp_scale_compress_strength {
                svt_config.qp_scale_compress_strength = v;
            }
            if let Some(v) = config.extended_crf_qindex_offset {
                svt_config.extended_crf_qindex_offset = v;
            }

            // === S-frame 品質制御 ===
            if let Some(v) = config.sframe_qp {
                svt_config.sframe_qp = v;
            }
            if let Some(v) = config.sframe_qp_offset {
                svt_config.sframe_qp_offset = v;
            }

            // === エンコード詳細制御 ===
            if let Some(v) = config.max_tx_size {
                svt_config.max_tx_size = v;
            }
            if let Some(v) = config.ac_bias {
                svt_config.ac_bias = v;
            }
            if let Some(v) = config.level_of_parallelism {
                svt_config.level_of_parallelism = v;
            }

            // per-frame のキーフレーム制御を有効にする
            // VBR モードでは SVT-AV1 がサポートしていないため無効にする
            svt_config.force_key_frames = !matches!(config.rate_control_mode, RcMode::Vbr);

            // core dump する場合を予防する (C++ 版からの移植コード）
            svt_config.frame_scale_evts.start_frame_nums = std::ptr::null_mut();
            svt_config.frame_scale_evts.resize_kf_denoms = std::ptr::null_mut();
            svt_config.frame_scale_evts.resize_denoms = std::ptr::null_mut();

            let code = sys::svt_av1_enc_set_parameter(handle.inner, &mut svt_config);
            Error::check(code, "svt_av1_enc_set_parameter")?;

            let code = sys::svt_av1_enc_init(handle.inner);
            Error::check(code, "svt_av1_enc_init")?;
            handle.initialized = true;

            let mut buffer_header = buffer.assume_init();
            let mut buffer = Box::new(buffer_format.assume_init());
            buffer_header.p_buffer = (&raw mut *buffer).cast();
            buffer_header.size = size_of_val(&buffer_header) as u32;
            buffer_header.p_app_private = std::ptr::null_mut();
            buffer_header.pic_type = sys::EbAv1PictureType_EB_AV1_INVALID_PICTURE;
            buffer_header.metadata = std::ptr::null_mut();

            // validate_config で検証済みなので unwrap は安全
            let (y_size, u_size, v_size) =
                Self::plane_sizes(config.width, config.height, config.color_format).unwrap();
            let mut input_yuv = vec![0; y_size + u_size + v_size];

            buffer.luma = input_yuv.as_mut_ptr();
            buffer.cb = input_yuv.as_mut_ptr().add(y_size);
            buffer.cr = input_yuv.as_mut_ptr().add(y_size + u_size);
            buffer_header.n_filled_len = input_yuv.len() as u32;

            let mut stream_header = std::ptr::null_mut();
            let code = sys::svt_av1_enc_stream_header(handle.inner, &mut stream_header);
            Error::check(code, "svt_av1_enc_stream_header")?;

            // FFI 境界の防御: 成功コードでも null が返った場合に備える
            if stream_header.is_null() {
                return Err(Error {
                    function: "svt_av1_enc_stream_header (null pointer)",
                    code: sys::EbErrorType_EB_ErrorBadParameter,
                });
            }

            // p_buffer が null の場合も未定義動作になるため検証する
            if (*stream_header).p_buffer.is_null() {
                sys::svt_av1_enc_stream_header_release(stream_header);
                return Err(Error {
                    function: "svt_av1_enc_stream_header (null p_buffer)",
                    code: sys::EbErrorType_EB_ErrorBadParameter,
                });
            }

            let extra_data = std::slice::from_raw_parts(
                (*stream_header).p_buffer,
                (*stream_header).n_filled_len as usize,
            )
            .to_vec();

            let code = sys::svt_av1_enc_stream_header_release(stream_header);
            Error::check(code, "svt_av1_enc_stream_header_release")?;

            Ok(Self {
                handle,
                buffer_header,
                buffer,
                input_yuv,
                extra_data,
                frame_count: 0,
                received_count: 0,
                width: config.width,
                color_format: config.color_format,
                eos: false,
            })
        }
    }

    /// MP4 の av01 ボックスに格納するデコーダー向けの情報
    pub fn extra_data(&self) -> &[u8] {
        &self.extra_data
    }

    /// 画像データをエンコードする
    ///
    /// エンコード結果は [`Encoder::next_frame()`] で取得できる
    ///
    /// なお Y プレーンのストライドは入力フレームの幅と等しいことが前提
    ///
    /// また B フレームは扱わない前提（つまり入力フレームと出力フレームの順番が一致する）
    pub fn encode(&mut self, frame: &FrameData<'_>, options: &EncodeOptions) -> Result<(), Error> {
        // EOS 送信済みの場合は追加フレームを受け付けない
        if self.eos {
            return Err(Error {
                function: "shiguredo_svt_av1::Encoder::encode (already finished)",
                code: sys::EbErrorType_EB_ErrorBadParameter,
            });
        }

        // ColorFormat と FrameData の variant が一致していることを検証する
        let format_matches = matches!(
            (&self.color_format, frame),
            (ColorFormat::I420, FrameData::I420 { .. })
                | (ColorFormat::I42010, FrameData::I42010 { .. })
        );
        if !format_matches {
            Error::check(
                sys::EbErrorType_EB_ErrorBadParameter,
                "shiguredo_svt_av1::Encoder::encode (color format mismatch)",
            )?;
        }

        let (y, u, v) = match frame {
            FrameData::I420 { y, u, v } | FrameData::I42010 { y, u, v } => (*y, *u, *v),
        };

        if self.input_yuv.len() != y.len() + u.len() + v.len() {
            Error::check(
                sys::EbErrorType_EB_ErrorBadParameter,
                "shiguredo_svt_av1::Encoder::encode",
            )?;
        }

        self.input_yuv[..y.len()].copy_from_slice(y);
        self.input_yuv[y.len()..][..u.len()].copy_from_slice(u);
        self.input_yuv[y.len() + u.len()..][..v.len()].copy_from_slice(v);

        self.buffer_header.flags = 0;
        self.buffer_header.p_app_private = std::ptr::null_mut();
        self.buffer_header.pts = self.frame_count as i64;
        self.buffer_header.pic_type = if options.force_keyframe {
            sys::EbAv1PictureType_EB_AV1_KEY_PICTURE
        } else {
            sys::EbAv1PictureType_EB_AV1_INVALID_PICTURE
        };
        self.buffer_header.metadata = std::ptr::null_mut();

        // 10-bit の場合はストライドをピクセル数で指定する（バイト数ではない）
        self.buffer.y_stride = self.width as u32;
        self.buffer.cb_stride = self.width.div_ceil(2) as u32;
        self.buffer.cr_stride = self.width.div_ceil(2) as u32;

        let code =
            unsafe { sys::svt_av1_enc_send_picture(self.handle.inner, &mut self.buffer_header) };
        Error::check(code, "svt_av1_enc_send_picture")?;

        self.frame_count += 1;
        Ok(())
    }

    /// これ以上データが来ないことをエンコーダーに伝える
    ///
    /// 残りのエンコード結果は [`Encoder::next_frame()`] で取得できる
    pub fn finish(&mut self) -> Result<(), Error> {
        // 多重呼び出しを防止する
        if self.eos {
            return Err(Error {
                function: "shiguredo_svt_av1::Encoder::finish (already finished)",
                code: sys::EbErrorType_EB_ErrorBadParameter,
            });
        }

        // EOS 送信時はデータサイズを 0 にする必要がある
        // n_filled_len が非ゼロのままだと SVT-AV1 が追加フレームとして解釈し、
        // CBR モードでハングする場合がある
        self.buffer_header.n_filled_len = 0;
        self.buffer_header.flags = sys::EB_BUFFERFLAG_EOS;
        self.buffer_header.pic_type = sys::EbAv1PictureType_EB_AV1_INVALID_PICTURE;
        let code =
            unsafe { sys::svt_av1_enc_send_picture(self.handle.inner, &mut self.buffer_header) };
        Error::check(code, "svt_av1_enc_send_picture")?;
        self.eos = true;

        Ok(())
    }

    /// エンコード済みのフレームを取り出す
    ///
    /// [`Encoder::encode()`] や [`Encoder::finish()`] の後には、
    /// このメソッドを、結果が `None` になるまで呼び出し続ける必要がある
    pub fn next_frame(&mut self) -> Option<EncodedFrame<'_>> {
        // EOS 送信前に全フレーム受信済みの場合は API を呼ばずに None を返す。
        // SVT-AV1 の低遅延モード (CBR) では svt_av1_enc_get_packet が
        // pic_send_done=0 でもブロックするため、この早期リターンが必要。
        if !self.eos && self.received_count >= self.frame_count {
            return None;
        }

        let mut output = std::ptr::null_mut();
        let pic_send_done = self.eos as u8;
        let code =
            unsafe { sys::svt_av1_enc_get_packet(self.handle.inner, &mut output, pic_send_done) };
        if code == sys::EbErrorType_EB_NoErrorEmptyQueue {
            return None;
        }
        if code != sys::EbErrorType_EB_ErrorNone {
            log::error!("svt_av1_enc_get_packet() failed: code={code}");
            return None;
        }

        // FFI 境界の防御: 成功コードでも null が返った場合に備える
        if output.is_null() {
            log::error!("svt_av1_enc_get_packet() returned success but output is null");
            return None;
        }

        let frame = unsafe { EncodedFrame(&mut *output) };
        if (frame.0.flags & sys::EB_BUFFERFLAG_EOS) != 0 {
            None
        } else {
            self.received_count += 1;
            Some(frame)
        }
    }

    /// プレーンサイズを計算する。overflow 時は None を返す。
    fn plane_sizes(
        width: usize,
        height: usize,
        color_format: ColorFormat,
    ) -> Option<(usize, usize, usize)> {
        let bytes_per_pixel = match color_format {
            ColorFormat::I420 => 1,
            ColorFormat::I42010 => 2,
        };
        let y_size = width.checked_mul(height)?.checked_mul(bytes_per_pixel)?;
        let u_size = width
            .div_ceil(2)
            .checked_mul(height.div_ceil(2))?
            .checked_mul(bytes_per_pixel)?;
        let v_size = u_size;
        Some((y_size, u_size, v_size))
    }
}

// SAFETY: Encoder は SVT-AV1 の FFI ハンドルを排他的に所有しており、
// 内部状態への同時アクセスは発生しない。
// SVT-AV1 のエンコーダー API 自体がスレッド安全な設計になっている。
unsafe impl Send for Encoder {}

#[derive(Debug)]
struct EncoderHandle {
    inner: *mut sys::EbComponentType,
    initialized: bool,
}

impl Drop for EncoderHandle {
    fn drop(&mut self) {
        unsafe {
            if self.initialized {
                sys::svt_av1_enc_deinit(self.inner);
            }
            sys::svt_av1_enc_deinit_handle(self.inner);
        }
    }
}

/// エンコードされた映像フレーム
#[derive(Debug)]
pub struct EncodedFrame<'a>(&'a mut sys::EbBufferHeaderType);

impl EncodedFrame<'_> {
    /// 圧縮データ
    pub fn data(&self) -> &[u8] {
        if self.0.p_buffer.is_null() || self.0.n_filled_len == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.0.p_buffer, self.0.n_filled_len as usize) }
    }

    /// キーフレームかどうか
    pub fn is_keyframe(&self) -> bool {
        matches!(
            self.0.pic_type,
            sys::EbAv1PictureType_EB_AV1_KEY_PICTURE
                | sys::EbAv1PictureType_EB_AV1_INTRA_ONLY_PICTURE
        )
    }

    /// Presentation Timestamp
    pub fn pts(&self) -> i64 {
        self.0.pts
    }

    /// Decoding Timestamp
    pub fn dts(&self) -> i64 {
        self.0.dts
    }

    /// テンポラルレイヤーインデックス (0-5)
    pub fn temporal_layer_index(&self) -> u8 {
        self.0.temporal_layer_index
    }

    /// 量子化パラメータ
    pub fn qp(&self) -> u32 {
        self.0.qp
    }

    /// 平均量子化パラメータ
    pub fn avg_qp(&self) -> u32 {
        self.0.avg_qp
    }

    /// フレームタイプ
    pub fn pic_type(&self) -> PictureType {
        match self.0.pic_type {
            sys::EbAv1PictureType_EB_AV1_INTER_PICTURE => PictureType::Inter,
            sys::EbAv1PictureType_EB_AV1_ALT_REF_PICTURE => PictureType::AltRef,
            sys::EbAv1PictureType_EB_AV1_INTRA_ONLY_PICTURE => PictureType::IntraOnly,
            sys::EbAv1PictureType_EB_AV1_KEY_PICTURE => PictureType::Key,
            sys::EbAv1PictureType_EB_AV1_NON_REF_PICTURE => PictureType::NonRef,
            sys::EbAv1PictureType_EB_AV1_FW_KEY_PICTURE => PictureType::ForwardKey,
            sys::EbAv1PictureType_EB_AV1_SHOW_EXISTING_PICTURE => PictureType::ShowExisting,
            sys::EbAv1PictureType_EB_AV1_SWITCH_PICTURE => PictureType::Switch,
            _ => PictureType::Unknown,
        }
    }

    /// Luma SSE (stat_report 有効時のみ)
    pub fn luma_sse(&self) -> u64 {
        self.0.luma_sse
    }

    /// Cb SSE (stat_report 有効時のみ)
    pub fn cb_sse(&self) -> u64 {
        self.0.cb_sse
    }

    /// Cr SSE (stat_report 有効時のみ)
    pub fn cr_sse(&self) -> u64 {
        self.0.cr_sse
    }

    /// Luma SSIM (stat_report 有効時のみ)
    pub fn luma_ssim(&self) -> f64 {
        self.0.luma_ssim
    }

    /// Cb SSIM (stat_report 有効時のみ)
    pub fn cb_ssim(&self) -> f64 {
        self.0.cb_ssim
    }

    /// Cr SSIM (stat_report 有効時のみ)
    pub fn cr_ssim(&self) -> f64 {
        self.0.cr_ssim
    }
}

impl Drop for EncodedFrame<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::svt_av1_enc_release_out_buffer(&mut (self.0 as *mut _));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_encoder() {
        // OK
        let config = encoder_config();
        assert!(Encoder::new(config).is_ok());

        // NG (どうしても SVT-AV1 のエラーログが出てしまい紛らわしいので、エラーログを抑制するようにしている）
        let mut config = encoder_config();
        config.fps_denominator = 0;
        assert!(Encoder::with_log_level(&config, "0").is_err());
    }

    #[test]
    fn encode_black() {
        let config = encoder_config();
        let width = config.width;
        let height = config.height;
        let mut encoder = Encoder::new(config).expect("failed to create");
        let mut encoded_count = 0;

        let size = width * height;
        let y = vec![0; size];
        let u = vec![0; size / 4];
        let v = vec![0; size / 4];
        let frame = FrameData::I420 {
            y: &y,
            u: &u,
            v: &v,
        };
        let options = EncodeOptions::default();

        encoder.encode(&frame, &options).expect("failed to encode");
        while encoder.next_frame().is_some() {
            encoded_count += 1;
        }

        // 一フレームだけ処理すると SVT-AV1 が `--avif 1` を使うようにエラーログを出すので
        // それを防止するために二フレーム目も与えている
        encoder.encode(&frame, &options).expect("failed to encode");
        while encoder.next_frame().is_some() {
            encoded_count += 1;
        }

        encoder.finish().expect("failed to finish");
        while encoder.next_frame().is_some() {
            encoded_count += 1;
        }

        assert_eq!(encoded_count, 2);
    }

    #[test]
    fn encode_10bit() {
        let mut config = EncoderConfig::new(320, 320, ColorFormat::I42010);
        config.target_bit_rate = 1_000_000;
        config.fps_numerator = 1;
        config.fps_denominator = 1;
        let mut encoder = Encoder::new(config).expect("failed to create");

        let size = 320 * 320;
        // 10-bit: 各ピクセル 2 バイト
        let y = vec![0u8; size * 2];
        let u = vec![0u8; (size / 4) * 2];
        let v = vec![0u8; (size / 4) * 2];
        let frame = FrameData::I42010 {
            y: &y,
            u: &u,
            v: &v,
        };

        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder.finish().expect("failed to finish");

        let mut count = 0;
        while encoder.next_frame().is_some() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn encode_cbr() {
        let mut config = encoder_config();
        config.rate_control_mode = RcMode::Cbr;
        let mut encoder = Encoder::new(config).expect("failed to create");

        let y = vec![0u8; 320 * 320];
        let u = vec![0u8; 160 * 160];
        let v = vec![0u8; 160 * 160];
        let frame = FrameData::I420 {
            y: &y,
            u: &u,
            v: &v,
        };

        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder.finish().expect("failed to finish");

        let mut count = 0;
        while encoder.next_frame().is_some() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn encode_crf() {
        let mut config = EncoderConfig::new(320, 320, ColorFormat::I420);
        config.enc_mode = 13;
        config.fps_numerator = 1;
        config.fps_denominator = 1;
        config.rate_control_mode = RcMode::CqpOrCrf;
        config.target_bit_rate = 0;
        config.qp = Some(35);
        let mut encoder = Encoder::new(config).expect("failed to create");

        let y = vec![0u8; 320 * 320];
        let u = vec![0u8; 160 * 160];
        let v = vec![0u8; 160 * 160];
        let frame = FrameData::I420 {
            y: &y,
            u: &u,
            v: &v,
        };

        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder.finish().expect("failed to finish");

        let mut count = 0;
        while encoder.next_frame().is_some() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn color_format_mismatch() {
        // I420 エンコーダーに I42010 データを渡すとエラーになる
        let config = encoder_config();
        let mut encoder = Encoder::new(config).expect("failed to create");

        let y = vec![0u8; 320 * 320 * 2];
        let u = vec![0u8; 160 * 160 * 2];
        let v = vec![0u8; 160 * 160 * 2];
        let frame = FrameData::I42010 {
            y: &y,
            u: &u,
            v: &v,
        };
        assert!(encoder.encode(&frame, &EncodeOptions::default()).is_err());
    }

    #[test]
    fn encoded_frame_has_data() {
        let config = encoder_config();
        let mut encoder = Encoder::new(config).expect("failed to create");

        let y = vec![0u8; 320 * 320];
        let u = vec![0u8; 160 * 160];
        let v = vec![0u8; 160 * 160];
        let frame = FrameData::I420 {
            y: &y,
            u: &u,
            v: &v,
        };

        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder
            .encode(&frame, &EncodeOptions::default())
            .expect("failed to encode");
        encoder.finish().expect("failed to finish");

        while let Some(frame) = encoder.next_frame() {
            assert!(!frame.data().is_empty());
        }
    }

    fn encoder_config() -> EncoderConfig {
        let mut config = EncoderConfig::new(320, 320, ColorFormat::I420);
        config.target_bit_rate = 1_000_000;
        config.fps_numerator = 1;
        config.fps_denominator = 1;
        config
    }
}
