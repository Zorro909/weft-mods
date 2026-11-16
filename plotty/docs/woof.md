# Golden Woof

Golden Woof is the server's money. It is no item: each player has a balance the server keeps,
also while they are offline, so it can't be dropped, stolen, duplicated or lost in death. The
inventory shows it in a panel beside it.

| Command | Who | What it does |
|---|---|---|
| `/woof` | everyone | Your balance |
| `/woof pay <name> <amount>` | everyone | Pays a player who has joined before, online or not |
| `/woof <name>` | moderator | That player's balance |
| `/woof give <name> <amount>` | game master | Gives a player Woof from outside the game |
| `/woof take <name> <amount>` | game master | Takes it out of the game; not more than they have |

A player joining for the first time gets `starting_woof` (100 by default, [settings](admin.md#settings)).
Besides that, Woof comes into the game only from game masters. A balance holds at most a million
million.

Every payment moves all of its amount or none, one at a time on the whole server, and is on disk
when the command answers.
