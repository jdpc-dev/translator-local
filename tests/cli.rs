use serde_json::Value;
use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn run(args: &[&str], stdin: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_translate-cli"));
    command.args(args);

    match stdin {
        Some(input) => {
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = command.spawn().expect("binary starts");
            child
                .stdin
                .as_mut()
                .expect("stdin is piped")
                .write_all(input.as_bytes())
                .expect("stdin accepts input");
            child.wait_with_output().expect("binary finishes")
        }
        None => command.output().expect("binary runs"),
    }
}

fn json_output(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is valid JSON")
}

#[test]
fn translates_the_spec_example_as_json() {
    let output = run(&["El gato rápido salta", "--format", "json"], None);
    let value = json_output(&output);

    assert_eq!(value["source_text"], "El gato rápido salta");
    assert_eq!(value["detected_language"], "es");
    assert_eq!(value["target_language"], "en");
    assert_eq!(value["translation"], "The cat fast jumps");

    let alternatives = value["word_alternatives"]
        .as_array()
        .expect("word_alternatives is an array");
    assert_eq!(alternatives.len(), 4);
    assert_eq!(alternatives[0]["word"], "The");
    assert_eq!(
        alternatives[0]["alternatives"],
        serde_json::json!(["A", "That"])
    );
    assert_eq!(
        alternatives[2]["alternatives"],
        serde_json::json!(["quick", "rapid", "swift"])
    );
    assert!(
        output.stderr.is_empty(),
        "unexpected warning: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn renders_human_readable_output() {
    let output = run(&["El gato rápido salta."], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Source (Detected: ES): El gato rápido salta."));
    assert!(stdout.contains("Target (EN): The cat fast jumps."));
    assert!(stdout.contains("Word Alternatives:"));
    assert!(stdout.contains("- The → (A, That)"));
    assert!(stdout.contains("- cat → (feline, kitty)"));
}

#[test]
fn reads_from_stdin() {
    let output = run(&["--format", "json"], Some("good morning\n"));
    let value = json_output(&output);

    assert_eq!(value["source_text"], "good morning");
    assert_eq!(value["detected_language"], "en");
    assert_eq!(value["target_language"], "es");
    assert_eq!(value["translation"], "buenos días");
    assert!(!output.stderr.is_empty(), "ambiguous detection should warn");
}

#[test]
fn reads_from_a_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after 1970")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "translate-cli-test-{}-{unique}.txt",
        std::process::id()
    ));
    fs::write(&path, "The fast cat jumps\n").expect("test file is written");

    let output = run(
        &[
            "--file",
            path.to_str().expect("utf-8 path"),
            "--format",
            "json",
        ],
        None,
    );
    fs::remove_file(&path).expect("test file is removed");

    let value = json_output(&output);
    assert_eq!(value["source_text"], "The fast cat jumps");
    assert_eq!(value["translation"], "El rápido gato salta");
    assert_eq!(value["target_language"], "es");
}

#[test]
fn explicit_source_overrides_detection() {
    let output = run(&["gato", "--source", "es", "--format", "json"], None);
    let value = json_output(&output);

    assert_eq!(value["detected_language"], "es");
    assert_eq!(value["translation"], "cat");
}

#[test]
fn target_can_be_requested_explicitly() {
    let output = run(&["hello", "--target", "es", "--format", "json"], None);
    let value = json_output(&output);

    assert_eq!(value["target_language"], "es");
    assert_eq!(value["translation"], "hola");
}

#[test]
fn rejects_a_target_equal_to_the_source() {
    let output = run(&["hello", "--source", "en", "--target", "en"], None);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must differ"), "stderr was: {stderr}");
}

#[test]
fn brackets_out_of_vocabulary_words_on_request() {
    let output = run(
        &["hello zorblax", "--oov", "bracket", "--format", "json"],
        None,
    );
    let value = json_output(&output);

    assert_eq!(value["translation"], "hola [zorblax]");
}

#[test]
fn help_documents_the_known_limitations() {
    let output = run(&["--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Known limitations:"));
    assert!(stdout.contains("--oov"));
}

#[test]
fn rejects_empty_input() {
    let output = run(&[], Some(""));

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("input text is empty"));
}
