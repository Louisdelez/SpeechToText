use crate::types::Language;

pub struct Strings {
    // App
    pub app_title: &'static str,
    pub app_subtitle: &'static str,

    // Empty state
    pub preparing_model: &'static str,
    pub downloading_model: &'static str,
    pub loading_model: &'static str,
    pub ready: &'static str,
    pub error_prefix: &'static str,
    pub ready_hint: &'static str,
    pub check_internet: &'static str,

    // Controls
    pub import: &'static str,
    pub transcribing: &'static str,

    // Bubble
    pub mic: &'static str,
    pub copy: &'static str,

    // Settings
    pub settings: &'static str,
    pub back: &'static str,
    pub acceleration: &'static str,
    pub acceleration_desc: &'static str,
    pub language_label: &'static str,
    pub language_desc: &'static str,

    // History
    pub history: &'static str,
    pub no_transcription: &'static str,
    pub clear: &'static str,
}

const FR: Strings = Strings {
    app_title: "Speech to Text",
    app_subtitle: "100% local · Whisper AI",

    preparing_model: "Preparation du modele...",
    downloading_model: "Telechargement du modele...",
    loading_model: "Chargement du modele...",
    ready: "Pret a transcrire",
    error_prefix: "Erreur : ",
    ready_hint: "Appuyez sur Record ou importez un fichier",
    check_internet: "Verifiez votre connexion internet",

    import: "Import",
    transcribing: "Transcription en cours...",

    mic: "Micro",
    copy: "Copier",

    settings: "Parametres",
    back: "Retour",
    acceleration: "Acceleration",
    acceleration_desc: "Choisir le processeur pour la transcription",
    language_label: "Langue",
    language_desc: "Langue de transcription et de l'interface",

    history: "Historique",
    no_transcription: "Aucune transcription",
    clear: "Effacer",
};

const EN: Strings = Strings {
    app_title: "Speech to Text",
    app_subtitle: "100% local · Whisper AI",

    preparing_model: "Preparing model...",
    downloading_model: "Downloading model...",
    loading_model: "Loading model...",
    ready: "Ready to transcribe",
    error_prefix: "Error: ",
    ready_hint: "Press Record or import a file",
    check_internet: "Check your internet connection",

    import: "Import",
    transcribing: "Transcribing...",

    mic: "Mic",
    copy: "Copy",

    settings: "Settings",
    back: "Back",
    acceleration: "Acceleration",
    acceleration_desc: "Choose the processor for transcription",
    language_label: "Language",
    language_desc: "Transcription and interface language",

    history: "History",
    no_transcription: "No transcriptions yet",
    clear: "Clear",
};

pub fn t(lang: Language) -> &'static Strings {
    match lang {
        Language::Fr => &FR,
        Language::En => &EN,
    }
}
