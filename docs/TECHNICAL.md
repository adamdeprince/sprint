# Silkie Sprint: technical notes

How the game is put together: the simulation, the children's safe landings, the
autopilot, the renderer, and how each kitty and Silkie protocol is used.

The terminal layer (capability discovery, input decoding, kitty graphics
and OSC 777 encoding, terminal lifecycle), the frame pacer, the text
overlays, the vector scene model and the hen, chick, fox and hawk art come
from [Silkie Scramble](../../silkie-scramble/docs/TECHNICAL.md) and work as
described there. This document covers what is different.

## Map of the code

| module | what it does |
|---|---|
| `world` | levels (lanes read from text files) and `traffic`: where every mover is at any moment |
| `sim` | the deterministic simulation: the hen and her flight, hazards, nests, the fox, the hawk, the children (Loki and Skaði), bugs, treats and stray chicks, scoring, recordings, the autopilot |
| `art` | procedural painting of every sprite sheet: the hen, chicks, fox and hawk, Loki and Skaði, vehicles, floats, flying things, treats and effects |
| `gfx` | the vector scene model, the field's scenery and decals, lettering |
| `render` | layout, the diffing renderer, uploads, text, and `snapshot` (a picture of a frame without a terminal) |
| `term` | capability discovery, input decoding, kitty graphics and OSC 777 encoding, terminal lifecycle |
| `app` | the main loop, screens, input, presentation (`show`), pacing, diagnostics |

## Traffic is a function of time

Every lane's movers go round a loop longer than the field is wide. A
mover's left end at time `t` is

```
wrap(start + speed × t, loop) − (loop − width) / 2
```

so the loop's hidden part is split between both sides. Diving turtles,
trains and crossing lights are likewise functions of `t`
(`world/traffic.rs`). Nothing about traffic is stored or stepped. This has
three consequences:

* the renderer draws traffic at the exact moment between two ticks that a
  frame shows, so it moves smoothly at any frame rate, without
  interpolating positions;
* replays and look-ahead copies of the game agree exactly on where
  everything is, in the past and in the future;
* a lane can hold any number of movers of any length, and the loop is made
  long enough to hide the longest one entirely (checked when the level is
  read).

## The simulation

`sim::Game::step` advances the game by one tick of 1/60 s. Given the same
level, seed and inputs, two runs produce the same state; `Recording` stores
the seed and every input, and `silkie-sprint replay` checks a recording ends
in the same state.

A level goes through `Ready` (its card; any key skips it), `Playing`,
`Dying` (with a `Fate`: squashed, drowned, swept away, tangled, snatched,
bitten, out of time), `Nested`, `Cleared` and `GameOver`.

### Hops and flight

Input is presses, not held keys: a held key arrives as repeated presses, so
the game plays the same with or without key-release reports. One press
becomes one hop, queued if a hop is under way.

A hop moves one tile in 0.15 s; a glide in the air takes 0.2 s. On a float
in a river she is carried at the river's speed, hop or no hop (the hop's
ends are carried too). A hop up into the nest row only succeeds within
0.62 tiles of an empty box, and snaps her into it.

Altitude runs from 0 (ground) to 1 (flying) over 0.22 s taking off and
0.3 s coming down. Above 0.45 she clears traffic, water and foxes, and is
in reach of flying things and the hawk. The meter drains in `flight`
seconds; it rests 0.7 s after landing, then refills in `recharge` seconds;
taking off needs a quarter of it. The meter carries over from one crossing
to the next (after a nest or a lost hen alike); only a new level starts
with it full, so flight is a budget for the whole level rather than for
each crossing. `silkie-sprint headless` counts the autopilot's take-offs,
which is a quick check that the levels are not simply flown. Wind pushes her sideways in proportion to
her altitude.

### Hazards

Checked every tick from where she is (`Game::hazard`): vehicles in her row
(half-width 0.28 tiles against each vehicle shrunk by 0.12 tiles at each
end), a passing train, water with no float under her when she is on the
ground and not mid-hop (a hop may cross a gap), a float carrying her past
the field's side, and when she is airborne, flying things in her row.

Sidewalks are checked like roads: their bicycles, prams and wagons knock
her flat just the same.

The fox prowls its verge, freezes for half a second when it sees a hen
standing on it, then runs at 3 tiles/s; it bites within 0.55 tiles.

