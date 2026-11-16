# Market stalls

## Building one

A stall is sixteen blocks, all of one wood or mixed: oak with a red and white awning, spruce
with blue and white, birch with green and yellow. They go only on a plot you own or are trusted
on.

```text
   awning   A A A          posts   P . P          ground   C C C   crates, behind
            A A A                  P . P                   c R c   counter, register, counter
                                                                   (customers stand here)
```

Place the register last, facing you: it faces the customers, and the rest of the stall is built
around it as above. Right-click any block of it. While a part is missing, its screen says which,
and the stall doesn't trade. A stall that ends up off its owner's plot, for a plot taken back or
given up, closes too.

The register holds everything: breaking it drops the samples, the stock and the till.

## Setting it up

Right-clicking your own stall opens its setup:

- **Offers**, eight rows. Each has what the stall gives on the left and what it gets for it on
  the right. Put a stack in a side's slot to make that side those items, that many of them: 16
  bread means 16 bread a trade. The stack is only a sample; take it back any time. Or type a
  price into the side's field to make it Golden Woof instead; empty, it is the slot's items
  again. Woof for Woof isn't an offer.
- **Stock**, 27 slots: what the stall gives out of.
- **Till**, 27 slots: items customers paid with. Take them out from time to time; a full till
  stops the offers that pay into it.

Woof the stall gets goes straight to your balance, and Woof it gives comes out of it.

| Offer | The stall gives | from | It gets | into |
|---|---|---|---|---|
| Sell for items | 16 bread | stock | 3 iron | till |
| Sell for Woof | 32 wheat | stock | 40 Woof | your balance |
| Buy for Woof | 5 Woof | your balance | 1 wheat | till |
| Buy for items | 1 emerald | stock | 8 wool | till |

## Trading

Anyone else who right-clicks the stall sees its shop: each offer, how many more times the stall
can do it, and a Trade button. A trade happens whole or not at all: the stall gives its side, the
customer pays theirs, from their inventory or balance. If something falls short, the line above
the hotbar says what: the stall's stock or till, its owner's Woof, or the customer's items,
room or Woof. An offer its owner changed while the shop was open asks the customer to look again.

Customers trade from within eight blocks.
