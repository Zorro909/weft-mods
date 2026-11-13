// /setcamp remembers where a player stands, in the data key camps:camp on that player.
// /camp brings them back and counts the returns there too, and /spawn takes anyone to a
// fixed place.

command("spawn", "", 0);
command("setcamp", "", 0);
command("camp", "", 0);

fn cmd_spawn(cmd) {
    teleport(cmd.sender, 0.5, 80, 0.5);
    "Back at the spawn"
}

fn cmd_setcamp(cmd) {
    if !data_set(cmd.sender, "camp", "at", position(cmd.sender)) {
        return "Your camp could not be saved";
    }
    "Camp set"
}

fn cmd_camp(cmd) {
    let at = data_get(cmd.sender, "camp", "at");
    if at == () {
        return "Set a camp first with /setcamp";
    }
    teleport(cmd.sender, at[0], at[1], at[2]);
    let returns = data_add(cmd.sender, "camp", "returns", 1);
    `Back at your camp, return number ${returns}`
}
