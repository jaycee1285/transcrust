// Standalone scorer for this numbered read-aloud script; no model dependencies.
use std::{env, fs};

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let script = fs::read_to_string(&args[0]).unwrap();
    let reference = words(
        &script.lines().filter_map(|line| {
            let (number, text) = line.split_once(". ")?;
            number.parse::<usize>().ok().map(|_| text)
        }).collect::<Vec<_>>().join(" "),
    );
    assert!(!reference.is_empty());
    println!("file\terrors\treference_words\twer_percent");
    for path in &args[1..] {
        let hypothesis = words(&fs::read_to_string(path).unwrap());
        let mut row: Vec<usize> = (0..=hypothesis.len()).collect();
        for (i, expected) in reference.iter().enumerate() {
            let mut diagonal = row[0];
            row[0] = i + 1;
            for (j, actual) in hypothesis.iter().enumerate() {
                let above = row[j + 1];
                row[j + 1] = (diagonal + usize::from(expected != actual))
                    .min(above + 1).min(row[j] + 1);
                diagonal = above;
            }
        }
        let errors = row[hypothesis.len()];
        println!("{}\t{}\t{}\t{:.2}", path, errors, reference.len(),
            100.0 * errors as f64 / reference.len() as f64);
    }
}
