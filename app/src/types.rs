use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub text: String,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub mode: AppMode,
    pub messages: Vec<ChatMessage>,
    pub created_at: DateTime<Local>,
}

// Keep for backward compat with worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionMessage {
    pub text: String,
    pub translated: Option<String>,
    pub timestamp: DateTime<Local>,
    pub duration_secs: f32,
    pub source: AudioSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioSource {
    Microphone,
    File(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Processing,
    Translating,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelState {
    NotDownloaded,
    Downloading { progress_pct: u8 },
    Loading,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeDevice {
    Cpu,
    Gpu,
}

impl ComputeDevice {
    pub fn label(&self) -> &'static str {
        match self {
            ComputeDevice::Cpu => "CPU",
            ComputeDevice::Gpu => "GPU",
        }
    }

    pub fn use_gpu(&self) -> bool {
        matches!(self, ComputeDevice::Gpu)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChoice {
    Cpu,
    Gpu,
    Auto,
}

impl DeviceChoice {
    pub const ALL: &'static [DeviceChoice] = &[DeviceChoice::Auto, DeviceChoice::Gpu, DeviceChoice::Cpu];

    pub fn resolve(&self) -> ComputeDevice {
        match self {
            DeviceChoice::Cpu => ComputeDevice::Cpu,
            DeviceChoice::Gpu => ComputeDevice::Gpu,
            DeviceChoice::Auto => ComputeDevice::Gpu,
        }
    }

    pub fn use_gpu(&self) -> bool {
        self.resolve().use_gpu()
    }
}

impl std::fmt::Display for DeviceChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceChoice::Cpu => write!(f, "CPU"),
            DeviceChoice::Gpu => write!(f, "GPU"),
            DeviceChoice::Auto => write!(f, "Auto"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceConfig {
    pub global: DeviceChoice,
    pub whisper: DeviceChoice,
    pub opus_fr_en: DeviceChoice,
    pub opus_en_fr: DeviceChoice,
    pub qwen: DeviceChoice,
    pub kokoro: DeviceChoice,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            global: DeviceChoice::Auto,
            whisper: DeviceChoice::Auto,
            opus_fr_en: DeviceChoice::Auto,
            opus_en_fr: DeviceChoice::Auto,
            qwen: DeviceChoice::Auto,
            kokoro: DeviceChoice::Auto,
        }
    }
}

impl DeviceConfig {
    fn resolve_for(&self, choice: DeviceChoice) -> bool {
        match choice {
            DeviceChoice::Auto => self.global.use_gpu(),
            other => other.use_gpu(),
        }
    }

    pub fn whisper_device(&self) -> ComputeDevice {
        match self.whisper {
            DeviceChoice::Auto => self.global.resolve(),
            other => other.resolve(),
        }
    }

    pub fn llm_use_gpu(&self) -> bool {
        self.resolve_for(self.qwen)
    }

    pub fn tts_use_gpu(&self) -> bool {
        self.resolve_for(self.kokoro)
    }

    pub fn opus_use_gpu(&self, src: Language, tgt: Language) -> bool {
        let choice = match (src, tgt) {
            (Language::Fr, Language::En) => self.opus_fr_en,
            (Language::En, Language::Fr) => self.opus_en_fr,
            _ => self.opus_fr_en,
        };
        self.resolve_for(choice)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Fr,
    En,
}

impl Language {
    pub const ALL: &'static [Language] = &[Language::Fr, Language::En];

    pub fn whisper_code(&self) -> &'static str {
        match self {
            Language::Fr => "fr",
            Language::En => "en",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Fr => write!(f, "Francais"),
            Language::En => write!(f, "English"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppMode {
    SpeechToText,
    Translation,
    Summary,
    Corrector,
    PromptEngineer,
    TextToSpeech,
}

impl AppMode {
    pub const ALL: &'static [AppMode] = &[AppMode::SpeechToText, AppMode::Translation, AppMode::Summary, AppMode::Corrector, AppMode::PromptEngineer, AppMode::TextToSpeech];
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::SpeechToText => write!(f, "Speech to Text"),
            AppMode::Translation => write!(f, "Translation"),
            AppMode::Summary => write!(f, "Resume"),
            AppMode::Corrector => write!(f, "Correcteur"),
            AppMode::PromptEngineer => write!(f, "Prompt Engineer"),
            AppMode::TextToSpeech => write!(f, "Text to Speech"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtsEngine {
    Kokoro,
    Chatterbox,
    Qwen3,
}

impl TtsEngine {
    pub const ALL: &'static [TtsEngine] = &[TtsEngine::Kokoro, TtsEngine::Chatterbox, TtsEngine::Qwen3];
}

impl std::fmt::Display for TtsEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsEngine::Kokoro => write!(f, "Kokoro (leger)"),
            TtsEngine::Chatterbox => write!(f, "Chatterbox"),
            TtsEngine::Qwen3 => write!(f, "Qwen3-TTS"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationEngine {
    Fast,
    Smart,
}

impl TranslationEngine {
    pub const ALL: &'static [TranslationEngine] = &[TranslationEngine::Fast, TranslationEngine::Smart];
}

impl std::fmt::Display for TranslationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslationEngine::Fast => write!(f, "Rapide"),
            TranslationEngine::Smart => write!(f, "Intelligent"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptLength {
    Short,
    Medium,
    Long,
}

impl PromptLength {
    pub const ALL: &'static [PromptLength] = &[PromptLength::Short, PromptLength::Medium, PromptLength::Long];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WritingStyle {
    Authentic,
    Serious,
    Casual,
    Professional,
    Friendly,
    Formal,
}

impl WritingStyle {
    pub const ALL: &'static [WritingStyle] = &[
        WritingStyle::Authentic,
        WritingStyle::Serious,
        WritingStyle::Casual,
        WritingStyle::Professional,
        WritingStyle::Friendly,
        WritingStyle::Formal,
    ];
}

impl std::fmt::Display for WritingStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WritingStyle::Authentic => write!(f, "Authentique"),
            WritingStyle::Serious => write!(f, "Serieux"),
            WritingStyle::Casual => write!(f, "Detendu"),
            WritingStyle::Professional => write!(f, "Professionnel"),
            WritingStyle::Friendly => write!(f, "Amical"),
            WritingStyle::Formal => write!(f, "Soutenu"),
        }
    }
}

impl std::fmt::Display for PromptLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptLength::Short => write!(f, "Court"),
            PromptLength::Medium => write!(f, "Moyen"),
            PromptLength::Long => write!(f, "Long"),
        }
    }
}

// ── Kokoro TTS settings ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KokoroVoice {
    pub code: &'static str,
    pub label: &'static str,
}

impl std::fmt::Display for KokoroVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub const KOKORO_VOICES: &[KokoroVoice] = &[
    // US Femme
    KokoroVoice { code: "af_heart",   label: "Heart (US F)" },
    KokoroVoice { code: "af_alloy",   label: "Alloy (US F)" },
    KokoroVoice { code: "af_aoede",   label: "Aoede (US F)" },
    KokoroVoice { code: "af_bella",   label: "Bella (US F)" },
    KokoroVoice { code: "af_jessica", label: "Jessica (US F)" },
    KokoroVoice { code: "af_kore",    label: "Kore (US F)" },
    KokoroVoice { code: "af_nicole",  label: "Nicole (US F)" },
    KokoroVoice { code: "af_nova",    label: "Nova (US F)" },
    KokoroVoice { code: "af_river",   label: "River (US F)" },
    KokoroVoice { code: "af_sarah",   label: "Sarah (US F)" },
    KokoroVoice { code: "af_sky",     label: "Sky (US F)" },
    // US Homme
    KokoroVoice { code: "am_adam",    label: "Adam (US H)" },
    KokoroVoice { code: "am_echo",    label: "Echo (US H)" },
    KokoroVoice { code: "am_eric",    label: "Eric (US H)" },
    KokoroVoice { code: "am_fenrir",  label: "Fenrir (US H)" },
    KokoroVoice { code: "am_liam",    label: "Liam (US H)" },
    KokoroVoice { code: "am_michael", label: "Michael (US H)" },
    KokoroVoice { code: "am_onyx",    label: "Onyx (US H)" },
    KokoroVoice { code: "am_puck",    label: "Puck (US H)" },
    KokoroVoice { code: "am_santa",   label: "Santa (US H)" },
    // UK Femme
    KokoroVoice { code: "bf_alice",     label: "Alice (UK F)" },
    KokoroVoice { code: "bf_emma",      label: "Emma (UK F)" },
    KokoroVoice { code: "bf_isabella",  label: "Isabella (UK F)" },
    KokoroVoice { code: "bf_lily",      label: "Lily (UK F)" },
    // UK Homme
    KokoroVoice { code: "bm_daniel",  label: "Daniel (UK H)" },
    KokoroVoice { code: "bm_fable",   label: "Fable (UK H)" },
    KokoroVoice { code: "bm_george",  label: "George (UK H)" },
    KokoroVoice { code: "bm_lewis",   label: "Lewis (UK H)" },
    // Francais
    KokoroVoice { code: "ff_siwis",   label: "Siwis (FR F)" },
    // Espagnol
    KokoroVoice { code: "ef_dora",    label: "Dora (ES F)" },
    KokoroVoice { code: "em_alex",    label: "Alex (ES H)" },
    KokoroVoice { code: "em_santa",   label: "Santa (ES H)" },
    // Hindi
    KokoroVoice { code: "hf_alpha",   label: "Alpha (HI F)" },
    KokoroVoice { code: "hf_beta",    label: "Beta (HI F)" },
    KokoroVoice { code: "hm_omega",   label: "Omega (HI H)" },
    KokoroVoice { code: "hm_psi",     label: "Psi (HI H)" },
    // Italien
    KokoroVoice { code: "if_sara",    label: "Sara (IT F)" },
    KokoroVoice { code: "im_nicola",  label: "Nicola (IT H)" },
    // Japonais
    KokoroVoice { code: "jf_alpha",      label: "Alpha (JA F)" },
    KokoroVoice { code: "jf_gongitsune", label: "Gongitsune (JA F)" },
    KokoroVoice { code: "jf_nezumi",     label: "Nezumi (JA F)" },
    KokoroVoice { code: "jf_tebukuro",   label: "Tebukuro (JA F)" },
    KokoroVoice { code: "jm_kumo",       label: "Kumo (JA H)" },
    // Portugais
    KokoroVoice { code: "pf_dora",    label: "Dora (PT F)" },
    KokoroVoice { code: "pm_alex",    label: "Alex (PT H)" },
    KokoroVoice { code: "pm_santa",   label: "Santa (PT H)" },
    // Chinois
    KokoroVoice { code: "zf_xiaobei",  label: "Xiaobei (ZH F)" },
    KokoroVoice { code: "zf_xiaoni",   label: "Xiaoni (ZH F)" },
    KokoroVoice { code: "zf_xiaoxiao", label: "Xiaoxiao (ZH F)" },
    KokoroVoice { code: "zf_xiaoyi",   label: "Xiaoyi (ZH F)" },
    KokoroVoice { code: "zm_yunjian",  label: "Yunjian (ZH H)" },
    KokoroVoice { code: "zm_yunxi",    label: "Yunxi (ZH H)" },
    KokoroVoice { code: "zm_yunxia",   label: "Yunxia (ZH H)" },
    KokoroVoice { code: "zm_yunyang",  label: "Yunyang (ZH H)" },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KokoroSpeed {
    VerySlow,
    Slow,
    Normal,
    Fast,
    VeryFast,
    Max,
}

impl KokoroSpeed {
    pub const ALL: &'static [KokoroSpeed] = &[
        KokoroSpeed::VerySlow, KokoroSpeed::Slow, KokoroSpeed::Normal,
        KokoroSpeed::Fast, KokoroSpeed::VeryFast, KokoroSpeed::Max,
    ];

    pub fn value(&self) -> f32 {
        match self {
            KokoroSpeed::VerySlow => 0.5,
            KokoroSpeed::Slow => 0.75,
            KokoroSpeed::Normal => 1.0,
            KokoroSpeed::Fast => 1.25,
            KokoroSpeed::VeryFast => 1.5,
            KokoroSpeed::Max => 2.0,
        }
    }
}

impl std::fmt::Display for KokoroSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KokoroSpeed::VerySlow => write!(f, "Tres lent (0.5x)"),
            KokoroSpeed::Slow => write!(f, "Lent (0.75x)"),
            KokoroSpeed::Normal => write!(f, "Normal (1.0x)"),
            KokoroSpeed::Fast => write!(f, "Rapide (1.25x)"),
            KokoroSpeed::VeryFast => write!(f, "Tres rapide (1.5x)"),
            KokoroSpeed::Max => write!(f, "Maximum (2.0x)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KokoroBlendRatio {
    R90_10,
    R80_20,
    R70_30,
    R60_40,
    R50_50,
    R40_60,
    R30_70,
    R20_80,
    R10_90,
}

impl KokoroBlendRatio {
    pub const ALL: &'static [KokoroBlendRatio] = &[
        KokoroBlendRatio::R90_10, KokoroBlendRatio::R80_20, KokoroBlendRatio::R70_30,
        KokoroBlendRatio::R60_40, KokoroBlendRatio::R50_50, KokoroBlendRatio::R40_60,
        KokoroBlendRatio::R30_70, KokoroBlendRatio::R20_80, KokoroBlendRatio::R10_90,
    ];

    pub fn weights(&self) -> (f32, f32) {
        match self {
            KokoroBlendRatio::R90_10 => (0.9, 0.1),
            KokoroBlendRatio::R80_20 => (0.8, 0.2),
            KokoroBlendRatio::R70_30 => (0.7, 0.3),
            KokoroBlendRatio::R60_40 => (0.6, 0.4),
            KokoroBlendRatio::R50_50 => (0.5, 0.5),
            KokoroBlendRatio::R40_60 => (0.4, 0.6),
            KokoroBlendRatio::R30_70 => (0.3, 0.7),
            KokoroBlendRatio::R20_80 => (0.2, 0.8),
            KokoroBlendRatio::R10_90 => (0.1, 0.9),
        }
    }
}

impl std::fmt::Display for KokoroBlendRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (a, _) = self.weights();
        write!(f, "{:.0}% / {:.0}%", a * 100.0, (1.0 - a) * 100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KokoroConfig {
    pub voice: KokoroVoice,
    pub speed: KokoroSpeed,
    pub blend_enabled: bool,
    pub blend_voice: KokoroVoice,
    pub blend_ratio: KokoroBlendRatio,
}

impl Default for KokoroConfig {
    fn default() -> Self {
        Self {
            voice: KOKORO_VOICES[0],         // af_heart
            speed: KokoroSpeed::Normal,
            blend_enabled: false,
            blend_voice: KOKORO_VOICES[11],  // am_adam
            blend_ratio: KokoroBlendRatio::R50_50,
        }
    }
}
