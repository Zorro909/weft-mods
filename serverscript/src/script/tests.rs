use super::*;
use crate::store::STORE;
use crate::value::Value;

fn compile(sources: &[(&str, &str)]) -> (Scripts, Vec<String>) {
    let owned: Vec<(String, String)> = sources
        .iter()
        .map(|(name, source)| (name.to_string(), source.to_string()))
        .collect();
    Scripts::compile(&owned, |_| {})
}

fn loaded(source: &str) -> Scripts {
    let (scripts, problems) = compile(&[("test", source)]);
    assert_eq!(problems, Vec::<String>::new(), "the script loads clean");
    scripts
}

fn argument(key: &str, value: i64) -> Map {
    Map::from([(key.into(), Dynamic::from(value))])
}

#[test]
fn a_hook_gets_its_argument_and_gives_its_answer() {
    let scripts = loaded("fn before_chat(q) { if q.length > 3 { return \"deny\"; } }");
    let said = |length| scripts.notify("before_chat", &argument("length", length));
    let long = said(9)
        .pop()
        .expect("the script has the hook")
        .expect("it runs");
    assert_eq!(long.into_string().as_deref(), Ok("deny"));
    let short = said(2)
        .pop()
        .expect("the script has the hook")
        .expect("it runs");
    assert!(short.is_unit(), "nothing said is nothing given back");
}

#[test]
fn only_scripts_with_the_hook_are_called() {
    let (scripts, _) = compile(&[("a", "fn on_chat(e) { 1 }"), ("b", "fn on_death(e) { 2 }")]);
    assert_eq!(scripts.len(), 2);
    assert!(scripts.has("on_chat") && !scripts.has("on_player_joined"));
    assert_eq!(scripts.notify("on_chat", &Map::new()).len(), 1);
    assert!(scripts.notify("on_player_joined", &Map::new()).is_empty());
}

#[test]
fn a_command_is_declared_at_the_top_and_run_by_its_function() {
    let scripts =
        loaded("command(\"pay\", \"<amount:int>\", 0);\nfn cmd_pay(cmd) { cmd.amount * 2 }");
    let command = scripts.command("pay").expect("the script declared it");
    assert_eq!(
        (command.usage_text.as_str(), command.permission),
        ("<amount:int>", 0)
    );
    let result = scripts
        .run(command, argument("amount", 21))
        .expect("it runs");
    assert_eq!(result.as_int(), Ok(42));
}

#[test]
fn a_script_that_does_not_compile_is_left_out_and_named() {
    let (scripts, problems) =
        compile(&[("broken", "fn on_chat(e) {"), ("fine", "fn on_chat(e) {}")]);
    assert_eq!(scripts.len(), 1, "the other still loads");
    assert_eq!(problems.len(), 1);
    assert!(
        problems[0].starts_with("broken: "),
        "{} names the script",
        problems[0]
    );
}

#[test]
fn a_command_that_cannot_work_is_left_out_and_says_why() {
    let cases = [
        (
            "command(\"Pay\", \"\", 0); fn cmd_Pay(cmd) {}",
            "a-z, 0-9 and _",
        ),
        (
            "command(\"pay\", \"<n:number>\", 0); fn cmd_pay(cmd) {}",
            "unknown parameter type number",
        ),
        (
            "command(\"pay\", \"\", 9); fn cmd_pay(cmd) {}",
            "permission is 0 to 4",
        ),
        ("command(\"pay\", \"\", 0);", "no fn cmd_pay(cmd)"),
        (
            "command(\"scripts\", \"\", 0); fn cmd_scripts(cmd) {}",
            "the mod's own command",
        ),
        (
            "command(\"pay\", \"\", 0); command(\"pay\", \"\", 0); fn cmd_pay(cmd) {}",
            "declared twice",
        ),
    ];
    for (source, reason) in cases {
        let (scripts, problems) = compile(&[("test", source)]);
        assert_eq!(problems.len(), 1, "{source} has one problem");
        assert!(
            problems[0].contains(reason),
            "{} says {reason}",
            problems[0]
        );
        assert!(scripts.commands().len() <= 1, "the bad one is left out");
    }
}

#[test]
fn a_command_declared_inside_a_function_is_an_error() {
    let scripts = loaded("fn on_chat(e) { command(\"late\", \"\", 0); }");
    let result = scripts
        .notify("on_chat", &Map::new())
        .pop()
        .expect("the script has the hook");
    let error = result.expect_err("declaring is for the top of a script");
    assert!(
        error.contains("test: on_chat: "),
        "{error} names script and function"
    );
    assert!(
        error.contains("top of a script"),
        "{error} says where it belongs"
    );
}

#[test]
fn a_function_sees_nothing_outside_itself() {
    let scripts = loaded("let seen = 0;\nfn on_chat(e) { seen += 1; seen }");
    let result = scripts
        .notify("on_chat", &Map::new())
        .pop()
        .expect("the script has the hook");
    assert!(
        result.is_err(),
        "a script's top-level variable is not shared state"
    );
}

