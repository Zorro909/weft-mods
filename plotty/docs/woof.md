# Golden Woof

Golden Woof is the server's currency. It isn't an item: the server keeps a balance per player,
even while they are offline, so it can't be dropped, stolen, duplicated or lost on death. Your
balance appears in a panel beside your inventory.

| Command | Who | What it does |
|---|---|---|
| `/woof` | everyone | Your balance |
| `/woof pay <name> <amount>` | everyone | Pays a player who has joined before, online or not |
| `/woof <name>` | moderator | That player's balance |
| `/woof give <name> <amount>` | game master | Gives a player Woof from outside the game |
| `/woof take <name> <amount>` | game master | Takes it out of the game; not more than they have |

A player joining for the first time gets `starting_woof` (100 by default; see
[settings](admin.md#settings)). Besides that, Woof enters the game only from game masters. A
balance holds at most 1,000,000,000,000 (one trillion).

Every payment moves all of its amount or none, one at a time across the server, and is written to
its table file before the command replies.
