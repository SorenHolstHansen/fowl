pub fn a_or_an(word: &str) -> String {
    if ['a', 'e', 'i', 'o', 'u', 'y'].contains(&word.chars().next().unwrap()) {
        return "an".to_string();
    }

    "a".to_string()
}
