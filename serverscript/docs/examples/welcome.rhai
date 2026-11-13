// Greets players, counts visits, and keeps the chat free of one word.

command("visits", "", 0);

fn on_player_joined(e) {
    let visits = store_add("visits", 1);
    hint(e.player, `Welcome! You are visitor ${visits}.`);
}

fn before_chat(q) {
    if q.text.contains("spam") {
        hint(q.player, "That word stays out of the chat.");
        return "deny";
    }
}

fn cmd_visits(cmd) {
    let visits = store_get("visits");
    `${visits} visits so far`
}
