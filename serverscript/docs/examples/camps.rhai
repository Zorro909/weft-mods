// /setcamp remembers where a player stands, as that player's own data, which is saved.
// /camp brings them back, and /spawn takes anyone to a fixed place.

command("spawn", "", 0);
command("setcamp", "", 0);
command("camp", "", 0);

fn cmd_spawn(cmd) {
    teleport(cmd.sender, 0.5, 80, 0.5);
    "Back at the spawn"
}

fn cmd_setcamp(cmd) {
    let here = position(cmd.sender);
    if !data_set(cmd.sender, "camp", here) {
        return "Your camp could not be saved";
    }
    "Camp set"
}

fn cmd_camp(cmd) {
    let camp = data_get(cmd.sender, "camp");
    if camp == () {
        return "Set a camp first with /setcamp";
    }
    teleport(cmd.sender, camp[0], camp[1], camp[2]);
    "Back at your camp"
}
