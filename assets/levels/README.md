# Level format

A level is a text file: settings, then `lanes:`, then one line per row of
the field from top to bottom. `#` starts a comment.

```text
name: Country Lane
tagline: Why did the Silkie cross the road?
width: 20          # tiles
time: 50           # seconds for each crossing
nests: 4           # nest boxes, spread evenly along the top
flight: 1.23       # seconds of flight on a full wings meter
recharge: 16       # seconds on the ground for an empty meter to refill
                   # (it carries over from one crossing to the next)
wind: 0            # drift of a flying hen, tiles/s (+ is rightwards)
hawk: no           # a hawk hunts flying and loitering hens
loki: no           # Loki chases the hen for cuddles
skadi: no          # so does his big sister Skaði
chicks: yes        # stray chicks ride the river now and then
lanes:
nests
river  speed=-1.0  log4@0 log3@7 log4@13
verge  items
road   speed=-1.6  hatchback@0 tractor@15   air=0.8 ^kite@3
start
```

The first lane must be `nests` and the last `start`.

## Lane kinds

| kind | what it is |
|---|---|
| `nests` | the henhouse wall with its nest boxes |
| `verge` | safe grass |
| `start` | the verge each crossing starts on |
| `road` | vehicles; they run the hen over (and splat bugs) |
| `sidewalk` | bicycles, prams and wagons; they run the hen over too (but miss bugs) |
| `river` | water; she must ride something across |
| `rail` | a railway; a train crosses every so often |

## Movers

A mover is written `kind@position`: where along the lane's loop its left end
is at time zero, in tiles. Every lane's movers travel at the lane's `speed`
(tiles per second; negative is leftwards) around a loop longer than the
field, so they leave one side and come back on the other. The loop is made
long enough to hide the longest mover completely on both sides and to hold
every starting position; `loop=` asks for a longer one (more space between
returns).

Road: `hatchback` (1.6 tiles), `tractor` (2), `icecream` (2.4), `sofa` (1.9),
`scooter` (1.1), `bathtub` (1.7), `mower` (1.2), `bus` (2.8), `camper` (2.2),
`milkfloat` (1.8), `limoN` (N tiles; default 4), and anything a sidewalk
takes.

Sidewalk: `bicycle` (1.1), `stroller` (1.1), `wagon` (1.3), `tandem` (1.6),
`unicycle` (0.7), `scooter` (1.1).

River: `logN` (N tiles; default 3), `turtlesN` (N turtles), `diversN` (N
turtles that dive now and then), `duck` (2), `flamingo` (1.6), `picnic` (2),
`pedalo` (2), `raft` (2), `canoe` (2.4), `ring` (1), `lilyN` (N pads).

Flying things are written with a `^` and move at the lane's `air` speed
around its own loop (`airloop=`): `^kite` (0.9), `^balloon` (1.5),
`^drone` (0.9), `^geese` (2), `^blimp` (2.8), `^plane` (0.7), `^umbrella`
(0.9). Any lane can have them; they only meet a flying hen.

## Lane options

| option | meaning |
|---|---|
| `speed=F` | speed of the lane's movers, tiles per second |
| `loop=F` | length of the movers' loop (at least what they need) |
| `air=F`, `airloop=F` | the same for flying things |
| `dive=F` | seconds in a diving turtle's cycle (default 7) |
| `bunting` | bunting over the row: no flying into it, no taking off under it; cover from the hawk |
| `items` | treats (corn, golden feathers) turn up here |
| `fox` | a fox patrols this verge |

A `rail` lane takes `speed=F` (default 14), `train=N` carriages (default 3),
`every=S` seconds between trains (default 14; raised if a train would not
have finished crossing), `warn=S` seconds of flashing lights before it comes
(default 2.5) and `offset=S` to shift its timetable.

Bugs need no setting: they turn up on every row but the nests (dragonflies
over rivers).

Mistakes are reported with their line number. A river with nothing to ride
on is refused, as is traffic without a speed.
