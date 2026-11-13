use super::*;
use crate::store::{Place, SHARED, STORE};
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

/// Scripts with the calls of `crate::bindings`, which the data calls are among.
fn with_calls(sources: &[(&str, &str)]) -> Scripts {
    let owned: Vec<(String, String)> = sources
        .iter()
        .map(|(name, source)| (name.to_string(), source.to_string()))
        .collect();
    let (scripts, problems) = Scripts::compile(&owned, crate::bindings::register);
    assert_eq!(problems, Vec::<String>::new(), "the scripts load clean");
    scripts
}

fn called(scripts: &Scripts, hook: &str) -> Result<Dynamic, String> {
    let mut results = scripts.notify(hook, &Map::new());
    results.pop().expect("a script has the hook")
}

#[test]
fn a_script_keeps_data_under_its_own_name() {
    let scripts = with_calls(&[(
        "keeper",
        "fn on_chat(e) { data_add(\"stats\", \"chats\", 2); data_append(\"stats\", \"log\", \"hi\") }
         fn on_death(e) { data_all(\"stats\") }",
    )]);
    let length = called(&scripts, "on_chat").expect("it runs");
    assert_eq!(length.as_int(), Ok(1), "the list has one entry");
    let chats = Place {
        owner: SHARED,
        component: "keeper:stats",
        key: "chats",
    };
    assert_eq!(STORE.get(chats), Some(Value::Int(2)), "under keeper:stats");
    let all = called(&scripts, "on_death").expect("it runs").cast::<Map>();
    assert_eq!(all.len(), 2, "both keys of the data key");
    assert_eq!(all["chats"].as_int(), Ok(2));
}

#[test]
fn another_script_reads_it_by_its_full_name() {
    let scripts = with_calls(&[
        (
            "writer",
            "fn on_chat(e) { data_set_if_absent(\"claims\", \"first\", 7) }",
        ),
        (
            "reader",
            "fn on_death(e) { data_get(\"writer:claims\", \"first\") }
             fn on_spawn(e) { data_get(\"claims\", \"first\") }",
        ),
    ]);
    let claim = |scripts| called(scripts, "on_chat").expect("it runs").as_bool();
    assert_eq!(claim(&scripts), Ok(true), "the first claim wins");
    assert_eq!(claim(&scripts), Ok(false), "the second does not");
    let full = called(&scripts, "on_death").expect("it runs");
    assert_eq!(full.as_int(), Ok(7), "the full name reaches it");
    let short = called(&scripts, "on_spawn").expect("it runs");
    assert!(short.is_unit(), "the short name is the reader's own, empty");
}

#[test]
fn data_refuses_what_it_cannot_keep() {
    let scripts = with_calls(&[(
        "refused",
        "fn on_chat(e) { data_add(\"stats\", \"text\", \"one\") }
         fn on_death(e) { data_get(\"My Stats\", \"x\") }
         fn on_spawn(e) { data_append(\"stats\", \"map\", #{}) }",
    )]);
    let error = |hook| called(&scripts, hook).expect_err("it is refused");
    assert!(error("on_chat").contains("data_add needs a number"));
    assert!(error("on_death").contains("a data key is a-z, 0-9 and _"));
    assert!(error("on_spawn").contains("data takes numbers"));
}

#[test]
fn what_all_share_cannot_be_replaced_or_removed() {
    let sources = [(
        "replacer".to_string(),
        "fn on_chat(e) { data_set(\"stats\", \"x\", 1); }".to_string(),
    )];
    let (scripts, _) = Scripts::compile(&sources, crate::bindings::register);
    let error = called(&scripts, "on_chat").expect_err("there is no such call");
    assert!(error.contains("Function not found"), "{error}");
}

#[test]
fn data_at_the_top_of_a_script_is_a_problem() {
    let sources = [(
        "early".to_string(),
        "data_add(\"stats\", \"loads\", 1);".to_string(),
    )];
    let (_, problems) = Scripts::compile(&sources, crate::bindings::register);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("data is reached from a function"));
}

#[test]
fn a_script_with_another_kind_of_name_is_left_out() {
    let (scripts, problems) = compile(&[("My Script", "fn on_chat(e) {}")]);
    assert_eq!(scripts.len(), 0);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("a-z, 0-9 and _"));
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
        data_get(\"a\", \"\"); data_all(\"a\"); data_add(\"a\", \"\", 1); data_append(\"a\", \"\", 1);
        data_set_if_absent(\"a\", \"\", 1);
        data_get(1, \"a\", \"\"); data_all(1, \"a\"); data_add(1, \"a\", \"\", 1);
        data_append(1, \"a\", \"\", 1); data_set_if_absent(1, \"a\", \"\", 1);
        data_set(1, \"a\", \"\", 1); data_remove(1, \"a\", \"\");
    }";
    let sources = [("calls".to_string(), calls.to_string())];
    let (_, problems) = Scripts::compile(&sources, crate::bindings::register);
    assert_eq!(
        problems,
        Vec::<String>::new(),
        "no call has a name Rhai keeps to itself"
    );
}
