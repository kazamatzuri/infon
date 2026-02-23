// LLM-friendly documentation endpoint content.

pub const LLMS_TXT: &str = r#"# Infon Battle Arena API
> A competitive bot programming platform where players write Lua bots to control creature swarms.

## API Base URL
/api/

## Authentication
Bearer token (JWT from /api/auth/login or API key from /api/api-keys)

## Key Endpoints
- POST /api/auth/register - Create account
- POST /api/auth/login - Get JWT token
- GET /api/auth/me - Get current user info
- GET/POST /api/bots - List/create bots
- GET/PUT/DELETE /api/bots/{id} - Get/update/delete bot
- GET/POST /api/bots/{id}/versions - List/create bot versions
- PUT /api/bots/{id}/active-version - Set active version
- GET /api/bots/{id}/stats - Get bot version stats
- GET /api/matches/{id} - Get match details
- GET/POST /api/tournaments - List/create tournaments
- POST /api/tournaments/{id}/run - Run a tournament
- GET /api/leaderboards/1v1 - View 1v1 rankings
- GET /api/leaderboards/ffa - View FFA rankings
- GET /api/leaderboards/2v2 - View 2v2 rankings
- GET/POST /api/teams - List/create teams
- GET/POST /api/api-keys - List/create API keys
- DELETE /api/api-keys/{id} - Revoke API key
- GET /api/docs/lua-api - Lua API reference (Markdown)
- GET /api/maps - List available maps
- POST /api/game/start - Start a game
- GET /api/game/status - Check game status
- POST /api/feedback - Submit feedback

## Bot Programming
Bots are written in Lua 5.1. Two API styles are supported:
- Object-oriented (oo): Define `Creature:main()` as a coroutine
- State machine (state): Define `bot()` with state functions

See /api/docs/lua-api for the full API reference.

## WebSocket
- /ws/game - Live game state stream (JSON frames)
"#;

pub const LLMS_FULL_TXT: &str = r#"# Infon Battle Arena - Complete Documentation

> Infon Battle Arena is a competitive bot programming platform where players write Lua scripts
> to control swarms of creatures competing for food and territory on a 2D tile-based map.
> Originally created by Florian Wesch, this web version brings the classic gameplay to the browser.

---

## Platform Overview

Infon Battle Arena is an open-source (GPL) multiplayer programming game. Players write Lua 5.1
scripts that control autonomous creatures in a shared game world. The game runs in 100ms ticks,
and each tick every creature's Lua coroutine is resumed to make decisions.

### Key Concepts
- **Creatures**: Autonomous units controlled by your Lua code
- **Food**: Resource on map tiles that creatures eat to survive and grow
- **King of the Hill**: A special tile; idle creatures on it score points for their player
- **Types**: 3 creature types with different stats (Small, Big, Flyer)
- **Spawning**: Big creatures (Type 1) can spawn new Small creatures (Type 0)
- **Converting**: Creatures can change type by spending food

---

## REST API Reference

### Authentication

All authenticated endpoints require a Bearer token in the Authorization header.

**Register:**
```
POST /api/auth/register
Content-Type: application/json
{"username": "player1", "email": "player1@example.com", "password": "secret123"}
```

**Login:**
```
POST /api/auth/login
Content-Type: application/json
{"username": "player1", "password": "secret123"}
Response: {"token": "jwt...", "user": {...}}
```

**Current User:**
```
GET /api/auth/me
Authorization: Bearer <token>
```

### Bots

**List Bots:**
```
GET /api/bots
GET /api/bots?all=true  (list all public bots)
Authorization: Bearer <token>
```

**Create Bot:**
```
POST /api/bots
Authorization: Bearer <token>
Content-Type: application/json
{"name": "MyBot", "description": "A simple bot"}
```

**Get/Update/Delete Bot:**
```
GET /api/bots/{id}
PUT /api/bots/{id}  {"name": "NewName", "description": "Updated"}
DELETE /api/bots/{id}
```

### Bot Versions

**List Versions:**
```
GET /api/bots/{id}/versions
```

**Create Version:**
```
POST /api/bots/{id}/versions
Content-Type: application/json
{"code": "function Creature:main()\n  self:eat()\nend"}
```

**Set Active Version:**
```
PUT /api/bots/{id}/active-version
Content-Type: application/json
{"version_id": 3}
```

**Get Bot Stats:**
```
GET /api/bots/{id}/stats
Response: [{version_id, elo_1v1, games_played, wins, losses, ...}]
```

### Matches

**List Recent Matches:**
```
GET /api/matches?limit=20
```

**Get Match Detail:**
```
GET /api/matches/{id}
Response: {"match": {...}, "participants": [...]}
```

