# Original Infon Architecture

Technical documentation of the original C/Lua implementation.

## Source Layout (orig_game/)

```
orig_game/
  infond.c          # Server entry point
  server.c/h        # Networking (libevent-based TCP)
  game.c/h          # Main game loop
  player.c/h        # Player management, Lua VM per player
  creature.c/h      # Creature logic, state machine, combat
  world.c/h         # Map, tiles, pathfinding (A*)
  common_creature.h # Shared creature constants & type definitions
  common_player.h   # Shared player constants
  common_world.h    # Shared world constants
  creature_config.gperf  # Creature stats (compiled via gperf)
  packet.h          # Network packet format
  config.lua        # Server configuration
  api/
    oo.lua          # Object-oriented high-level API (~523 lines)
    state.lua       # State machine high-level API (~518 lines)
  level/            # Map definitions (.lua)
  rules/            # Game rule handlers (.lua)
    default.lua     # Default scoring/win rules
  contrib/bots/     # Example bot scripts
  lua-5.1.2/        # Embedded Lua interpreter
```

## Server Architecture

### Game Loop (game.c)

Each tick (~100ms):
1. Call `onRound` rule handler
2. World tick (food regeneration, map updates)
3. Player round management
4. Execute `player_think()` for each player (runs their Lua)
5. Player synchronization (send updates)
6. Creature movement and state processing
7. Advance game time by delta
8. Handle network I/O

### Player Isolation (player.c)

- Each player gets a separate `lua_State` with custom memory allocator
- CPU limits: ~100K Lua VM cycles per think call
- Memory limits: ~50MB per player
- High-level API (oo.lua or state.lua) loaded into each player VM
- Creature callbacks dispatched through the player's VM

### Networking (server.c)

- libevent for async I/O
- TCP on port 1234 (configurable)
- Max 1024 concurrent clients
- Multiple clients can control same player
- Optional zlib compression

### Packet Protocol (packet.h)

```
+--------+--------+------------------+
| len(1) | type(1)| payload(0-255)   |
+--------+--------+------------------+
```

Key packet types:
- `PACKET_CREATURE_UPDATE (3)` - Position, state, health, food
- `PACKET_WORLD_UPDATE (1)` - Tile food changes
- `PACKET_PLAYER_UPDATE (0)` - Scores
- `PACKET_ROUND (9)` - Round start, delta time
- `PACKET_GAME_INFO (8)` - Game time
- `PACKET_KOTH_UPDATE (5)` - King position

### Creature System (creature.c)

Creature struct (key fields):
```c
struct creature {
    int x, y;                // Position in world units
    creature_type type;      // SMALL=0, BIG=1, FLYER=2
    int food, health;        // Resources
    player_t *player;        // Owner
    creature_state state;    // Current action
    int target_id;           // Attack/feed target
    pathnode_t *path;        // Movement waypoints
    int convert_food;        // Conversion progress
    int spawn_food;          // Spawning progress
    char message[9];         // Display message
};
```

State processing per tick:
- **WALK**: Follow A* path, move by speed * delta
- **EAT**: Transfer tile food to creature food (rate depends on type)
- **HEAL**: Convert creature food to health (rate depends on type)
- **ATTACK**: Deal damage to target if in range
- **CONVERT**: Accumulate food toward conversion cost
- **SPAWN**: Accumulate food toward spawn cost, then create new creature
- **FEED**: Transfer food to target creature if in range
- **Aging**: All creatures lose health every tick

### World System (world.c)

- Tile grid loaded from Lua level files
- Each tile: walkable type, food amount, graphics type
- A* pathfinding for ground creatures
- Flyers bypass pathfinding (direct movement)
- Food capped at MAX_TILE_FOOD per tile

### Game Rules (rules/)

Lua scripts that handle game events:
- `onNewGame()`, `onRound()`, `onGameEnded()`
- `onCreatureSpawned(id, parent_id)`
- `onCreatureKilled(victim_id, killer_id)`
- `onPlayerScoreChange(player_id, new_score, reason)`
- Manage map rotation, win conditions, scoring

## Key Design Decisions

1. **Per-player Lua VM** - Security isolation prevents players from accessing each other's state
2. **Coroutine-based creature control** - Each creature runs as a Lua coroutine, yielded each tick
3. **Tick-based simulation** - Deterministic 100ms ticks ensure consistent gameplay
4. **Event-driven networking** - libevent scales to many concurrent clients
5. **Delta compression** - Only changed state sent over the wire
6. **Two API levels** - Low-level C functions + high-level Lua wrappers give flexibility