The hawk (on every level) is drawn by a hen in the air, or by one who has
stood still for 2.5 s on open grass (a verge with no bunting over it). Its
shadow follows a flier for 1.4 s, then it stoops and takes her only if she
is still flying. For a loiterer the shadow settles over the spot where it
saw her, and it takes her only if she is still under it, on the ground and
not under bunting.

## Bugs

`sim/bugs.rs`. A bug turns up every 0.7 to 1.5 s on a random row (not the
nests; over a river it is a dragonfly), away from the hen. Each kind moves
its own way: ladybirds, caterpillars and snails wander slowly along their
row, beetles dash and stop, worms stay put and go back down after a few
seconds, grasshoppers sit and then leap a tile or two (sometimes to the
next row, never into water) and leap away at once from a hen on the ground
within 1.7 tiles, after which they need 1.3 s before they can leap again.
Butterflies flutter anywhere between the start and the nests;
dragonflies dart and hover along their river.

A ground bug is caught by a hen on the ground within 0.6 tiles (not mid-
leap); a flier only by a hen in the air. A ground bug on a road or the
railway under a vehicle or a train is splatted; sidewalk traffic never
touches one. A catch within 3 s of the last extends the combo, multiplying
its points. Bugs leave after 10 to 16 s, so they never pile up.

## The children

`sim/kids.rs`. Loki and Skaði are two `Child`ren driven by the same code,
each with their own speed, care and strength:

| | Loki | Skaði |
|---|---|---|
| chasing speed | 1.9 tiles/s | 2.3 tiles/s |
| carrying speed | 6.5 tiles/s | 5 tiles/s |
| petting | 2.4 s | 2.8 s |
| safe for at least | 2 s | 4 s |
| tug-of-war wins | 2 in 5 | 3 in 5 |

They are always on the field: each starts somewhere well away from the hen
(and from the other) and never leaves. Between chases a child plays about,
heading from one random spot to another (5 to 10 s; the first wait is half
that for Loki, nine tenths for Skaði, so they do not both start at once).
Then they steer for the hen with limited acceleration (she hops at about 6
tiles/s), grab a hen on the ground within 0.55 tiles, pet her (the crossing
clock stops), carry her to where she is put down, and go back to playing.
A chase is given up after 15 s, and called off when the hen nests or is
lost (the countdown between chases is not restarted, or a quick player
would never see them). When one child puts her down, the other, if
chasing, calms down too.

**Tug of war.** If one child is holding the hen (petting her, before
setting off) and the other comes within 2.2 tiles, both take hold: the hen
in the middle, a child 0.62 tiles to each side. After 2 s the winner is
drawn by the game's random generator in proportion to strength, runs off
with her (choosing where to put her down by *their* own care), and the
loser goes off to play elsewhere. The hawk will not tangle with a child: if
its shadow is over her when she is picked up, it gives up.

Where she is put down is chosen by `find_drop`:

1. Candidates: the middle of every column of every verge, road, sidewalk
   and rail row, and every float in the rivers, at the place it will have drifted to
   when the child gets there (found by iterating: the carry time depends on the
   target, the target on the time). The nest row is never a candidate, and
   nothing within 2 tiles of where she was picked up.
2. They are shuffled with the game's random generator (so a replay picks
   the same spot).
3. Each is tested by `safe_drop`: a copy of the game runs forward, with
   the children removed and the hen held (untouchable, clock stopped), for exactly
   the ticks the carry takes; she is put down there; and the copy runs on
   for the child's safe time (2 s for Loki, 4 s for Skaði) with no input
   at all. The first candidate where she is
   still playing at the end wins. Because traffic is a function of time,
   the copy sees exactly the cars, trains, diving turtles and currents she
   will meet, and the fox moves as it will.
4. If nothing were safe (it does not happen on the built-in levels) she is
   put back where she was.

The search is complete: every candidate can be tried, and nothing caps
it. A test is about 300 ticks of a copy and costs well under a
millisecond; the diagnostics panel shows the last search's time. Look-ahead
copies made by the autopilot do not search: they only need to know the hen
will be somewhere, unharmed.

## The autopilot

`sim/autopilot.rs` plays the title-screen demo, `--autoplay` and the
headless tests. It runs copies of the game through sequences of actions
(wait 0.1 s, hop in each direction, take off or land), keeping a beam of the
fourteen most promising positions per step for twelve steps, one position
per place (row, a third of a tile, flying or not), the earliest to get
there. A position is valued by how far up the field the hen is, how near an
empty nest (more so the nearer the top), how much meter she has left and
how long it took, plus a little for points scored (so it will detour for a
bug). Copies where she comes to grief are dropped, and a plan
that ends where she would not last another fifth of a second is
discounted. It takes the first action of the best plan and thinks again.