**Get Match Replay:**
```
GET /api/matches/{id}/replay
Response: {"match_id", "tick_count", "messages": [...]}
```

**Create Challenge:**
```
POST /api/matches/challenge
Content-Type: application/json
{
  "bot_version_id": 1,
  "opponent_bot_version_id": 2,
  "format": "1v1",
  "headless": true,
  "map": "default"
}
```

### Game Control (Live Games)

**Start Game:**
```
POST /api/game/start
Content-Type: application/json
{
  "players": [{"bot_version_id": 1, "name": "Bot A"}, {"bot_version_id": 2}],
  "map": "random"
}
```

**Game Status:**
```
GET /api/game/status
Response: {"running": true}
```

**Stop Game:**
```
POST /api/game/stop
```

### Tournaments

**List/Create Tournaments:**
```
GET /api/tournaments
POST /api/tournaments  {"name": "Weekly", "map": "default"}
```

**Get Tournament:**
```
GET /api/tournaments/{id}
```

**Update Tournament:**
```
PUT /api/tournaments/{id}
{"format": "round_robin", "config": "{}"}
Formats: round_robin, single_elimination, swiss_N
```

**Tournament Entries:**
```
GET /api/tournaments/{id}/entries
POST /api/tournaments/{id}/entries  {"bot_version_id": 1}
DELETE /api/tournaments/{id}/entries/{entry_id}
```

**Run Tournament:**
```
POST /api/tournaments/{id}/run
```

**Standings & Results:**
```
GET /api/tournaments/{id}/standings
GET /api/tournaments/{id}/results
```

### Leaderboards

```
GET /api/leaderboards/1v1?limit=50&offset=0
GET /api/leaderboards/ffa?limit=50&offset=0
GET /api/leaderboards/2v2?limit=50&offset=0
```

### Teams (2v2)

```
GET /api/teams
POST /api/teams  {"name": "Dream Team"}
GET /api/teams/{id}
PUT /api/teams/{id}  {"name": "New Name"}
DELETE /api/teams/{id}
GET /api/teams/{id}/versions
POST /api/teams/{id}/versions  {"bot_version_a": 1, "bot_version_b": 2}
```

### API Keys

```
GET /api/api-keys
POST /api/api-keys  {"name": "CI Key", "scopes": "bots:read,matches:read"}
DELETE /api/api-keys/{id}
```

### Maps

```
GET /api/maps
Response: [{"name": "random", "width": 30, "height": 30, "description": "..."}]
```

### Lua Validation

```
POST /api/validate-lua
Content-Type: application/json
{"code": "function Creature:main() end"}
Response: {"valid": true} or {"valid": false, "error": "..."}
```

### Feedback

```
POST /api/feedback
Content-Type: application/json
{"category": "bug", "description": "Something went wrong"}
Categories: bug, feature, general
```

### Documentation

```
GET /api/docs/lua-api  - Lua API reference (Markdown)
GET /llms.txt          - LLM-friendly summary
GET /llms-full.txt     - Complete LLM documentation (this file)
```

### WebSocket

```
WS /ws/game  - Live game state stream
```

Messages:
- `world`: Map dimensions, tiles, KotH position
- `snapshot`: Creature positions, player scores (sent each tick)
- `game_end`: Final scores, winner, match ID
- `player_load_error`: Lua loading errors

---

## Game Mechanics

### Game World
- 2D tile-based grid (each tile 256x256 units)
- X increases rightward, Y increases downward
- Tiles: TILE_SOLID (0, walls) or TILE_PLAIN (1, walkable)
- Food spawns on tiles and can be consumed by creatures
- world_size() returns playable boundaries as x1, y1, x2, y2

### Creature Types

**Type 0 - Small (Balanced)**
- Speed: 200-300 units/sec
- Max Food: 10,000 | HP: 10,000
- Attack: 768 range, vs Flyers only (1000 dmg)
- Converts to Type 1 (8000 food) or Type 2 (5000 food)

**Type 1 - Big (Tank)**
- Speed: 400 units/sec
- Max Food: 20,000 | HP: 20,000
- Attack: 512 range, all types (1500 dmg)
- Spawns Type 0 (costs 5000 food + 20% health)
- Converts to Type 0 (8000 food)

**Type 2 - Flyer (Scout)**
- Speed: 800 units/sec (fastest)
- Max Food: 5,000 | HP: 5,000
- Cannot attack
- Can fly over water and walls
- Converts to Type 0 (5000 food)

### Combat Damage Table

| Attacker | vs Small | vs Big | vs Flyer |
|----------|----------|--------|----------|
| Small    | 0        | 0      | 1000     |
| Big      | 1500     | 1500   | 1500     |
| Flyer    | -        | -      | -        |

