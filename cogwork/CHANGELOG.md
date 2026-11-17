# Changelog

## Unreleased

### Added

- Every machine's screen shows what it used in the last tick, "Using 120 energy a tick", and how
  far short of what it wanted its supply fell, "short 30": the excavator, the leveler, the
  assembly bench and the pump. The oil engine's screen shows what it sends into its drives.
- Rubble and Ore Rubble. Ore Rubble is of one metal, any mod's (an ore tagged `ores/<metal>`
  with a raw ore tagged `raw_ores/<metal>`), at a grade from Poor to Pure; its tooltip and
  flecks show which.
- The millstone grinds Ore Rubble into raw ore by hand, with water and sand: six turns of its
  runner an Ore Rubble, one raw ore of Poor up to 2.5 of Pure.
- The sieve sifts Rubble by hand into sand, gravel, flint and clay, and rarely coal, redstone,
  diamond or Poor iron or gold Ore Rubble.
- The Timber Quarry, a small first quarry of wood and string: it lashes a frame of logs and
  string from its frame slots around a 5 by 5 area (up to 7 by 7 with markers) and digs it 16
  deep with a drill bit, bringing up one Ore Rubble an ore block. Its frame gives the logs back
  when broken.
- Stone and iron drill bits for the Timber Quarry. A bit breaks what its pickaxe breaks and
  wears out; what it can't break, such as gold ore under a stone bit, is left standing.
- The water wheel: over flowing water it turns and makes 12 energy a tick, pushed out of its
  axle.
- The wooden axle, a cheap wooden torque shaft.
- Spring engines are wound by hand: use one with an empty hand to turn its crank, 200 energy a
  turn, up to 10,000, which runs it down at 10 a tick.
- The shaker screen sifts Rubble with engine energy, as the sieve does: tubes bring the Rubble,
  and what it sifts out leaves by its front.
- The grinding mill grinds Ore Rubble into raw ore with engine energy, as the millstone does,
  with water by liquid tube and sand by tube; the raw ore leaves by its front.
- The hand bellows: placed against a furnace and pumped with an empty hand, it makes the furnace
  cook twice as fast for a minute.
- The driven bellows does the same for as long as an engine feeds it 20 energy a tick, its cogs
  pumping the bag.
- The Iron Quarry, the quarry after the Timber Quarry: it builds a frame of iron bars, an iron
  ingot a block that comes back when the frame is broken, around a 9 by 9 area (up to 17 by 17
  with markers) and digs it 64 deep with an iron drill bit at up to 60 energy a tick, bringing up
  a quarter more Ore Rubble than the Timber Quarry.
- The washer washes Poor Ore Rubble into Common with engine energy and water, its paddle wheel
  churning the trough: tubes bring the Ore Rubble, a liquid tube the water, and the washed
  rubble leaves by its front.
- The coke oven, a 3 by 3 by 3 box of coke oven bricks round a hollow, cooks coal into coke
  eight at a time with no energy, with creosote, a new liquid, and a sulfur every eight coal.
  Coke burns twice as long as coal. A drain in its wall lets the creosote out into liquid tubes.
- The blast furnace, of fire bricks 3 by 3 and 4 high, smelts an iron ingot and a coke into a
  steel bloom and slag while a driven bellows blows into it at 80 energy a tick.
- The forge anvil and forge hammer: five strikes beat a steel bloom into a steel ingot. Steel
  ingots pack into a block of steel.
- The steel drill bit, for the Iron Quarry: it breaks what a diamond pickaxe breaks, at up to 90
  energy a tick, for 8,000 blocks.
- The sieve and the shaker screen sift slag into gravel, and now and then a Poor iron Ore Rubble.
- The firebox and the boiler on it: the firebox burns any fuel, lava buckets and lava by liquid
  tube too, while the boiler on it boils water from liquid tubes into steam, 20 mB a tick, let
  out of its top. Its door glows while it burns.
- Steam, a new liquid that spreads through liquid tubes and tanks from fuller to emptier,
  whatever their height.
- The steam engine runs on 20 mB of steam a tick for 160 energy, pushed out of both ends of its
  flywheel's axle; its piston and flywheel turn and it puffs steam while it runs.
- The trip hammer forges steel blooms from tubes into steel ingots on steam, five strikes a
  bloom, the automatic counterpart of the forge anvil.
- The Steel Excavator, the quarry after the Iron Quarry: it builds a frame of steel lattice, a
  steel ingot every four blocks, around a 9 by 9 area (up to 63 by 63 with markers) and digs it
  down to the world's `excavator_lowest_y` with a steel or diamond-tipped drill bit at up to 120
  or 160 energy a tick, bringing up 1.5 Ore Rubble an ore block. The frame block that took an
  ingot gives it back when broken.
- The diamond-tipped drill bit, a steel bit set with diamonds, for the Steel Excavator: 40,000
  blocks.
- The flotation cell floats Common Ore Rubble into Rich with engine energy, water and creosote,
  its rotor stirring up a froth on top: tubes bring the Ore Rubble, liquid tubes the water, and
  it draws creosote from a tube, tank or coke oven drain beside it. Rich Ore Rubble leaves by its
  front and muddy water by its back.
