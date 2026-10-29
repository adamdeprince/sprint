//! Tuning constants. Distances are in tiles, times in seconds, speeds in
//! tiles per second.

/// Simulation rate. Rendering is independent of it.
pub const TICK_HZ: u32 = 60;
pub const DT: f32 = 1.0 / TICK_HZ as f32;

// Hopping.
pub const HOP_TIME: f32 = 0.15;
/// A hop while flying: a fluttering glide, a little slower than a hop.
pub const GLIDE_TIME: f32 = 0.2;
/// Half the hen's width, for what runs her over.
pub const HEN_HALF: f32 = 0.28;
/// Vehicles hurt a little inside their art.
pub const VEHICLE_INSET: f32 = 0.12;
/// How far from a nest box's centre a hop up still finds the way in.
pub const NEST_SNAP: f32 = 0.62;
/// The hen keeps this far inside the field's sides on her own feet.
pub const EDGE_MARGIN: f32 = 0.32;
/// Carried this far past a side, she is swept away.
pub const SWEPT_MARGIN: f32 = 0.05;

// Flight.
/// Altitude (0 on the ground, 1 cruising) from which the hen clears
/// traffic, and is in reach of flying things.
pub const AIR_ALT: f32 = 0.45;
pub const TAKEOFF_TIME: f32 = 0.22;
pub const LAND_TIME: f32 = 0.3;
/// The meter needs at least this much to get off the ground.
pub const MIN_TAKEOFF: f32 = 0.25;
/// After landing, the meter rests this long before it refills.
pub const RECHARGE_DELAY: f32 = 0.7;

// The fox.
pub const FOX_PROWL_SPEED: f32 = 1.1;
pub const FOX_CHASE_SPEED: f32 = 3.0;
pub const FOX_SIGHT: f32 = 5.0;
pub const FOX_ALERT_TIME: f32 = 0.5;
pub const FOX_BITE: f32 = 0.55;
pub const FOX_GIVE_UP: f32 = 1.0;

// The hawk.
pub const HAWK_WARN: f32 = 1.4;
pub const HAWK_STOOP: f32 = 0.35;
pub const HAWK_REACH: f32 = 1.0;
pub const HAWK_AWAY_MIN: f32 = 4.0;
pub const HAWK_AWAY_MAX: f32 = 9.0;
/// Standing about this long on open grass draws the hawk too.
pub const HAWK_LOITER: f32 = 2.5;

// The children (Loki and Skaði; their own speeds and care are in `kids`).
pub const KID_ACCEL: f32 = 6.0;
pub const KID_REACH: f32 = 0.55;
/// A child gives up the chase after this long and plays instead.
pub const KID_PATIENCE: f32 = 15.0;
/// A child plays about between chases for this long.
pub const KID_REST_MIN: f32 = 5.0;
pub const KID_REST_MAX: f32 = 10.0;
/// Where Loki puts the hen down must stay safe this long if she does
/// nothing at all.
pub const LOKI_SAFE_TIME: f32 = 2.0;
/// Skaði is older, and more careful.
pub const SKADI_SAFE_TIME: f32 = 4.0;
/// The other child this close to one holding the hen: tug of war.
pub const TUG_RANGE: f32 = 2.2;
pub const TUG_TIME: f32 = 2.0;
/// How far either side of the hen the children stand to pull.
pub const TUG_GAP: f32 = 0.62;

// Bugs.
pub const BUG_EVERY_MIN: f32 = 0.7;
pub const BUG_EVERY_MAX: f32 = 1.5;
/// How close the hen's beak (or, in the air, her feet) must come.
pub const BUG_REACH: f32 = 0.6;
/// Seconds a grasshopper is in the air when it leaps.
pub const GRASSHOPPER_LEAP: f32 = 0.35;
/// A grasshopper leaps away from a hen this close.
pub const GRASSHOPPER_WARY: f32 = 1.7;
/// Bugs caught within this many seconds of each other make a combo.
pub const COMBO_WINDOW: f32 = 3.0;

// Treats and stray chicks.
pub const ITEM_LIFE: f32 = 9.0;
pub const ITEM_EVERY_MIN: f32 = 6.0;
pub const ITEM_EVERY_MAX: f32 = 11.0;
pub const PICKUP_DIST: f32 = 0.6;
pub const CHICK_EVERY_MIN: f32 = 10.0;
pub const CHICK_EVERY_MAX: f32 = 18.0;

// Phases.
pub const READY_TIME: f32 = 2.2;
pub const READY_TIME_FIRST: f32 = 6.0;
pub const DEATH_TIME: f32 = 1.8;
pub const NESTED_TIME: f32 = 1.2;
pub const CLEARED_TIME: f32 = 4.5;

// Scoring.
pub const POINTS_ROW: u32 = 10;
pub const POINTS_NEST: u32 = 50;
/// For every second left on the clock when she reaches a nest.
pub const POINTS_PER_SECOND: u32 = 20;
pub const POINTS_CHICK: u32 = 200;
pub const POINTS_ALL_NESTS: u32 = 1000;
pub const POINTS_CORN: u32 = 50;
pub const POINTS_FEATHER: u32 = 50;
pub const START_LIVES: u32 = 4;
pub const EXTRA_LIFE_EVERY: u32 = 10_000;