### Creature States
- CREATURE_IDLE (0): Doing nothing; required for King of the Hill
- CREATURE_WALK (1): Moving to destination
- CREATURE_HEAL (2): Using food to restore health
- CREATURE_EAT (3): Consuming food from tile
- CREATURE_ATTACK (4): Attacking target
- CREATURE_CONVERT (5): Changing type
- CREATURE_SPAWN (6): Type 1 producing Type 0
- CREATURE_FEED (7): Transferring food (max 256 units distance)

### King of the Hill
- Special tile on the map at a fixed position
- Creature in IDLE state on tile becomes king
- King's player scores points each tick
- king_player() returns current king's player ID

### Scoring
- Points from holding King of the Hill
- suicide() costs 40 points
- Players may be kicked if score drops too low

---

## Lua API Reference

### Constants
```lua
CREATURE_IDLE=0  CREATURE_WALK=1  CREATURE_HEAL=2  CREATURE_EAT=3
CREATURE_ATTACK=4  CREATURE_CONVERT=5  CREATURE_SPAWN=6  CREATURE_FEED=7
TILE_SOLID=0  TILE_PLAIN=1
CREATURE_SPAWNED=0  CREATURE_KILLED=1  CREATURE_ATTACKED=2  PLAYER_CREATED=3
```

### Low-Level API

**Creature Actions:**
- set_path(id, x, y) -> bool: Set movement destination
- set_state(id, state) -> bool: Set creature state
- get_state(id) -> state: Get current state
- set_target(id, target_id) -> bool: Set attack/feed target
- set_convert(id, type) -> bool: Set conversion type (0,1,2)
- suicide(id): Kill creature, drop 1/3 food
- set_message(id, msg): Display message (max 8 chars)

**Creature Queries:**
- get_pos(id) -> x, y
- get_type(id) -> type
- get_food(id) -> food (own only)
- get_health(id) -> 0-100
- get_speed(id) -> speed (own only)
- get_tile_food(id) -> food (own only)
- get_tile_type(id) -> type
- get_max_food(id) -> food (own only)
- get_distance(id, target_id) -> dist
- get_nearest_enemy(id) -> id, x, y, playernum, dist (or nil)
- creature_exists(id) -> bool
- creature_player(id) -> player_no

**World Functions:**
- world_size() -> x1, y1, x2, y2
- game_time() -> ms
- get_koth_pos() -> x, y
- player_exists(id) -> bool
- king_player() -> player_id
- player_score(id) -> score
- get_cpu_usage() -> 0-100
- print(msg)

### Object-Oriented API (oo.lua)

Entry point: `function Creature:main() ... end`

**Blocking methods:** moveto(x,y), heal(), eat(), feed(target), attack(target), convert(type), spawn(), suicide()

**Non-blocking:** begin_idling(), begin_walk_path(), begin_healing(), begin_eating(), begin_attacking(), begin_converting(), begin_spawning(), begin_feeding()

**Properties:** self.id, self:pos(), self:speed(), self:health(), self:food(), self:max_food(), self:tile_food(), self:tile_type(), self:type(), self:distance(other), self:nearest_enemy()

**Utility:** set_path(x,y), set_target(c), set_conversion(t), screen_message(msg), sleep(ms), wait_for_next_round(), restart()

**Callbacks:** Creature:onSpawned(parent_id), Creature:onKilled(killer_id), Creature:onAttacked(attacker_id)

### State Machine API (state.lua)

Entry point: `function bot() ... end`

**State transitions:** and_start_state(name,...), and_be_in_state(name,...), and_keep_state, and_restart_state, in_state(name)

**Actions:** move_to(x,y), move_path(x,y), random_move(), random_path(), heal(), eat(), feed(target), attack(target), convert(type), spawn(), sleep(ms)

**Properties:** food(), health(), max_food(), tile_food(), tile_type(), type(), speed(), pos(), can_eat()

**Events:** onSpawned(parent_id), onKilled(killer_id), onIdle(), onTileFood(), onLowHealth(), onTick()

### Example Bot (OO)
```lua
function Creature:main()
    while true do
        if self:health() < 50 then
            self:heal()
        elseif self:tile_food() > 0 and self:food() < self:max_food() then
            self:eat()
        elseif self:type() == 0 and self:food() > 8000 then
            self:convert(1)
        elseif self:type() == 1 and self:food() > 8000 and self:health() > 60 then
            self:spawn()
        else
            local x1, y1, x2, y2 = world_size()
            self:moveto(math.random(x1, x2), math.random(y1, y2))
        end
    end
end
```
"#;
