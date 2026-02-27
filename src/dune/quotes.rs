const QUOTES: &[&str] = &[
    "Fear is the mind-killer.",
    "The spice must flow.",
    "He who controls the spice controls the universe.",
    "I must not fear. Fear is the mind-killer.",
    "The mystery of life isn't a problem to solve, but a reality to experience.",
    "Without change, something sleeps inside us and seldom awakens.",
    "The sleeper must awaken.",
    "Beginnings are such delicate times.",
    "Deep in the human unconscious is a pervasive need for a logical universe.",
    "Survival is the ability to swim in strange water.",
    "God created Arrakis to train the faithful.",
    "The mind commands the body and it obeys.",
    "There is no escape — we pay for the violence of our ancestors.",
    "A process cannot be understood by stopping it.",
    "Arrakis teaches the attitude of the knife.",
];

pub fn get_quote(index: usize) -> &'static str {
    QUOTES[index % QUOTES.len()]
}

pub fn count() -> usize {
    QUOTES.len()
}
