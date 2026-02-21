use mlua::Lua;

use super::config::LUA_MAX_INSTRUCTIONS;
use super::lua_api;

/// Represents a player controlling a swarm of creatures.
pub struct Player {
    pub id: u32,
    pub name: String,
    pub score: i32,
    pub color: u8,
    pub num_creatures: i32,
    pub lua: Lua,
    pub api_type: String,
    pub output: Vec<String>,
}

impl Player {
    /// Create a new player with a fresh Lua VM.
    /// Registers all API functions and constants, loads the high-level API (oo or state).
    /// Bot code is NOT loaded here — call `load_code()` after setting game state
    /// so that top-level bot code can call API functions like `world_size()`.
    pub fn new(id: u32, name: &str, api_type: &str) -> Result<Self, String> {
        // SAFETY: We need the debug library for debug.sethook to set instruction
        // limits on coroutine threads. The debug global is removed in the bootstrap
        // after saving a reference to debug.sethook, so user code cannot access it.
        let lua = unsafe { Lua::unsafe_new() };

        // Register constants and API functions
        lua_api::register_constants(&lua, id)
            .map_err(|e| format!("Failed to register constants: {e}"))?;
        lua_api::register_functions(&lua, id)
            .map_err(|e| format!("Failed to register API functions: {e}"))?;

        // Provide _TRACEBACK as a simple passthrough (debug.traceback removed in sandbox)
        lua.load(
            r#"
            function _TRACEBACK(...)
                return tostring((...))
            end
            function epcall(handler, func, ...)
                return xpcall(func, handler, ...)
            end
            "#,
        )
        .exec()
        .map_err(|e| format!("Failed to set up _TRACEBACK/epcall: {e}"))?;

        // Compatibility aliases and bootstrap (from original player.lua)
        let bootstrap = format!(
            r#"
-- Compatibility aliases (from player.lua)
nearest_enemy = get_nearest_enemy
exists = creature_exists

-- creature_config metatable
creature_config = setmetatable({{}}, {{
    __index = function(t, val)
        return creature_get_config(val)
    end
}})

-- needs_api function (validates API type)
do
    local _api_type = "{api_type}"
    function needs_api(needed)
        assert(needed == _api_type, "This Code needs the API '" .. needed .. "' but '" .. _api_type .. "' is loaded")
    end
end

-- Switch print to client_print
print = client_print

-- p() pretty-print helper
function p(x)
    if type(x) == "table" then
        print("+--- Table: " .. tostring(x))
        for key, val in pairs(x) do
            print("| " .. tostring(key) .. " " .. tostring(val))
        end
        print("+-----------------------")
    else
        print(type(x) .. " - " .. tostring(x))
    end
end

-- restart and info functions used by oo.lua
function restart()
    for id, creature in pairs(creatures) do
        creature:restart()
    end
end

function info()
    for id, creature in pairs(creatures) do
        print(tostring(creature))
    end
end

-- Default onCommand
function onCommand(cmd)
    print("huh? use '?' for help")
end

-- Instruction limit for coroutines: Lua 5.1 hooks are per-thread,
-- so we wrap coroutine.resume to install the hook on each coroutine.
do
    local _sethook = debug.sethook
    local _resume = coroutine.resume
    local _instruction_limit = {LUA_MAX_INSTRUCTIONS}
    coroutine.resume = function(co, ...)
        _sethook(co, function() error("lua vm cycles exceeded") end, "", _instruction_limit)
        local results = {{_resume(co, ...)}}
        _sethook(co)
        -- If resume failed with cycles exceeded, print so it appears in output
        if not results[1] and type(results[2]) == "string" and results[2]:find("cycles exceeded") then
            print("Lua error: " .. results[2])
        end
        return unpack(results)
    end
end

-- Disable dangerous functions for sandbox
debug = nil
load = nil
require = nil
loadfile = nil
os = nil
package = nil
io = nil
module = nil
collectgarbage = nil
"#
        );
        lua.load(&bootstrap)
            .set_name("bootstrap")
            .exec()
            .map_err(|e| format!("Failed to load bootstrap: {e}"))?;

        // Load the high-level API
        let api_code = match api_type {
            "oo" => include_str!("../../../orig_game/api/oo.lua"),
            "state" => include_str!("../../../orig_game/api/state.lua"),
            _ => return Err(format!("Unknown API type: {api_type}")),
        };
        lua.load(api_code)
            .set_name(&format!("api/{api_type}.lua"))
            .exec()
            .map_err(|e| format!("Failed to load {api_type} API: {e}"))?;

        // Load the default code for the API
        let default_code = match api_type {
            "oo" => include_str!("../../../orig_game/api/oo-default.lua"),
            "state" => include_str!("../../../orig_game/api/state-default.lua"),
            _ => "",
        };
        if !default_code.is_empty() {
            lua.load(default_code)
                .set_name(&format!("api/{api_type}-default.lua"))
                .exec()
                .map_err(|e| format!("Failed to load {api_type} defaults: {e}"))?;
        }

        Ok(Player {
            id,
            name: name.to_string(),
            score: 0,
            color: (id % 16) as u8,
            num_creatures: 0,
            lua,
            api_type: api_type.to_string(),
            output: Vec::new(),
        })
    }

