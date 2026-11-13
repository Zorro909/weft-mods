// Greets players, counts visits in its data key welcome:stats, and keeps the chat free of
// one word.

command("visits", "", 0);

fn on_player_joined(e) {
    let visits = data_add("stats", "visits", 1);
    hint(e.player, `Welcome! You are visitor ${visits}.`);
}

fn before_chat(q) {
    if q.text.contains("spam") {
        hint(q.player, "That word stays out of the chat.");
        return "deny";
    }
}

fn cmd_visits(cmd) {
    let visits = data_get("stats", "visits");
    `${visits} visits so far`
}
