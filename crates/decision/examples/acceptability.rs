//! Standalone sentence scoring through the installed decision model.
use serde_json::json;
use std::path::Path;
use transcrust_decision::ultra::Ultra;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<_> = std::env::args().collect();
    let model = Ultra::load(Path::new(&args[1]))?;
    // Household sentences stay out of the repo (AGENTS.md); the 14-sentence
    // bellwether is ~/syncthing/transcrust-work/window-study/bellwether-14.json.
    let default = dirs::home_dir()
        .unwrap_or_default()
        .join("syncthing/transcrust-work/window-study/bellwether-14.json");
    let path = args.get(3).map(std::path::PathBuf::from).unwrap_or(default);
    let sentences: Vec<String> = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    let probes = [
        (
            "grammar",
            "Assess this sentence independently for standard written English grammar, including capitalization. ",
            [
                "The sentence uses correct English grammar and capitalization.",
                "The sentence violates English grammar or capitalization rules.",
                "There is insufficient evidence to assess its grammar.",
            ],
        ),
        (
            "meaning",
            "Assess whether this sentence makes sense as ordinary, literal English. ",
            [
                "The sentence describes a coherent, sensible situation.",
                "The words combine into an incoherent or nonsensical situation.",
                "There is insufficient context to judge the sentence's meaning.",
            ],
        ),
    ];
    let mut output = Vec::new();
    for (kind, instructions, labels) in probes {
        let labels = labels.map(String::from).to_vec();
        let requests: Vec<_> = sentences
            .iter()
            .map(|sentence| (sentence.to_string(), instructions.to_string(), labels.clone()))
            .collect();
        let scores = model.batch(&requests)?;
        for (sentence, scores) in sentences.iter().zip(scores) {
            println!("{kind}\t{:.3}\t{:.3}\t{:.3}\t{sentence}", scores[0], scores[1], scores[2]);
            output.push(json!({"probe":kind,"sentence":sentence,"instructions":instructions,
                "labels":labels,"scores":scores}));
        }
    }
    std::fs::write(&args[2], serde_json::to_string_pretty(&output)?)?;
    Ok(())
}
