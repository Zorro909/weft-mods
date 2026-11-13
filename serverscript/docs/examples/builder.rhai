// /kit hands every player a few things once, /carried counts an item in what they carry, and
// /pillar builds a column of stone where they stand, in the air above them only.

command("kit", "", 0);
command("carried", "<item:item>", 0);
command("pillar", "<height:int>", 2);

fn cmd_kit(cmd) {
    if !data_set_if_absent(cmd.sender, "kit", "given", true) {
        return "You had your kit already";
    }
    let left = give(cmd.sender, "base:torch", 16) + give(cmd.sender, "base:bread", 4);
    if left > 0 {
        return `Your pockets are full: ${left} things did not fit`;
    }
    "Torches and bread, for the first night"
}

fn cmd_carried(cmd) {
    let count = 0;
    for stack in inventory(cmd.sender) {
        if stack.item == cmd.item {
            count += stack.count;
        }
    }
    `You carry ${count} of ${cmd.item}`
}

fn cmd_pillar(cmd) {
    let feet = position(cmd.sender);
    let top = feet[1];
    let built = 0;
    for step in 0..cmd.height.min(32) {
        let y = feet[1] + step;
        let free = block(feet[0], y, feet[2]) == "core:air";
        if free && set_block(feet[0], y, feet[2], "base:stone") {
            built += 1;
            top = y.floor() + 1;
        }
    }
    teleport(cmd.sender, feet[0], top, feet[2]);
    `A pillar of ${built} blocks`
}
