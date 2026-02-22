const BASE_URL = '';

function authHeaders(): Record<string, string> {
  const token = localStorage.getItem('infon_token');
  if (token) {
    return { Authorization: `Bearer ${token}` };
  }
  return {};
}

export interface Bot {
  id: number;
  name: string;
  description: string;
  owner_id: number | null;
  visibility: string;
  created_at: string;
  updated_at: string;
}

export interface BotVersion {
  id: number;
  bot_id: number;
  version: number;
  code: string;
  api_type: string;
  created_at: string;
}

export interface Tournament {
  id: number;
  name: string;
  status: string;
  map: string;
  created_at: string;
  updated_at: string;
}

export interface TournamentEntry {
  id: number;
  tournament_id: number;
  bot_version_id: number;
  slot_name: string;
  bot_name?: string;
  version?: number;
}

export interface MapInfo {
  name: string;
  width: number;
  height: number;
  description: string;
}

export interface TournamentResult {
  player_id: number;
  player_name: string;
  score: number;
  num_creatures: number;
  kills: number;
}

// WebSocket message types
export interface WorldMsg {
  type: 'world';
  width: number;
  height: number;
  koth_x: number;
  koth_y: number;
  tiles: TileSnapshot[];
}

export interface SnapshotMsg {
  type: 'snapshot';
  game_time: number;
  creatures: CreatureSnapshot[];
  players: PlayerSnapshot[];
  king_player_id?: number;
}

export interface GameEndMsg {
  type: 'game_end';
  winner?: number;
  final_scores: PlayerSnapshot[];
}

export type GameMessage = WorldMsg | SnapshotMsg | GameEndMsg;

export interface TileSnapshot {
  x: number;
  y: number;
  food: number;
  tile_type: number;
  gfx: number;
}

export interface CreatureSnapshot {
  id: number;
  x: number;
  y: number;
  creature_type: number;
  state: number;
  health: number;
  max_health: number;
  food: number;
  player_id: number;
  message: string;
  target_id?: number;
}

export interface PlayerSnapshot {
  id: number;
  name: string;
  score: number;
  color: number;
  num_creatures: number;
  output?: string[];
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text().catch(() => 'Unknown error');
    throw new Error(`API error ${response.status}: ${text}`);
  }
  return response.json();
}

export const api = {
  // Bots
  listBots: (): Promise<Bot[]> =>
    fetch(`${BASE_URL}/api/bots`, { headers: authHeaders() }).then(r => handleResponse<Bot[]>(r)),

  createBot: (name: string, description?: string): Promise<Bot> =>
    fetch(`${BASE_URL}/api/bots`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ name, description }),
    }).then(r => handleResponse<Bot>(r)),

  getBot: (id: number): Promise<Bot> =>
    fetch(`${BASE_URL}/api/bots/${id}`, { headers: authHeaders() }).then(r => handleResponse<Bot>(r)),

  updateBot: (id: number, name: string, description?: string): Promise<Bot> =>
    fetch(`${BASE_URL}/api/bots/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ name, description }),
    }).then(r => handleResponse<Bot>(r)),

  deleteBot: (id: number): Promise<void> =>
    fetch(`${BASE_URL}/api/bots/${id}`, { method: 'DELETE', headers: authHeaders() }).then(r => {
      if (!r.ok) throw new Error(`Delete failed: ${r.status}`);
    }),

  // Versions
  listVersions: (botId: number): Promise<BotVersion[]> =>
    fetch(`${BASE_URL}/api/bots/${botId}/versions`, { headers: authHeaders() }).then(r => handleResponse<BotVersion[]>(r)),

  createVersion: (botId: number, code: string, apiType?: string): Promise<BotVersion> =>
    fetch(`${BASE_URL}/api/bots/${botId}/versions`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ code, api_type: apiType || 'oo' }),
    }).then(r => handleResponse<BotVersion>(r)),

  getVersion: (botId: number, versionId: number): Promise<BotVersion> =>
    fetch(`${BASE_URL}/api/bots/${botId}/versions/${versionId}`, { headers: authHeaders() }).then(r => handleResponse<BotVersion>(r)),

  // Tournaments
  listTournaments: (): Promise<Tournament[]> =>
    fetch(`${BASE_URL}/api/tournaments`, { headers: authHeaders() }).then(r => handleResponse<Tournament[]>(r)),

  createTournament: (name: string, map?: string): Promise<Tournament> =>
    fetch(`${BASE_URL}/api/tournaments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ name, map }),
    }).then(r => handleResponse<Tournament>(r)),

  getTournament: (id: number): Promise<Tournament> =>
    fetch(`${BASE_URL}/api/tournaments/${id}`, { headers: authHeaders() }).then(r => handleResponse<Tournament>(r)),

  listEntries: (tournamentId: number): Promise<TournamentEntry[]> =>
    fetch(`${BASE_URL}/api/tournaments/${tournamentId}/entries`, { headers: authHeaders() }).then(r => handleResponse<TournamentEntry[]>(r)),

  addEntry: (tournamentId: number, botVersionId: number, slotName?: string): Promise<TournamentEntry> =>
    fetch(`${BASE_URL}/api/tournaments/${tournamentId}/entries`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ bot_version_id: botVersionId, slot_name: slotName }),
    }).then(r => handleResponse<TournamentEntry>(r)),

  removeEntry: (tournamentId: number, entryId: number): Promise<void> =>
    fetch(`${BASE_URL}/api/tournaments/${tournamentId}/entries/${entryId}`, {
      method: 'DELETE',
      headers: authHeaders(),
    }).then(r => {
      if (!r.ok) throw new Error(`Delete failed: ${r.status}`);
    }),

  runTournament: (id: number): Promise<void> =>
    fetch(`${BASE_URL}/api/tournaments/${id}/run`, { method: 'POST', headers: authHeaders() }).then(r => {
      if (!r.ok) throw new Error(`Run failed: ${r.status}`);
    }),

  getResults: (tournamentId: number): Promise<TournamentResult[]> =>
    fetch(`${BASE_URL}/api/tournaments/${tournamentId}/results`, { headers: authHeaders() }).then(r => handleResponse<TournamentResult[]>(r)),

  // Game
  gameStatus: (): Promise<{ running: boolean }> =>
    fetch(`${BASE_URL}/api/game/status`).then(r => handleResponse<{ running: boolean }>(r)),

  listMaps: (): Promise<MapInfo[]> =>
    fetch(`${BASE_URL}/api/maps`).then(r => handleResponse<MapInfo[]>(r)),

  startGame: (players: { bot_version_id: number; name?: string }[], map?: string): Promise<{ status: string; message: string }> =>
    fetch(`${BASE_URL}/api/game/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...authHeaders() },
      body: JSON.stringify({ players, map }),
    }).then(r => handleResponse<{ status: string; message: string }>(r)),

  stopGame: (): Promise<void> =>
    fetch(`${BASE_URL}/api/game/stop`, { method: 'POST', headers: authHeaders() }).then(r => {
      if (!r.ok) throw new Error(`Stop failed: ${r.status}`);
    }),
};
