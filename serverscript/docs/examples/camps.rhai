// /setcamp remembers where a player stands, in the component camps:camp on that player.
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
    let camp = #{ at: position(cmd.sender), returns: 0 };
    if !set_component(cmd.sender, "camp", camp) {
        return "Your camp could not be saved";
    }
    "Camp set"
}

fn cmd_camp(cmd) {
    let camp = component(cmd.sender, "camp");
    if camp == () {
        return "Set a camp first with /setcamp";
    }
    teleport(cmd.sender, camp.at[0], camp.at[1], camp.at[2]);
    camp.returns += 1;
    set_component(cmd.sender, "camp", camp);
    `Back at your camp, return number ${camp.returns}`
}
