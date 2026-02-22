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

## Bot Programming
Bots are written in Lua 5.1. Two API styles are supported:
- Object-oriented (oo): Define `Creature:main()` as a coroutine
- State machine (state): Define `bot()` with state functions

See /api/docs/lua-api for the full API reference.

## WebSocket
- /ws/game - Live game state stream (JSON frames)
"#;
