// Greets players, counts visits in its component welcome:stats on the name "server", and
// keeps the chat free of one word.

command("visits", "", 0);

fn on_player_joined(e) {
    let stats = component("server", "stats") ?? #{ visits: 0 };
    stats.visits += 1;
    set_component("server", "stats", stats);
    hint(e.player, `Welcome! You are visitor ${stats.visits}.`);
}

fn before_chat(q) {
    if q.text.contains("spam") {
        hint(q.player, "That word stays out of the chat.");
        return "deny";
    }
}

fn cmd_visits(cmd) {
    let stats = component("server", "stats") ?? #{ visits: 0 };
    `${stats.visits} visits so far`
}
