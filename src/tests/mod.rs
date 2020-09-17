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

#[test]
#[cfg(feature = "backends")]
fn empty_encode() {
    use crate::backends::*;
    use crate::interfaces::backend::*;

    let rx: String = Empty {}.encode(Tx::<String> { procedure: "", payload: String::from("test")}).unwrap();

    assert_eq!(rx, "\"\"");
}

#[test]
#[cfg(feature = "backends")]
fn empty_decode() {
    use crate::backends::*;
    use crate::interfaces::backend::*;

    let rx: Rx<String> = Empty {}.decode(&String::from("\"test\"")).unwrap();

    assert_eq!(rx.payload, "test");
}
