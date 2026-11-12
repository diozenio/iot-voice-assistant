use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoicePreset {
    pub id: &'static str,
    pub name: &'static str,
    pub provider: &'static str,
    pub emotion: &'static str,
    pub description: &'static str,
    pub model: &'static str,
    pub voice: &'static str,
    pub instructions: &'static str,
}

const GEMINI: &str = "google/gemini-3.8-flash-tts";
const MAI: &str = "microsoft/mai-voice-2.1";

const PRESETS: &[VoicePreset] = &[
    VoicePreset {
        id: "gemini-amigavel",
        name: "Amigável",
        provider: "Google Gemini",
        emotion: "Acolhedora",
        description: "Tom caloroso e simpático, como um amigo conversando com você.",
        model: GEMINI,
        voice: "Sulafat",
        instructions: "Speak in natural Brazilian Portuguese with a warm, friendly and welcoming tone, like a close friend.",
    },
    VoicePreset {
        id: "gemini-animada",
        name: "Animada",
        provider: "Google Gemini",
        emotion: "Alegre e enérgica",
        description: "Voz leve e sorridente, cheia de energia para o dia a dia.",
        model: GEMINI,
        voice: "Puck",
        instructions: "Speak in natural Brazilian Portuguese, upbeat, cheerful and full of energy, as if smiling while talking.",
    },
    VoicePreset {
        id: "gemini-empolgada",
        name: "Empolgada",
        provider: "Google Gemini",
        emotion: "Entusiasmo",
        description: "Entonação viva e empolgada, ideal para dar boas notícias.",
        model: GEMINI,
        voice: "Fenrir",
        instructions: "Speak in natural Brazilian Portuguese, excited and enthusiastic, with lively intonation.",
    },
    VoicePreset {
        id: "gemini-calma",
        name: "Calma",
        provider: "Google Gemini",
        emotion: "Tranquila e relaxante",
        description: "Voz suave e pausada, que passa serenidade.",
        model: GEMINI,
        voice: "Achernar",
        instructions: "Speak in natural Brazilian Portuguese, calm, soft and soothing, at a slow and relaxed pace.",
    },
    VoicePreset {
        id: "gemini-empatica",
        name: "Empática",
        provider: "Google Gemini",
        emotion: "Acolhimento",
        description: "Tom gentil e reconfortante, como quem tranquiliza alguém.",
        model: GEMINI,
        voice: "Gacrux",
        instructions: "Speak in natural Brazilian Portuguese, empathetic, gentle and comforting, as if reassuring someone.",
    },
    VoicePreset {
        id: "gemini-profissional",
        name: "Profissional",
        provider: "Google Gemini",
        emotion: "Séria e confiante",
        description: "Voz clara e firme, sem emoção exagerada, em ritmo constante.",
        model: GEMINI,
        voice: "Charon",
        instructions: "Speak in natural Brazilian Portuguese, clear, confident and professional, with neutral emotion and a steady pace.",
    },
    VoicePreset {
        id: "gemini-misteriosa",
        name: "Misteriosa",
        provider: "Google Gemini",
        emotion: "Sussurrada e misteriosa",
        description: "Voz baixa e sussurrada, com clima de suspense.",
        model: GEMINI,
        voice: "Enceladus",
        instructions: "Speak in natural Brazilian Portuguese, low, breathy and mysterious, almost whispering.",
    },
    VoicePreset {
        id: "mai-luana",
        name: "Luana",
        provider: "Microsoft MAI",
        emotion: "Neutra e natural",
        description: "Voz feminina brasileira realista, sem variação emocional marcada.",
        model: MAI,
        voice: "pt-BR-Luana:MAI-Voice-2.1",
        instructions: "",
    },
    VoicePreset {
        id: "mai-caio",
        name: "Caio",
        provider: "Microsoft MAI",
        emotion: "Neutro e natural",
        description: "Voz masculina brasileira realista, direta e sem exageros.",
        model: MAI,
        voice: "pt-BR-Caio:MAI-Voice-2.1",
        instructions: "",
    },
    VoicePreset {
        id: "eleven-natural",
        name: "Sarah (natural)",
        provider: "ElevenLabs",
        emotion: "Suave e expressiva",
        description: "Voz feminina natural, com boa expressividade em português.",
        model: "elevenlabs/eleven-multilingual-v2",
        voice: "sarah",
        instructions: "",
    },
    VoicePreset {
        id: "eleven-rapida",
        name: "Sarah (rápida)",
        provider: "ElevenLabs",
        emotion: "Neutra e ágil",
        description: "A mesma voz com menor latência: a resposta começa mais cedo.",
        model: "elevenlabs/eleven-flash-v2.5",
        voice: "sarah",
        instructions: "",
    },
];

pub fn all() -> &'static [VoicePreset] {
    PRESETS
}
