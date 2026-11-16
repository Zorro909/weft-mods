# Plots

## Sizes and allowance

| Size | Blocks | Who gets one |
|---|---|---|
| Small | 16 × 16 | Every player may claim one, free |
| Normal | 32 × 32 | `plot_allowance` of them (one by default); each small plot past the first counts too |
| Large | 64 × 64 | Only from a moderator |

A plot a moderator gave doesn't count against the allowance.

## For players

Stand on the plot and type:

| Command | What it does |
|---|---|
| `/plot info` | Its number, city, size and owner |
| `/plot claim` | Makes a free plot yours, if your allowance has room |
| `/plot unclaim` | Gives yours up; your trusted players go with it |
| `/plot trust <name>` | Lets a player build and use things on your plot |
| `/plot untrust <name>` | Takes that back |
| `/plot show` | Outlines the plots around you for 30 seconds: green yours, blue someone else's, white free |
| `/plot list` | Your plots and where they are |

Right-clicking a plot's post says whose it is; the city stone on the plaza lists the city's free
plots.

## For moderators

| Command | What it does |
|---|---|
| `/plot give <name>` | Gives the plot you stand on to a player, of any size, outside their allowance |
| `/plot take` | Takes it back from its owner |
| `/plot allowance <name> <count>` | How many normal plots that player may claim, 0 to 64 |
| `/plot bypass` | Protection off for you, until you type it again |