#[test]
fn a_loop_without_end_is_stopped() {
    let scripts = loaded("fn on_chat(e) { loop {} }");
    let result = scripts
        .notify("on_chat", &Map::new())
        .pop()
        .expect("the script has the hook");
    assert!(
        result
            .expect_err("it never ends")
            .contains("Too many operations")
    );
}

#[test]
fn scripts_share_the_store() {
    let (scripts, problems) = compile(&[
        (
            "writer",
            "fn on_chat(e) { store_add(\"script_test_chats\", 2); store_append(\"script_test_log\", \"hi\") }",
        ),
        (
            "reader",
            "fn on_death(e) { store_get(\"script_test_chats\") }",
        ),
        (
            "claimer",
            "fn on_spawn(e) { store_set_if_absent(\"script_test_claim\", true) }",
        ),
    ]);
    assert_eq!(problems, Vec::<String>::new());
    let call = |hook| {
        scripts
            .notify(hook, &Map::new())
            .pop()
            .expect("a script has it")
            .expect("it runs")
    };
    assert_eq!(call("on_chat").as_int(), Ok(1), "the list has one entry");
    assert_eq!(
        call("on_death").as_int(),
        Ok(2),
        "another script reads the total"
    );
    assert_eq!(call("on_spawn").as_bool(), Ok(true), "the first claim wins");
    assert_eq!(call("on_spawn").as_bool(), Ok(false), "the second does not");
    assert_eq!(
        STORE.get("script_test_log"),
        Some(Value::List(vec![Value::Text("hi".into())]))
    );
}

#[test]
fn the_store_refuses_what_it_cannot_add() {
    let scripts = loaded("fn on_chat(e) { store_add(\"script_test_text\", \"one\") }");
    let result = scripts
        .notify("on_chat", &Map::new())
        .pop()
        .expect("the script has the hook");
    assert!(
        result
            .expect_err("text is no number")
            .contains("store_add needs a number")
    );
}

#[test]
fn a_folder_is_loaded_in_the_order_of_its_names() {
    let folder = std::env::temp_dir().join(format!("serverscript-test-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("the test folder is made");
    let write = |name: &str, source: &str| {
        std::fs::write(folder.join(name), source).expect("a file is written")
    };
    write(
        "b.rhai",
        "command(\"second\", \"\", 0); fn cmd_second(cmd) {}",
    );
    write(
        "a.rhai",
        "command(\"first\", \"\", 0); fn cmd_first(cmd) {}",
    );
    write("notes.txt", "not a script");
    let (scripts, problems) = Scripts::load(&folder, |_| {});
    std::fs::remove_dir_all(&folder).expect("the test folder is removed");
    assert_eq!(problems, Vec::<String>::new());
    let names: Vec<&str> = scripts
        .commands()
        .iter()
        .map(|command| command.name.as_str())
        .collect();
    assert_eq!(names, ["first", "second"], "other files are left alone");
}

#[test]
fn a_folder_that_is_not_there_is_a_problem_and_no_scripts() {
    let (scripts, problems) = Scripts::load(Path::new("/nonexistent/serverscript"), |_| {});
    assert_eq!(scripts.len(), 0);
    assert!(
        problems[0].contains("can't be read"),
        "{} says so",
        problems[0]
    );
}

#[test]
fn the_example_scripts_load_clean() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/examples");
    let (scripts, problems) = Scripts::load(&folder, crate::bindings::register);
    assert_eq!(problems, Vec::<String>::new());
    assert_eq!(scripts.len(), 3, "the examples are there");
    for name in [
        "spawn", "setcamp", "camp", "visits", "kit", "carried", "pillar",
    ] {
        assert!(
            scripts.command(name).is_some(),
            "an example declares /{name}"
        );
    }
}

#[test]
fn every_call_can_be_written_in_a_script() {
    let calls = "fn on_chat(e) {
        hint(1, \"\"); chat(1, \"\"); title(1, \"\"); broadcast(\"\"); log(\"\");
        block(0, 0, 0); set_block(0, 0, 0, \"\"); time(); set_time(0);
        players(); position(1); teleport(1, 0, 0, 0); permission(1); kind(1); health(1);
        damage(1, 1); heal(1, 1); summon(\"\", 0, 0, 0); remove(1);
        inventory(1); held(1); give(1, \"\", 1); take(1, \"\", 1);
        data_get(1, \"\"); data_set(1, \"\", 1); data_remove(1, \"\");
        store_get(\"\"); store_add(\"\", 1); store_append(\"\", 1); store_set_if_absent(\"\", 1);
    }";
    let sources = [("calls".to_string(), calls.to_string())];
    let (_, problems) = Scripts::compile(&sources, crate::bindings::register);
    assert_eq!(
        problems,
        Vec::<String>::new(),
        "no call has a name Rhai keeps to itself"
    );
}
