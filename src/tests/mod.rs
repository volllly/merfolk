use super::*;

fn setup_empty() -> Mer<backends::Empty> {
    Mer::new()
        .with_backend(backends::Empty {})
        .build()
}

#[test]
fn initializes() {
    setup_empty();
}