The tests use it to show every level can be crossed, and that every level
can be crossed on foot (with flight forbidden), so flight is a choice, not a
requirement.

## Rendering

As in Scramble, the renderer keeps a model of what the terminal holds and
writes only the difference. What is new:

* **Clipping.** A sprite partly off the field is placed with a narrower
  source rectangle, starting at the field's edge; one entirely off it is not
  placed. Traffic slides in from the sides instead of popping into view or
  spilling into the margins.
* **Segments.** Logs, limousines, turtle rafts, lily pads and trains are
  drawn one segment (or carriage) per sprite, so any length works with a
  few frames of art. Segments overlap by a couple of art units so no gap
  shows however pixels round.
* **Layers.** Shadows and treats on the ground; then bodies sorted by the
  height of their feet (floats just under whoever stands on them); the
  canopy (bunting); then whatever flies (the hen in the air, kites,
  balloons, drones, the children) sorted the same way; then effects. The
  hen in a child's arms and the arms themselves are drawn over everything,
  in that order, and in a tug of war the hen is drawn between the two
  children at the height of their hands.
* **Decals.** Nest boxes (empty, an egg, a hatched shell) and crossing lamps
  (dark, or either lamp lit) are a few elements with stable ids. On Silkie a
  change is a patch of the ground's vector image; on kitty a different
  source rectangle of a small atlas of every decal in every state. The
  diagnostics panel counts any edit that would make Silkie redraw the whole
  image (there are none).

`render/snapshot.rs` composites the same sheets and scenery into one
picture in the same stacking order, for `silkie-sprint snapshot`, the
pictures in the README, and art review.

## The protocols

Exactly as in Scramble, except:

* **Tablet pointer**: a pen or touchscreen finger touching down is a tap
  (fly or land on the hen, hop towards it elsewhere). There are no strokes,
  so terminal-side ink is not used.
* **Image pointer**: a little feather.
* **Haptics**: `alignment` on take-off, cuddles and bugs, `complete` on nesting,
  `click` on a bump or a refused take-off, `generic` on a mishap and when
  a child puts the hen down, and on a tug of war; at most one every 80 ms.

## Testing

`cargo test` runs about ninety tests, none needing a terminal:

* the level format: every built-in level, mistakes reported with their line,
  loops long enough to hide their movers, sized movers;
* traffic: wrapping in both directions, diving turtles' cycle, trains
  warning, passing and going;
* the simulation, headless with fixed seeds: hops and row points, queued
  hops, staying on the field, cars (and flying over them), the meter
  emptying and refilling, rivers drowning and carrying and sweeping off the
  edge, nests filling and clearing a level, the clock and lives, kites only
  catching a flying hen, bunting, the hawk taking a hen who stays up and
  missing one who lands, the fox, Loki on twelve seeds (catching, petting,
  stopping the clock, putting her somewhere else, and her surviving the
  idle seconds there), each child's own safe time (two seconds for Loki,
  four for Skaði), tugs of war (both children win some; the winner puts her
  down safe for their own time), neither child reaching a flying hen, the hawk taking a
  loiterer but missing one who hops away or stands under bunting, bugs
  splatted on roads but not sidewalks, combos, grasshoppers leaping away,
  butterflies only caught in the air, replays through JSON,
  the autopilot reaching a nest on every level, and on foot;
* scenery: every level builds, and every decal state uses the same element
  ids (so changes are patches);
* rendering, by decoding the bytes it writes: uploads only on the first
  frame and none when nothing changed, one placement for a moving sprite,
  clipping at the field's edge, decals as vector patches on Silkie and atlas
  placements on kitty, vector scenery surviving a resize, text diffing,
  teardown, evicted images re-sent, every sheet painting;
* the terminal layer's tests from Scramble.

The game was also run in kitty 0.48 and Silkie on macOS with `--autoplay`,
through the title, several levels and level changes, with the terminal's
replies logged: no graphics errors; on Silkie, vector scenery with no full
redraws, presentation-report pacing, low latency, haptics and the tablet
pointer all on, at about 4 KB a frame. The window was not captured (no
screen-recording permission was available), so what it looks like is
checked with `snapshot`, which composites the same sheets and scenery.

Not tried: a real pen or touchscreen (taps are exercised through the
protocol code), how the haptics feel, HDR on a display with headroom, and
play over `netsim`.