    /// Load user bot code into the Lua VM.
    /// Game state must be set in app_data before calling this, so top-level
    /// bot code (e.g. `world_size()` calls) can access the game world.
    pub fn load_code(&self, code: &str) -> Result<(), String> {
        if !code.is_empty() {
            self.lua
                .load(code)
                .set_name("user_bot")
                .exec()
                .map_err(|e| format!("Failed to load bot code: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_player() {
        let player = Player::new(1, "TestBot", "oo");
        assert!(player.is_ok());
        let player = player.unwrap();
        assert_eq!(player.id, 1);
        assert_eq!(player.name, "TestBot");
        assert_eq!(player.score, 0);
        assert_eq!(player.api_type, "oo");
    }

    #[test]
    fn test_create_player_state_api() {
        let player = Player::new(2, "StateBot", "state");
        assert!(player.is_ok());
        let player = player.unwrap();
        assert_eq!(player.api_type, "state");
    }

    #[test]
    fn test_player_lua_constants() {
        let player = Player::new(1, "TestBot", "oo").unwrap();
        let lua = &player.lua;

        // Check creature type constants
        let val: i32 = lua.globals().get("CREATURE_SMALL").unwrap();
        assert_eq!(val, 0);
        let val: i32 = lua.globals().get("CREATURE_BIG").unwrap();
        assert_eq!(val, 1);
        let val: i32 = lua.globals().get("CREATURE_FLYER").unwrap();
        assert_eq!(val, 2);

        // Check creature state constants
        let val: i32 = lua.globals().get("CREATURE_IDLE").unwrap();
        assert_eq!(val, 0);
        let val: i32 = lua.globals().get("CREATURE_WALK").unwrap();
        assert_eq!(val, 1);
        let val: i32 = lua.globals().get("CREATURE_ATTACK").unwrap();
        assert_eq!(val, 4);

        // Check event constants
        let val: i32 = lua.globals().get("CREATURE_SPAWNED").unwrap();
        assert_eq!(val, 0);
        let val: i32 = lua.globals().get("CREATURE_KILLED").unwrap();
        assert_eq!(val, 1);
        let val: i32 = lua.globals().get("CREATURE_ATTACKED").unwrap();
        assert_eq!(val, 2);
        let val: i32 = lua.globals().get("PLAYER_CREATED").unwrap();
        assert_eq!(val, 3);

        // Check tile constants
        let val: i32 = lua.globals().get("TILE_SOLID").unwrap();
        assert_eq!(val, 0);
        let val: i32 = lua.globals().get("TILE_PLAIN").unwrap();
        assert_eq!(val, 1);

        // Check player_number
        let val: u32 = lua.globals().get("player_number").unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_create_player_invalid_api() {
        let result = Player::new(1, "Bad", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_player_think_exists() {
        let player = Player::new(1, "TestBot", "oo").unwrap();
        let lua = &player.lua;
        let _func: mlua::Function = lua.globals().get("player_think").unwrap();
        // If we got here without error, the function exists and is a Function type
    }
}
