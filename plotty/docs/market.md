# Market stalls

## Building one

A stall is sixteen blocks, all of one wood or mixed: oak with a red and white awning, spruce
with blue and white, birch with green and yellow. They go only on a plot you own or are trusted
on, or on a market place you rent.

```text
   awning   A A A          posts   P . P          ground   C C C   crates, behind
            A A A                  P . P                   c R c   counter, register, counter
                                                                   (customers stand here)
```

Place the register last, facing you: it faces the customers, and the rest of the stall is built
around it as above. Right-click any block of it. While a part is missing, its screen says which,
and the stall doesn't trade. A stall that ends up off its owner's plot, for a plot taken back or
given up, closes too, and so does one whose market place's rent has ended.

The register holds everything: breaking it drops the samples, the stock and the till. Only its
owner can break it, or a moderator with `/plot bypass`; a friend trusted on the plot can't.

## Market places

A market square has six places outlined in stone bricks, each with a rent post at its corner
toward the middle. Right-click a post, or stand on a place and type `/market info`, to see who
rents it or what it costs.

| Command | What it does |
|---|---|
| `/market rent 1`, `7` or `30` | Rents the place you stand on for that many real days, or adds them to your rent. 10, 50 or 150 Golden Woof by default (`rent_woof`) |
| `/market info` | Who rents the place you stand on, and for how long |
| `/market places` | The places you rent, and the time left |

The rent leaves the game: nobody gets it. On a place you rent you may build a stall and nothing
else. When the time is up the stall closes, but you can still open it, empty it, break it, or
renew. After `rent_grace_days` (3) more days the place is free for anyone; a stall still on it
has to be cleared by a moderator before someone else can rent it. A market square with a place
rented can't be undone.

## Setting it up

Right-clicking your own stall opens its setup:

- **Offers**, eight rows, five in view at a time: the mouse wheel scrolls to the rest. Each has
  what the stall gives on the left and what it gets for it on the right. Put a stack in a side's
  slot to make that side those items, that many of them: 16 bread means 16 bread a trade. The
  stack is only a sample; take it back any time. Or type a price into the side's field to make it
  Golden Woof instead; empty, it is the slot's items again. Woof for Woof isn't an offer.
- **Stock**, 27 slots beside the offers: what the stall gives out of.
- **Till**, 27 slots beside the stock: items customers paid with. Take them out from time to
  time; a full till stops the offers that pay into it.

Woof the stall gets goes straight to your balance, and Woof it gives comes out of it.

| Offer | The stall gives | from | It gets | into |
|---|---|---|---|---|
| Sell for items | 16 bread | stock | 3 iron | till |
| Sell for Woof | 32 wheat | stock | 40 Woof | your balance |
| Buy for Woof | 5 Woof | your balance | 1 wheat | till |
| Buy for items | 1 emerald | stock | 8 wool | till |

## Trading

Anyone else who right-clicks the stall sees its shop: each offer, how many more times the stall
can do it, and a Trade button, four offers in view at a time and the rest a scroll away. A trade
happens whole or not at all: the stall gives its side, the customer pays theirs, from their
inventory or balance. If something falls short, the line above the hotbar says what: the stall's
stock or till, its owner's Woof, or the customer's items, room or Woof. An offer its owner changed
while the shop was open asks the customer to look again.

Customers trade from within eight blocks. When the owner next joins, a line says how often their
stalls traded since they last did.
