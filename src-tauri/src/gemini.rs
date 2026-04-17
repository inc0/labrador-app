/// Gemini API — OCR, classification, alt-text, and linking.
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::json;

pub struct GeminiClient {
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl GeminiClient {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            http: reqwest::Client::new(),
        }
    }

    async fn complete(&self, system: &str, user_text: &str, image: Option<(&[u8], &str)>) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let mut parts = vec![json!({ "text": user_text })];
        if let Some((bytes, mime)) = image {
            parts.push(json!({
                "inline_data": {
                    "mime_type": mime,
                    "data": B64.encode(bytes),
                }
            }));
        }

        let body = json!({
            "system_instruction": { "parts": [{ "text": system }] },
            "contents": [{ "role": "user", "parts": parts }],
        });

        let resp: serde_json::Value = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("unexpected Gemini response: {resp}"))
    }

    /// Classify an image as "notes" or "photo".
    pub async fn classify(&self, image: &[u8], mime: &str) -> Result<ImageKind> {
        let answer = self.complete(
            "You classify lab images into exactly one of two categories:\n\
             notes — primarily a photo of handwritten or printed text \
             (notebook page, whiteboard, sticky note, printed document).\n\
             photo — primarily a photograph of something physical \
             (specimen, equipment, petri dish, gel, sample, scene).\n\
             Reply with a single word: notes or photo.",
            "Classify this image.",
            Some((image, mime)),
        ).await?;
        if answer.to_lowercase().contains("notes") {
            Ok(ImageKind::Notes)
        } else {
            Ok(ImageKind::Photo)
        }
    }

    /// OCR a handwritten notes image. Returns cleaned-up plain text.
    pub async fn transcribe(&self, image: &[u8], mime: &str) -> Result<String> {
        self.complete(
            "You are an expert handwriting transcription engine. Rules:\n\
             - Preserve structure: headings, lists.\n\
             - Mark underlined words/phrases with underscores: _word_ or _multi word phrase_.\n\
             - Describe diagrams/drawings briefly in [brackets].\n\
             - Correct obvious OCR artifacts: stray dots, accidental marks, noise — omit them.\n\
             - Fix punctuation that is clearly wrong given context.\n\
             - If a word is ambiguous, choose the spelling that makes most sense in context.\n\
             - Do NOT invent content. Every word must trace back to something visible.\n\
             - Output plain text only, no commentary.",
            "Transcribe all handwritten text. Fix obvious artifacts and resolve ambiguous \
             words using context, but do not add or invent any content.",
            Some((image, mime)),
        ).await
    }

    /// Generate a concise scientific alt-text for a lab photo.
    pub async fn alt_text(&self, image: &[u8], mime: &str) -> Result<String> {
        let text = self.complete(
            "You write concise alt-text for images in a scientific lab notebook. \
             Describe what is shown in one sentence (≤20 words). \
             Be specific: mention colours, morphology, labels, scale if visible. \
             Do not start with 'Image of' or 'Photo of'.",
            "Describe this lab image for an alt-text attribute.",
            Some((image, mime)),
        ).await?;
        Ok(text.trim().to_string())
    }

    /// Transcribe a spoken-word audio recording to plain text.
    pub async fn transcribe_audio(&self, bytes: &[u8], mime: &str) -> Result<String> {
        self.complete(
            "You are an accurate speech transcription engine. Rules:\n\
             - Transcribe all spoken words faithfully.\n\
             - Preserve structure: if the speaker enumerates items, use bullet points.\n\
             - Remove filler words (um, uh, like) and obvious false starts.\n\
             - Do NOT add commentary, timestamps, or speaker labels.\n\
             - Output plain text only.",
            "Transcribe this audio recording.",
            Some((bytes, mime)),
        ).await
    }

    /// Take combined transcribed text and return a structured note.
    pub async fn link(&self, raw_text: &str, max_concepts: usize) -> Result<LinkedNote> {
        let user_prompt = format!(
            "Analyze the following transcribed notes and return a JSON object with this exact schema:\n\
             {{\n\
               \"title\": \"<concise title, 2-6 words>\",\n\
               \"tags\": [\"<broad topic 1>\", \"<broad topic 2>\"],\n\
               \"concepts\": [\"<specific concept A>\"],\n\
               \"linked_text\": \"<original text with [[wikilinks]] added>\"\n\
             }}\n\
             Rules:\n\
             - tags: 1-5 broad topic areas\n\
             - concepts: up to {max_concepts} specific named entities or terms\n\
             - linked_text: copy original text verbatim, wrap key concepts in [[double brackets]]\n\
             - Only link a concept the FIRST time it appears\n\
             - Preserve any _underscored_ spans exactly as-is\n\
             Notes to process:\n{raw_text}"
        );

        let resp = self.complete(
            "You are a scientific note organizer. Return only valid JSON, no markdown fences.",
            &user_prompt,
            None,
        ).await?;

        // Strip optional markdown fences
        let json_str = resp
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let v: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("linker returned invalid JSON: {e}\nRaw: {resp}"))?;

        let mut linked_text = v["linked_text"].as_str().unwrap_or(raw_text).to_string();
        // Convert _underscored_ spans to [[wikilinks]]
        linked_text = regex_replace_underscores(&linked_text);

        Ok(LinkedNote {
            title: v["title"].as_str().unwrap_or("Untitled").to_string(),
            tags: v["tags"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|t| t.as_str())
                .map(str::to_string)
                .collect(),
            linked_text,
        })
    }
}

fn regex_replace_underscores(text: &str) -> String {
    // Simple state-machine replacement of _..._  →  [[...]]
    // Avoids pulling in the regex crate.
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '_' {
            // Find closing underscore that doesn't cross a newline
            if let Some(end) = chars[i+1..].iter().position(|&c| c == '_' || c == '\n') {
                if chars[i + 1 + end] == '_' {
                    let inner: String = chars[i+1..i+1+end].iter().collect();
                    result.push_str("[[");
                    result.push_str(&inner);
                    result.push_str("]]");
                    i += 2 + end;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageKind {
    Notes,
    Photo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkedNote {
    pub title: String,
    pub tags: Vec<String>,
    pub linked_text: String,
}
