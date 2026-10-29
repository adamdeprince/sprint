# Asset provenance

Everything the game shows is listed here.

## Sprites

Original artwork, painted at run time by the code in `src/art/` (shapes,
gradients and strokes drawn with tiny-skia, from fixed seeds) at exactly the
size a tile has on screen. `cargo run --release --example preview_sheet`
paints any sheet to a PNG.

* `hen`, `chick`, `fox`, `hawk`: from Silkie Scramble, by the same author.
  The hen gains flight poses (spread wings beating, in four directions),
  her shadow, a pancake (run over), a drenched look (fished out of the
  river) and a smaller pose for being cuddled.
* `loki`: Loki on his space hopper, bouncing both ways and pulling in a tug
  of war; Loki cuddling, and his arms alone (drawn over the hen).
* `skadi`: Skaði running both ways, pulling in a tug of war, holding the
  hen, and her arms alone.
* `cars`: hatchback with sheepdog, tractor, ice-cream van, motorised sofa
  with knitting granny, mobility scooter, bathtub racer, ride-on mower,
  double-decker bus of sheep, tandem, camper van, milk float, clown on a
  unicycle, limousine segments, the Custard Express's engine and carriage;
  and for sidewalks a cyclist, a dad with a pram and a girl with a wagon.
* `floats`: log segments, turtles (swimming, sinking, under), rubber duck,
  inflatable flamingo, picnic table, lily pads, swan pedalo, scarecrow's
  raft, canoe of ducklings, rubber ring.
* `sky`: kite, hot-air balloon, parcel drone, geese, blimp, paper aeroplane,
  umbrella, and their shadows.
* `items`: corn and golden feathers; the bugs (snail, ladybird,
  caterpillar, worm, beetle, grasshopper sitting and leaping, butterfly,
  dragonfly).
* `fx`: emote bubbles, feathers, dust, sparkles, stars, hearts, a splash, a
  puff and a (cartoon) bug splat.

The pointer on Silkie is a small feather painted the same way.

The characters, vehicles, field and presentation were designed to be
original. They do not reproduce the characters, layouts, animations or look
of any existing game.

## Scenery

Drawn by `src/gfx/scenery.rs` from each level's lanes: vector shapes for
verges, roads, kerbs and markings, rivers and banks, the railway and its
crossing lamps, sidewalks (slabs, hopscotch, drains), the henhouse and its
nest boxes, gnomes and bunting.
Original.

## Typeface

`assets/fonts/LilitaOne-Regular.ttf`: Lilita One by Juan Montoreano,
licensed under the SIL Open Font License 1.1 (`assets/fonts/OFL.txt`).
Used for the title lettering and score pop-ups. The font file is unmodified.

## Levels

`assets/levels/*.txt`: original, written for this game.
