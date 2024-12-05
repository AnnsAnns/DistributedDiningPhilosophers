
use rand::Rng;

/// Create a random port number between 3000 and 9000
pub fn random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(3000..9000)
}

/// Create a random philosopher name
pub fn random_philosopher_name() -> String {
    let names = vec![
        "Aristotle",
        "Plato",
        "Socrates",
        "Kant",
        "Hume",
        "Locke",
        "Descartes",
        "Nietzsche",
        "Wittgenstein",
        "Hegel",
        "Marx",
        "Russell",
        "Heidegger",
        "Kierkegaard",
        "Sartre",
        "Camus",
        "Foucault",
        "Derrida",
        "Deleuze",
        "Zizek",
    ];
    let mut rng = rand::thread_rng();
    let number = rng.gen_range(0..1000);
    let name = format!(
        "{} {} {}",
        names[rng.gen_range(0..names.len())],
        names[rng.gen_range(0..names.len())],
        number
    );
    name.to_string()
}


/// Create a random cutlery name
pub fn random_cutlery_name() -> String {
    let names = ["Fork",
        "Spoon",
        "Knife",
        "Chopsticks",
        "Spork",
        "Splayd",
        "Trongs",
        "Chork",
        "Knork"];
    let mut rng = rand::thread_rng();
    let name = names[rng.gen_range(0..names.len())];
    let number = rng.gen_range(0..1000);
    format!("{} {}", name, number).to_string()
}
