//! A single Markdown file transaction contains both body and note intention.
//! Namespaced, versioned HTML comment is hidden by ordinary Markdown renderers.
//! Unknown/corrupt metadata is rejected, never silently stripped or rewritten.
use crate::domain::intention::DraftingIntention;
const NAMESPACE: &str = "<!-- vocalnotes:intention:";
const PREFIX: &str = "<!-- vocalnotes:intention:v1\n";
const END: &str = "\n-->\n\n";

pub fn encode(source: &str, intention: &DraftingIntention) -> String {
    if intention.is_empty() && !source.starts_with(NAMESPACE) {
        return source.to_owned();
    }
    // Prevent user text from closing the comment, while retaining plain JSON
    // readable by other tools and decoding it losslessly (including CRLF).
    let json = serde_json::to_string(intention)
        .expect("bounded string fields")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");
    format!("{PREFIX}{json}{END}{source}")
}

pub fn decode(document: &str) -> Result<(String, DraftingIntention), &'static str> {
    if !document.starts_with(NAMESPACE) {
        return Ok((document.to_owned(), DraftingIntention::default()));
    }
    let rest = document
        .strip_prefix(PREFIX)
        .ok_or("Version des métadonnées d’intention non prise en charge ; fichier inchangé.")?;
    let (metadata, source) = rest
        .split_once(END)
        .ok_or("Métadonnées d’intention incomplètes ; fichier inchangé.")?;
    if metadata.len() > 32_768 {
        return Err("Métadonnées d’intention trop volumineuses ; fichier inchangé.");
    }
    let intention: DraftingIntention = serde_json::from_str(metadata)
        .map_err(|_| "Métadonnées d’intention invalides ; fichier inchangé.")?;
    if !DraftingIntention::valid_parts(
        intention.purpose(),
        intention.form(),
        intention.tone(),
        intention.context(),
    ) {
        return Err("Intention enregistrée trop volumineuse ; fichier inchangé.");
    }
    Ok((source.to_owned(), intention))
}
