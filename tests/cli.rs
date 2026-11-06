use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
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
    assert_eq!(value["translation"], "The fast cat jumps");

    let alternatives = value["word_alternatives"]
        .as_array()
        .expect("word_alternatives is an array");
    assert_eq!(alternatives.len(), 4);
    assert_eq!(alternatives[0]["word"], "The");
    assert_eq!(
        alternatives[0]["alternatives"],
        serde_json::json!(["A", "That"])
    );
    assert_eq!(alternatives[1]["word"], "fast");
    assert_eq!(
        alternatives[1]["alternatives"],
        serde_json::json!(["quick", "rapid", "swift"])
    );
    assert_eq!(alternatives[2]["word"], "cat");
    assert!(
        output.stderr.is_empty(),
        "unexpected warning: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn translates_spanish_inflections_and_object_pronouns() {
    let output = run(
        &[
            "el gobierno nos miente y el dinero nos manipula",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);

    assert_eq!(value["detected_language"], "es");
    assert_eq!(
        value["translation"],
        "the government lies to us and the money manipulates us"
    );

    let alternatives = value["word_alternatives"]
        .as_array()
        .expect("word_alternatives is an array");
    let alternatives_for = |word: &str| -> Vec<String> {
        alternatives
            .iter()
            .find(|entry| entry["word"] == word)
            .and_then(|entry| entry["alternatives"].as_array())
            .map(|list| {
                list.iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    assert_eq!(alternatives_for("lies"), vec!["fibs", "deceives"]);
    assert_eq!(
        alternatives_for("manipulates"),
        vec!["controls", "exploits"]
    );
}

#[test]
fn translates_english_object_phrases_to_spanish_order() {
    let output = run(
        &[
            "the government lies to us and the money manipulates us",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);

    assert_eq!(value["detected_language"], "en");
    assert_eq!(
        value["translation"],
        "el gobierno nos miente y el dinero nos manipula"
    );
}

#[test]
fn translates_plural_nouns_and_anatomy_vocabulary() {
    let output = run(&["boys has dick and girls has pussy."], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Target (ES): los niños tienen pene y las niñas tienen vagina."));
    assert!(stdout.contains("- niños → (chicos, muchachos)"));
    assert!(stdout.contains("- pene → (falo, miembro)"));
    assert!(stdout.contains("- vagina → (coño, vulva)"));
}

#[test]
fn translates_spanish_articles_and_object_phrases() {
    let output = run(
        &["si la perra esta amarrada. dejenla que llore todo el dia, no la deben de soltar"],
        None,
    );
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(
        "Target (EN): if the bitch is tied up. let her cry all day, you should not let her go"
    ));
    assert!(stdout.contains("- bitch → (cur, mongrel)"));
    assert!(stdout.contains("- tied → (bound, fastened)"));
}

#[test]
fn translates_spanish_plural_and_present_tense_inflections() {
    for (input, expected) in [
        ("camiones", "truck"),
        ("lápices", "pencil"),
        ("hablo", "talk"),
        ("comemos", "eat"),
        ("paises", "countries"),
    ] {
        let output = run(&[input, "--source", "es", "--format", "json"], None);
        let value = json_output(&output);
        assert_eq!(value["translation"], expected, "input: {input}");
    }
}

#[test]
fn translates_spanish_infinitive_verb_lists() {
    let output = run(
        &[
            "alimentar, insertar, cortar, apuntar, disparar, repetir",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);

    assert_eq!(value["detected_language"], "es");
    assert_eq!(value["target_language"], "en");
    assert_eq!(
        value["translation"],
        "feed, insert, cut, aim, shoot, repeat"
    );
}

#[test]
fn translates_technical_prose() {
    let output = run(
        &[
            "The daemon process crashed because of a memory leak in the kernel module.",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(
        value["translation"],
        "El proceso daemon se estrelló debido a una fuga de memoria en el módulo del kernel."
    );

    let output = run(
        &[
            "El servidor se cayó por un error de sintaxis en el script de bash.",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(
        value["translation"],
        "The server crashed because of a syntax error in the bash script."
    );
}

#[test]
fn translates_gaming_prose() {
    let output = run(
        &[
            "El boss tiene un ataque de área que te hace one-shot si no esquivas el hitbox.",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(
        value["translation"],
        "The boss has an area attack that makes you one-shot if you don't dodge the hitbox."
    );

    let output = run(
        &[
            "I need to grind some XP to level up my character and unlock the skill tree.",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(
        value["translation"],
        "Necesito farmear algo de xp para subir de nivel a mi personaje y desbloquear el árbol de habilidades."
    );
}

#[test]
fn translates_shotgun_animation_instructions() {
    let input = "multiple reload shotgun animation should not go far from frame 4 till last shell";
    let output = run(&[input, "--format", "json"], None);
    let value = json_output(&output);
    assert_eq!(value["detected_language"], "en");
    assert_eq!(
        value["translation"],
        "la animación de recarga múltiple de la escopeta no debería alejarse del fotograma 4 hasta el último cartucho"
    );

    let output = run(
        &[
            "la animación de recarga múltiple de la escopeta no debería alejarse del fotograma 4 hasta el último cartucho",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(value["detected_language"], "es");
    assert_eq!(
        value["translation"],
        "multiple reload shotgun animation should not go far from frame 4 till last shell"
    );
}

#[test]
fn translates_slang_and_profanity() {
    let output = run(
        &[
            "Tío, juegas como el culo, te han troleado hasta la madre en el puto lobby.",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(
        value["translation"],
        "Bro, you play like shit, they trolled you hard in the fucking lobby."
    );

    let output = run(
        &[
            "The fucking server crashed again because some dumbass pushed untested code to the main branch.",
            "--format",
            "json",
        ],
        None,
    );
    let value = json_output(&output);
    assert_eq!(
        value["translation"],
        "El puto servidor se estrelló otra vez porque algún idiota subió código sin probar a la rama principal."
    );
}

#[test]
fn translates_my_ass_hurts_phrase() {
    let output = run(&["now my ass hurts.", "--format", "json"], None);
    let value = json_output(&output);

    assert_eq!(value["detected_language"], "en");
    assert_eq!(value["translation"], "ahora me duele el culo.");
}

#[test]
fn translates_prompt_corpus_technical_lines() {
    for (input, expected) in [
        (
            "crear tabla tbl_actividades_mantenimiento_ups que contenga las actividades relacionadas a registros en tbl_mantenimiento_ups con los campos nombre, estado y detalles",
            "create table tbl_actividades_mantenimiento_ups that contains the activities related to records in tbl_mantenimiento_ups with the fields name, state and details",
        ),
        (
            "Implementar módulo de cronograma para agregar mantenimientos a equipos de aire acondicionado",
            "Implement schedule module to add maintenance to air conditioning equipment",
        ),
        (
            "Mostrar unicamente compras en almacenes asignados al usuario",
            "Show only purchases in warehouses assigned to the user",
        ),
        (
            "Exportar todos los resultados a pdf y xlsx basado en products/index_products.vue",
            "Export all results to pdf and xlsx based on products/index_products.vue",
        ),
        (
            "Modificar módulo de salidas @/resources/src/views/app/pages/sales/create_sale.vue y @/resources/src/views/app/pages/sales/edit_sale.vue para permitir duplicados en la lista de productos seleccionados",
            "Modify sales module @/resources/src/views/app/pages/sales/create_sale.vue and @/resources/src/views/app/pages/sales/edit_sale.vue to allow duplicate in the product list selected",
        ),
    ] {
        let output = run(&[input, "--format", "json"], None);
        let value = json_output(&output);
        assert_eq!(value["detected_language"], "es", "input: {input}");
        assert_eq!(value["translation"], expected, "input: {input}");
    }
}

#[test]
fn translates_single_spanish_infinitive_and_gaming_term() {
    for (input, expected) in [("ocultar medkits", "hide medkits"), ("ocultar", "hide")] {
        let output = run(&[input, "--format", "json"], None);
        let value = json_output(&output);
        assert_eq!(value["detected_language"], "es", "input: {input}");
        assert_eq!(value["translation"], expected, "input: {input}");
    }
}

#[test]
fn translates_moon_questions() {
    for (input, expected) in [
        (
            "porque la luna brilla en la noche?",
            "because the moon shines at night?",
        ),
        (
            "por que la luna brilla en la noche?",
            "why does the moon shine at night?",
        ),
    ] {
        let output = run(&[input, "--format", "json"], None);
        let value = json_output(&output);
        assert_eq!(value["detected_language"], "es", "input: {input}");
        assert_eq!(value["translation"], expected, "input: {input}");
    }
}

#[test]
fn translates_common_ambiguous_phrases() {
    for (input, expected) in [
        ("el papel", "the paper"),
        ("la mañana", "the morning"),
        ("el vino", "the wine"),
        ("a veces", "sometimes"),
    ] {
        let output = run(&[input, "--format", "json"], None);
        let value = json_output(&output);
        assert_eq!(value["detected_language"], "es", "input: {input}");
        assert_eq!(value["translation"], expected, "input: {input}");
    }
}

#[test]
fn preserves_code_identifiers() {
    let input = "user.name hello.world _private CAT_2 hello123 api/v2/users";
    let output = run(&[input, "--source", "en", "--format", "json"], None);
    let value = json_output(&output);

    assert_eq!(value["translation"], input);
}

#[test]
fn translates_unaccented_and_typographic_input() {
    for (input, expected) in [
        ("el gato rapido salta", "the fast cat jumps"),
        ("don’t worry", "no te preocupes"),
        ("I’m sorry", "Lo siento"),
    ] {
        let output = run(&[input, "--format", "json"], None);
        let value = json_output(&output);
        assert_eq!(value["translation"], expected, "input: {input}");
    }
}

#[test]
fn removes_inverted_spanish_punctuation() {
    let output = run(&["¿Dónde está la farmacia?", "--format", "json"], None);
    let value = json_output(&output);

    assert_eq!(value["translation"], "Where is the pharmacy?");
}

#[test]
fn renders_human_readable_output() {
    let output = run(&["El gato rápido salta."], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Source (Detected: ES): El gato rápido salta."));
    assert!(stdout.contains("Target (EN): The fast cat jumps."));
    assert!(stdout.contains("Word Alternatives:"));
    assert!(stdout.contains("- The → (A, That)"));
    assert!(stdout.contains("- fast → (quick, rapid, swift)"));
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
    assert!(
        output.stderr.is_empty(),
        "dictionary words should not trigger a detection warning"
    );
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
    assert_eq!(value["translation"], "El gato rápido salta");
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

#[test]
fn does_not_fail_when_stdout_closes_early() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock is after 1970")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "translate-cli-broken-pipe-{}-{unique}.txt",
        std::process::id()
    ));
    fs::write(&path, "hello world ".repeat(10_000)).expect("test file is written");

    let mut child = Command::new(env!("CARGO_BIN_EXE_translate-cli"))
        .args(["--file", path.to_str().expect("utf-8 path")])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary starts");

    let mut stdout = child.stdout.take().expect("stdout is piped");
    stdout
        .read_exact(&mut [0u8; 1])
        .expect("binary writes output");
    drop(stdout);

    let output = child.wait_with_output().expect("binary finishes");
    fs::remove_file(&path).expect("test file is removed");

    assert!(
        output.status.success(),
        "closed pipe caused an error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