- Muddy water, a new liquid, with its bucket.
- The clarifier settles muddy water back into water, slowly and with no energy, its rake turning
  over its floor: 900 mB of water from every 1,000, and a one in ten chance of a clay ball.
- The core drill, for prospecting: crank it sixteen times and it reports in chat how many blocks
  of each ore lie in the 16 by 16 below it, in bands of 16 layers down to 128 below.
- The refinery, a steel column 3 by 3 and 6 high, distils 10 mB of crude oil a tick, heated by
  40 mB of steam or 300 energy a tick in a heater at its foot and cooled by 50 mB of water a tick
  in a condenser on top. In its temperature band a bucket of crude gives 200 mB of gas, 400 of
  diesel, 200 of lubricant, 200 of tar and a sulfur, out of the taps on each product's layer; too
  cold it all goes to tar, too hot the gas vents. Its screen shows the band.
- Gas, diesel, lubricant and tar, new liquids, with their buckets.
- The oil engine burns diesel, 1 mB a tick for its 600 energy, ahead of oil.
- Lubricant led into a quarry by liquid tube makes each block cost a fifth less energy and wears
  its bit half as fast, at 1 mB a block.
- Asphalt, a road block of tar, gravel and sand that you walk on a fifth faster.
- The polymerizer turns gas into plastic pellets: 250 mB of gas by liquid tube, a use of a
  catalyst (64 uses, of gold, redstone and clay) and 20,000 energy make 4 pellets.
- The molding press presses pellets in steel molds into plastic sheets, gears and pipes, and a
  sheet onto a steel ingot as a composite panel, 400 energy a press; a mold lasts 64 presses.
  The parts are tagged by their form for other mods.
- The sealed liquid tube, sleeved in plastic: a pump lifts liquid 32 blocks into it, twice what
  it lifts into a liquid tube.
- The pressure tube, a plastic pipe that carries steam twice as fast as a liquid tube.
- The Bit Magazine: placed against a quarry, it swaps a fresh bit that quarry takes into its bit
  slot when the bit is down to its last use, and keeps the worn one. The quarry never uses that
  last use, so the worn bit waits fitted while the magazine's worn-bit slots are full.
- The Gantry Excavator takes a diamond-tipped drill bit, which digs twice as hard.

### Changed

- The Gantry Excavator is made of composite panels and plastic gears around a Steel Excavator,
  and builds its frame of composite panels on steel, a panel every four blocks, from new frame
  slots; the block that took a panel gives it back when broken. Its frames already standing in a
  world keep their old lattice and are never charged for, and a Gantry broken and put back beside
  one takes up its area again.
- The excavator brings stone up as Rubble and metal ore as Poor Ore Rubble, two for each ore
  block; coal, diamond and redstone ore bring up twice their drops. The leveler and the pump
  keep the drops as they are.
- The excavator is now called the Gantry Excavator. Excavators already built work as before.
- Spring engines no longer run from redstone alone: they run on their wound spring, and a
  redstone signal now holds it like a brake. Their recipe takes an iron ingot instead of a
  piston.
- The steel drill bit also fits the Steel Excavator.
- Liquid tubes now join quarries, which take lubricant.
- Ember engines no longer burn lava: their fuel slot refuses lava buckets. A lava bucket already
  in one stays in its slot unburnt until taken out. Lava burns in the firebox instead.

## 0.4.1 - 2026-11-15

### Added

- World settings for how deep excavators dig, `excavator_unlimited_depth` (on) and
  `excavator_lowest_y` (-64), and how far apart survey markers may stand, `marker_max_area`: 32,
  64 (the default), 128 or 256 blocks.
- A running oil engine hazes the air round it, warm exhaust that fades the distance, and muffles
  the world's sounds close by, all but its own low beat. The haze clears a few seconds after it
  stops.

### Changed

- Builds against Weft v26.11.5.
- Excavators dig on below y = -64 until a layer never loads, unless the world limits their
  depth. Past 64 blocks below the excavator, a block's energy cost doubles every 160 blocks: 4
  times as much at 384 below, 64 times at 1,024. It puts as much more into a block each tick, so
  with enough engines a block takes as long at any depth.
- An excavator's standby grows with the length of its area's sides, not with its area: 33 by 33
  needs 78 energy a tick, not 282, and 255 by 255 needs 522. 9 by 9 still needs 30.
- An excavator stores 20 ticks of its standby and of what it puts into a block, at least 20,000
  energy, so deep layers can run.
- Oil fields are rare: one in 32 squares of 96 × 96 blocks holds one, not every other, and each
  has its seep. `/structure locate cogwork:oil_field` says where the nearest is.
- A field lies 45 to 85 blocks under its seep, and all fields lie elsewhere than in 0.4.0. In a
  world made with 0.4.0 the old fields and seeps are gone, except in cubes somebody changed.
- A seep lies on the ground as the world generated it, trees, caves and ravines included: on a
  rise or in a dip within three blocks of the terrain's height, never where a cave opened the
  ground or a tree stands. A place with no dry dirt or sand to lie on holds no field, so any
  field may lie a little higher or lower than in earlier builds of this version, and some are
  gone.
- The spring and ember engines' items stand on their base plate with the trunk up, as an engine
  placed on the floor does, in slots, in the hand and on the ground, instead of lying on their
  side.

### Fixed

- New terrain generates as fast with Cogwork as without: oil took a seventh on top of every
  cube's generation.
- No grass stands on a seep's soaked ground any more.

## 0.4.0 - 2026-11-12

### Added

- Oil: fields of it deep under the overworld, each with a seep of oil in oil-soaked dirt or sand
  on the surface above, and the Oil Bucket.
- Pump: bores down with engine power to a liquid and pumps it up a bucket at a time.
- Liquid Tube: carries any liquid of any mod between pumps and tanks, and shows how full it is.
  Liquid runs by itself from where it stands higher to where it stands lower; uphill it needs a
  pump.
- Timber Liquid Tube: a liquid tube with a small pump in it. Driven by an engine, it pulls liquid
  out of the tanks it touches and pushes it up to 16 blocks uphill, its paddle wheel turning in
  liquid: while it pumps it shows itself full.
- Oil Engine: a 3 × 3 × 3 cube of casing round a combustion chamber, with a controller at its
  front and intakes and drives anywhere on its shell but the front and the top. Built, it burns
  oil with water for 600 energy a tick, about 4 hours on a medium oil field; its casing joins
  into one worn, riveted panel a face with its intakes and drives on it, and two pistons turn a
  crankshaft and flywheel on top, each stroke a heavy boom, letting off vapor.
- Liquid Tank: holds 16 buckets; tanks next to each other make one tank of up to 3 × 3 × 6.
  Buckets pour into it and fill from it.

### Changed

- Built for Weft 26.11.4.
- The mod's description opens with a shorter summary, which the Weft mod index also shows.
- Every shape is a Blockbench model under `assets/models/`: the excavator's rig, the engines
  and their pistons, survey markers, tubes and torque shafts.
- An engine's piston makes its stroke as an animation of its model
  (`assets/animations/piston.animation.json`), at the pace it had.

### Fixed

- An engine's piston no longer flickers over the end of its trunk at the top of its stroke:
  the trunk's end stays in view, flush with the plate.
- On Weft 26.11.4 the piston of an engine whose trunk points east or west stands square to its
  trunk again, not turned 50 degrees short of it.

## 0.3.0 - 2026-11-08

### Added

- Cuboid Loader: keeps its own cube (16 × 16 × 16 blocks) loaded with nobody near.
- Ultra Cuboid Loader: keeps the 7 × 7 cubes around it loaded, at its height.
- The loaders' screen switches between keeping machines running and keeping everything
  running, and shows how many cubes Cogwork keeps loaded of the world's limit.
- Excavators (while mining) and levelers keep their own cubes and the layer they work on
  loaded.
- Cuboid Lens: held in either hand, it outlines every cube Cogwork keeps loaded.

### Changed

- A shorter description on the Mods screen.
- Built for Weft 26.11.3.
- Machines, engines and loaders break quickest with an axe, and still drop whatever breaks them.

### Fixed

- The tuning wrench turns an ember engine while sneaking; before, the engine only opened its
  screen.
- An engine turned to face west no longer stands its piston askew.
- An excavator's gantry beams no longer flicker where their pieces met.
- The lowest piece of a drill's shaft is no longer drawn black.

## 0.2.2 - 2026-11-05

### Changed

- Built for Weft 26.11.2.
- Item tubes leave liquid slot groups alone.

## 0.2.1 - 2026-11-04

### Added

- Sound effects for the engines, tubes, excavator, assembly bench, leveler, tuning wrench,
  survey markers and setting ghost items.

## 0.2.0 - 2026-11-04

### Added

- The excavator says "Jammed" and retries every 5 seconds when it may not break what stands in
  its frame's way, instead of paying for that block every tick.

### Changed

- The assembly bench waits while its output is full instead of dropping items into the world.
- Bench patterns and sorting tube samples are saved by item name, so they survive adding or
  removing mods. Those saved by older versions start empty.

### Fixed

- The assembly bench no longer loses an item's data when it adds it to a cell holding the same
  item with other data.
- Ghost items can be set from as far away as a machine's screen stays open.
- Tubes no longer mistake another world's entities for their item pictures.

## 0.1.2 - 2026-11-02

### Added

- Documentation of every block and item, in `docs/`.

## 0.1.1 - 2026-11-02

### Changed

- Built for Weft 26.11.1.

## 0.1.0 - 2026-11-02

First release.

- Item tubes: stone, timber, gilded, sorting, directing and void, with every item drawn as it
  travels. A line that jams clears itself.
- Sorting tube with a color for each side and ghost samples.
- Spring and ember engines, with pistons in step, and torque shafts that join other mods'
  cables.
- Excavator: builds a gantry frame and mines its area layer by layer down to y = -64. Standby
  power grows with the area; it takes up a standing frame's area.
- Assembly bench: crafts the recipe of its ghost pattern with engine power.
- Leveler: makes an area flat at its height.
- Survey markers, the tuning wrench and five tiers of cogs.
