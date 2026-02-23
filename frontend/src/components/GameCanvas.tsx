import { useRef, useEffect, useState, useCallback } from 'react';
import type { WorldMsg, SnapshotMsg, GameEndMsg, PlayerSnapshot, PlayerLoadErrorMsg, MatchDetail } from '../api/client';
import { api } from '../api/client';
import {
  getTileSpriteForGfx, isSnowGfx,
  FOOD_SPRITES, SNOW_FOOD_SPRITES,
  CREATURE_BASE_SPRITES, CREATURE_OVERLAY_SPRITES,
  THOUGHT_SPRITES, CROWN_SPRITE, KOTH_SPRITE,
  type SpriteRect
} from '../sprites/spriteAtlas';

const TILE_SIZE = 256; // game units per tile

const PLAYER_COLORS = [
  '#e94560', '#0f3460', '#16c79a', '#f5a623',
  '#9b59b6', '#1abc9c', '#e67e22', '#3498db',
];

function tileHash(x: number, y: number): number {
  // Simple hash for deterministic variant selection
  let h = x * 374761393 + y * 668265263;
  h = (h ^ (h >> 13)) * 1274126177;
  return (h ^ (h >> 16)) >>> 0;
}

function createTintedCreature(
  sheet: HTMLImageElement,
  baseRect: SpriteRect,
  overlayRect: SpriteRect,
  color: string
): HTMLCanvasElement {
  const size = 16;
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext('2d')!;

  // Draw base sprite
  ctx.drawImage(sheet, baseRect.x, baseRect.y, baseRect.w, baseRect.h, 0, 0, size, size);

  // Tint the base with player color using source-atop
  ctx.globalCompositeOperation = 'source-atop';
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, size, size);

  // Reset composite and draw overlay (eyes/outlines) on top
  ctx.globalCompositeOperation = 'source-over';
  ctx.drawImage(sheet, overlayRect.x, overlayRect.y, overlayRect.w, overlayRect.h, 0, 0, size, size);

  return canvas;
}

interface GameCanvasProps {
  wsUrl: string;
  onGameEnd?: () => void;
  onNewGame?: () => void;
}

export function GameCanvas({ wsUrl, onGameEnd, onNewGame }: GameCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const worldRef = useRef<WorldMsg | null>(null);
  const snapshotRef = useRef<SnapshotMsg | null>(null);
  const [gameEnd, setGameEnd] = useState<GameEndMsg | null>(null);
  const [players, setPlayers] = useState<PlayerSnapshot[]>([]);
  const [gameTime, setGameTime] = useState(0);
  const [connected, setConnected] = useState(false);
  const animFrameRef = useRef<number>(0);
  const [playerLoadErrors, setPlayerLoadErrors] = useState<PlayerLoadErrorMsg[]>([]);
  const [sidebarTab, setSidebarTab] = useState<'scores' | 'console'>('scores');
  const consoleLogRef = useRef<Map<number, string[]>>(new Map());
  const [consoleLogs, setConsoleLogs] = useState<Map<number, string[]>>(new Map());
  const [matchDetail, setMatchDetail] = useState<MatchDetail | null>(null);
  const onGameEndRef = useRef(onGameEnd);


  const spriteSheetRef = useRef<HTMLImageElement | null>(null);
  const spriteLoadedRef = useRef(false);
  const animTickRef = useRef(0);
  const lastAnimTimeRef = useRef(0);
  const creatureCacheRef = useRef<Map<string, HTMLCanvasElement>>(new Map());
  const drawRef = useRef<() => void>(() => {});
  const lastWorldRef = useRef<WorldMsg | null>(null);

  useEffect(() => { onGameEndRef.current = onGameEnd; }, [onGameEnd]);

  // Load sprite sheet
  useEffect(() => {
    const img = new Image();
    img.onload = () => {
      spriteSheetRef.current = img;
      spriteLoadedRef.current = true;
    };
    img.src = '/sprites/theme.png';
  }, []);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const world = worldRef.current;
    const snapshot = snapshotRef.current;
    if (!canvas || !world) {
      animFrameRef.current = requestAnimationFrame(drawRef.current);
      return;
    }

    const ctx = canvas.getContext('2d')!;
    const worldPixelWidth = world.width * TILE_SIZE;
    const worldPixelHeight = world.height * TILE_SIZE;
    const scale = Math.min(canvas.width / worldPixelWidth, canvas.height / worldPixelHeight);

    // Clear
    ctx.fillStyle = '#111';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const sheet = spriteSheetRef.current;
    if (!spriteLoadedRef.current || !sheet) {
      // Fallback: simple colored tiles while sprites load
      for (const tile of world.tiles) {
        const px = tile.x * TILE_SIZE * scale;
        const py = tile.y * TILE_SIZE * scale;
        const size = TILE_SIZE * scale;
        switch (tile.tile_type) {
          case 0: ctx.fillStyle = '#1a1a1a'; break;
          case 1: ctx.fillStyle = '#1a2a1a'; break;
          case 2: ctx.fillStyle = '#1a1a2e'; break;
          default: ctx.fillStyle = '#1a2a1a'; break;
        }
        ctx.fillRect(px, py, size, size);
      }
      ctx.fillStyle = '#888';
      ctx.font = '16px monospace';
      ctx.textAlign = 'center';
      ctx.fillText('Loading sprites...', canvas.width / 2, canvas.height / 2);
      animFrameRef.current = requestAnimationFrame(drawRef.current);
      return;
    }

    // Crisp pixel rendering
    ctx.imageSmoothingEnabled = false;

    // Clear creature cache when world changes (new players may have joined)
    if (lastWorldRef.current !== world) {
      creatureCacheRef.current.clear();
      lastWorldRef.current = world;
    }

    // Update animation tick every 128ms
    const now = performance.now();
    if (now - lastAnimTimeRef.current >= 128) {
      animTickRef.current++;
      lastAnimTimeRef.current = now;
    }

    const tileSize = TILE_SIZE * scale;

    // Draw tiles
    for (const tile of world.tiles) {
      const px = tile.x * TILE_SIZE * scale;
      const py = tile.y * TILE_SIZE * scale;

      const { sprites, animated } = getTileSpriteForGfx(tile.gfx);
      let sprite: SpriteRect;
      if (animated) {
        sprite = sprites[(animTickRef.current + tile.x + tile.y) % sprites.length];
      } else {
        sprite = sprites[tileHash(tile.x, tile.y) % sprites.length];
      }
      ctx.drawImage(sheet, sprite.x, sprite.y, sprite.w, sprite.h, px, py, tileSize, tileSize);

      // Food overlay
      if (tile.food > 0) {
        const level = Math.min(10, Math.ceil(tile.food / 1000));
        const foodSprites = isSnowGfx(tile.gfx) ? SNOW_FOOD_SPRITES : FOOD_SPRITES;
        const fs = foodSprites[level - 1];
        ctx.drawImage(sheet, fs.x, fs.y, fs.w, fs.h, px, py, tileSize, tileSize);
      }
    }

    // KOTH highlight
    const kx = world.koth_x * TILE_SIZE * scale;
    const ky = world.koth_y * TILE_SIZE * scale;
    // Draw KOTH sprite underneath the highlight
    ctx.drawImage(sheet, KOTH_SPRITE.x, KOTH_SPRITE.y, KOTH_SPRITE.w, KOTH_SPRITE.h, kx, ky, tileSize, tileSize);
    ctx.fillStyle = 'rgba(255, 215, 0, 0.25)';
    ctx.fillRect(kx, ky, tileSize, tileSize);
    ctx.strokeStyle = 'rgba(255, 215, 0, 0.6)';
    ctx.lineWidth = 2;
    ctx.strokeRect(kx, ky, tileSize, tileSize);
    ctx.lineWidth = 1;

    // Draw creatures
    if (snapshot) {
      for (const c of snapshot.creatures) {
        const cx = c.x * scale;
        const cy = c.y * scale;
        const color = PLAYER_COLORS[c.player_id % PLAYER_COLORS.length];
        const animFrame = animTickRef.current % 2;

        // Get or create cached tinted creature sprite
        const cacheKey = `${c.player_id}_${c.creature_type}_${animFrame}`;
        let cached = creatureCacheRef.current.get(cacheKey);
        if (!cached) {
          const baseRect = CREATURE_BASE_SPRITES[c.creature_type]?.[animFrame] ?? CREATURE_BASE_SPRITES[0][0];
          const overlayRect = CREATURE_OVERLAY_SPRITES[c.creature_type]?.[animFrame] ?? CREATURE_OVERLAY_SPRITES[0][0];
          cached = createTintedCreature(sheet, baseRect, overlayRect, color);
          creatureCacheRef.current.set(cacheKey, cached);
        }

        // Creature render size: ~80% of tile size, minimum 12px
        const renderSize = Math.max(12, Math.round(tileSize * 0.8));
        ctx.drawImage(cached, cx - renderSize / 2, cy - renderSize / 2, renderSize, renderSize);

        // Thought bubble
        const thoughtSprite = THOUGHT_SPRITES[c.message ? 8 : c.state] ?? THOUGHT_SPRITES[0];
        ctx.globalAlpha = 0.33;
        const thoughtSize = renderSize * 0.5;
        ctx.drawImage(
          sheet,
          thoughtSprite.x, thoughtSprite.y, thoughtSprite.w, thoughtSprite.h,
          cx + renderSize * 0.3, cy - renderSize * 0.6,
          thoughtSize, thoughtSize
        );
        ctx.globalAlpha = 1.0;

        // Health bar background
        const barWidth = renderSize;
        const barHeight = 3;
        const barX = cx - barWidth / 2;
        const barY = cy - renderSize / 2 - barHeight - 2;
        ctx.fillStyle = '#333';
        ctx.fillRect(barX, barY, barWidth, barHeight);
        // Health bar fill
        const healthPct = c.max_health > 0 ? c.health / c.max_health : 0;
        ctx.fillStyle = healthPct > 0.5 ? '#0f0' : healthPct > 0.25 ? '#ff0' : '#f00';
        ctx.fillRect(barX, barY, barWidth * healthPct, barHeight);

        // Message
        if (c.message) {
          ctx.fillStyle = '#fff';
          ctx.font = '9px monospace';
          ctx.textAlign = 'center';
          ctx.fillText(c.message.substring(0, 20), cx, barY - 4);
        }
      }

      // KOTH crown
      if (snapshot.king_player_id != null) {
        const crownX = kx + tileSize / 2;
        const bobY = Math.sin(Date.now() / 500) * 3;
        const crownSize = tileSize * 0.8;
        ctx.drawImage(
          sheet,
          CROWN_SPRITE.x, CROWN_SPRITE.y, CROWN_SPRITE.w, CROWN_SPRITE.h,
          crownX - crownSize / 2, ky - crownSize * 0.6 + bobY,
          crownSize, crownSize * (CROWN_SPRITE.h / CROWN_SPRITE.w)
        );
      }
    }

    animFrameRef.current = requestAnimationFrame(drawRef.current);
  }, []);
  useEffect(() => { drawRef.current = draw; }, [draw]);

  // WebSocket connection
  useEffect(() => {
    const ws = new WebSocket(wsUrl);
    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onerror = () => setConnected(false);

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        switch (msg.type) {
          case 'world':
            worldRef.current = msg;
            break;
          case 'snapshot':
            snapshotRef.current = msg;
            setPlayers(msg.players || []);
            setGameTime(msg.game_time || 0);
            // Accumulate player output for console
            {
              let hasNew = false;
              for (const p of (msg.players || [])) {
                if (p.output && p.output.length > 0) {
                  const existing = consoleLogRef.current.get(p.id) || [];
                  const combined = [...existing, ...p.output];
                  // Cap at 500 lines per player
                  consoleLogRef.current.set(p.id, combined.slice(-500));
                  hasNew = true;
                }
              }
              if (hasNew) setConsoleLogs(new Map(consoleLogRef.current));
            }
            break;
          case 'player_load_error':
            setPlayerLoadErrors(prev => [...prev, msg]);
            break;
          case 'game_end':
            setGameEnd(msg);
            onGameEndRef.current?.();
            break;
        }
      } catch {
        // ignore parse errors
      }
    };

    return () => ws.close();
  }, [wsUrl]);

  // Render loop
  useEffect(() => {
    animFrameRef.current = requestAnimationFrame(drawRef.current);
    return () => cancelAnimationFrame(animFrameRef.current);
  }, [draw]);

  // Fetch match detail (for Elo) when game ends with a match_id
  useEffect(() => {
    if (!gameEnd?.match_id) return;
    const matchId = gameEnd.match_id;
    let cancelled = false;

    const poll = async () => {
      // Wait a bit for the completion callback to finish
      await new Promise(r => setTimeout(r, 2000));
      for (let attempt = 0; attempt < 5 && !cancelled; attempt++) {
        try {
          const detail = await api.getMatch(matchId);
          if (!cancelled) {
            setMatchDetail(detail);
            if (detail.match.status === 'finished') return;
          }
        } catch { /* ignore */ }
        await new Promise(r => setTimeout(r, 1500));
      }
    };
    poll();
    return () => { cancelled = true; };
  }, [gameEnd]);

  // Resize canvas to fit container
  useEffect(() => {
    const resize = () => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const parent = canvas.parentElement;
      if (!parent) return;
      canvas.width = parent.clientWidth;
      canvas.height = parent.clientHeight;
    };
    resize();
    window.addEventListener('resize', resize);
    return () => window.removeEventListener('resize', resize);
  }, []);

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      {/* Canvas area */}
      <div style={{ flex: 1, position: 'relative', minHeight: 0 }}>
        <canvas ref={canvasRef} style={{ display: 'block', width: '100%', height: '100%', background: '#111' }} />
        {!connected && (
          <div style={{
            position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)',
            color: '#888', fontSize: '18px', textAlign: 'center',
          }}>
            Connecting to game server...
          </div>
        )}
        {gameEnd && (() => {
          const sorted = [...gameEnd.final_scores].sort((a, b) => b.score - a.score);
          const winnerName = gameEnd.winner != null
            ? gameEnd.final_scores.find(p => p.id === gameEnd.winner)?.name ?? 'Unknown'
            : null;
          const ticks = gameEnd.game_duration_ticks ?? 0;
          const totalSec = Math.round(ticks * 0.1);
          const mins = Math.floor(totalSec / 60);
          const secs = totalSec % 60;
          const durationStr = `${mins}:${secs.toString().padStart(2, '0')}`;

          // Build elo lookup from match detail participants
          const eloMap = new Map<number, { before: number; after: number }>();
          if (matchDetail?.match.status === 'finished') {
            for (const mp of matchDetail.participants) {
              if (mp.elo_before != null && mp.elo_after != null) {
                eloMap.set(mp.player_slot, { before: mp.elo_before, after: mp.elo_after });
              }
            }
          }

          return (
            <div style={{
              position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)',
              background: 'rgba(0,0,0,0.92)', padding: '28px 36px', borderRadius: '12px',
              color: '#e0e0e0', border: '1px solid #444', minWidth: '360px', maxWidth: '500px',
              fontFamily: 'monospace',
            }}>
              <h2 style={{ color: '#f5a623', margin: '0 0 4px 0', textAlign: 'center', fontSize: '20px' }}>
                GAME OVER
              </h2>
              {winnerName && (
                <div style={{ textAlign: 'center', color: '#16c79a', fontSize: '14px', marginBottom: '8px' }}>
                  Winner: {winnerName}
                </div>
              )}
              <div style={{ textAlign: 'center', color: '#888', fontSize: '12px', marginBottom: '16px' }}>
                Duration: {durationStr} ({ticks.toLocaleString()} ticks)
              </div>

              {sorted.map((p, i) => {
                const stats = gameEnd.player_stats?.find(s => s.player_id === p.id);
                const elo = eloMap.get(i);
                const isWinner = gameEnd.winner != null && p.id === gameEnd.winner;
                return (
                  <div key={p.id} style={{
                    padding: '8px 10px', marginBottom: '8px', background: '#111',
                    borderRadius: '6px',
                    borderLeft: `3px solid ${PLAYER_COLORS[p.id % PLAYER_COLORS.length]}`,
                  }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <span style={{ color: isWinner ? '#16c79a' : '#ccc', fontWeight: 600, fontSize: '14px' }}>
                        #{i + 1} {p.name}
                      </span>
                      <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                        <span style={{ color: '#f5a623', fontSize: '14px' }}>{p.score.toLocaleString()} pts</span>
                        {isWinner && <span style={{ color: '#16c79a', fontWeight: 700, fontSize: '12px' }}>W</span>}
                      </span>
                    </div>
                    {stats && (
                      <div style={{ color: '#888', fontSize: '11px', marginTop: '4px' }}>
                        Spawned: {stats.creatures_spawned} &nbsp; Killed: {stats.creatures_killed} &nbsp; Lost: {stats.creatures_lost}
                      </div>
                    )}
                    {elo && (
                      <div style={{ fontSize: '11px', marginTop: '2px' }}>
                        <span style={{ color: '#888' }}>Elo: {elo.before}</span>
                        <span style={{ color: '#888' }}> → {elo.after} </span>
                        <span style={{ color: elo.after >= elo.before ? '#16c79a' : '#e94560', fontWeight: 600 }}>
                          ({elo.after >= elo.before ? '+' : ''}{elo.after - elo.before})
                        </span>
                      </div>
                    )}
                  </div>
                );
              })}

              <div style={{ display: 'flex', justifyContent: 'center', gap: '12px', marginTop: '16px' }}>
                {gameEnd.match_id && (
                  <a
                    href={`/replay/${gameEnd.match_id}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{
                      padding: '8px 20px', borderRadius: '6px', fontSize: '13px', fontWeight: 600,
                      color: '#f5a623', border: '1px solid #f5a623', background: 'transparent',
                      textDecoration: 'none', cursor: 'pointer',
                    }}
                  >
                    Watch Replay
                  </a>
                )}
                {onNewGame && (
                  <button
                    onClick={onNewGame}
                    style={{
                      padding: '8px 20px', borderRadius: '6px', fontSize: '13px', fontWeight: 600,
                      color: '#fff', background: '#16c79a', border: 'none', cursor: 'pointer',
                    }}
                  >
                    New Game
                  </button>
                )}
              </div>
            </div>
          );
        })()}
      </div>

      {/* Sidebar panel */}
      <div style={{ width: '240px', background: '#16213e', borderLeft: '1px solid #333', display: 'flex', flexDirection: 'column' }}>
        {/* Tab strip */}
        <div style={{ display: 'flex', borderBottom: '1px solid #333' }}>
          {(['scores', 'console'] as const).map(tab => (
            <button
              key={tab}
              onClick={() => setSidebarTab(tab)}
              style={{
                flex: 1, padding: '8px', border: 'none', cursor: 'pointer',
                background: sidebarTab === tab ? '#16213e' : '#0a0a1a',
                color: sidebarTab === tab ? '#e0e0e0' : '#666',
                fontWeight: sidebarTab === tab ? 600 : 400,
                fontSize: '13px', textTransform: 'capitalize',
                borderBottom: sidebarTab === tab ? '2px solid #f5a623' : '2px solid transparent',
              }}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* Tab content */}
        <div style={{ flex: 1, overflowY: 'auto', padding: '16px' }}>
          {sidebarTab === 'scores' ? (
            <>
              <div style={{ marginBottom: '16px' }}>
                <div style={{ color: '#888', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
                  Game Time
                </div>
                <div style={{ color: '#e0e0e0', fontSize: '24px', fontWeight: 700, fontFamily: 'monospace' }}>
                  {Math.floor(gameTime / 1000)}s
                </div>
              </div>

              {playerLoadErrors.length > 0 && (
                <>
                  <div style={{ color: '#e94560', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '8px' }}>
                    Load Errors
                  </div>
                  {playerLoadErrors.map((err, i) => (
                    <div key={i} style={{
                      padding: '8px',
                      marginBottom: '8px',
                      background: '#2a0a0a',
                      borderRadius: '6px',
                      borderLeft: '3px solid #e94560',
                    }}>
                      <div style={{ color: '#ff8a8a', fontWeight: 600, fontSize: '13px', marginBottom: '4px' }}>
                        {err.player_name}
                      </div>
                      <div style={{ color: '#cc6666', fontSize: '12px', fontFamily: 'monospace', whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                        {err.error}
                      </div>
                    </div>
                  ))}
                </>
              )}

              {players.length === 0 && playerLoadErrors.length === 0 ? (
                <p style={{ color: '#666', fontSize: '13px' }}>Waiting for game data...</p>
              ) : (
                <>
                  {players.length > 0 && (
                    <div style={{ color: '#888', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '8px' }}>
                      Players
                    </div>
                  )}
                  {players
                    .sort((a, b) => b.score - a.score)
                    .map(p => (
                      <div key={p.id} style={{
                        padding: '8px',
                        marginBottom: '8px',
                        background: '#0a0a1a',
                        borderRadius: '6px',
                        borderLeft: `3px solid ${PLAYER_COLORS[p.id % PLAYER_COLORS.length]}`,
                      }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                          <span style={{
                            display: 'inline-block', width: '10px', height: '10px', borderRadius: '2px', flexShrink: 0,
                            background: PLAYER_COLORS[p.id % PLAYER_COLORS.length],
                          }} />
                          <span style={{ color: '#e0e0e0', fontWeight: 600, fontSize: '14px' }}>{p.name}</span>
                        </div>
                        <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px' }}>
                          <span style={{ color: '#16c79a', fontSize: '13px' }}>{p.score} pts</span>
                          <span style={{ color: '#888', fontSize: '13px' }}>{p.num_creatures} units</span>
                        </div>
                      </div>
                    ))}
                </>
              )}
            </>
          ) : (
            // Console tab
            <div>
              {players.length === 0 ? (
                <p style={{ color: '#666', fontSize: '13px' }}>Waiting for game data...</p>
              ) : (
                players.map(p => {
                  const lines = consoleLogs.get(p.id) || [];
                  return (
                    <div key={p.id} style={{ marginBottom: '12px' }}>
                      <div style={{
                        color: PLAYER_COLORS[p.id % PLAYER_COLORS.length],
                        fontWeight: 600, fontSize: '12px', marginBottom: '4px',
                      }}>
                        {p.name}
                      </div>
                      <div style={{
                        background: '#0a0a1a', borderRadius: '4px', padding: '6px',
                        fontFamily: 'monospace', fontSize: '11px', maxHeight: '150px',
                        overflowY: 'auto', whiteSpace: 'pre-wrap', wordBreak: 'break-all',
                      }}>
                        {lines.length === 0 ? (
                          <span style={{ color: '#444' }}>No output</span>
                        ) : (
                          lines.map((line, i) => (
                            <div key={i} style={{ color: line.startsWith('Lua error') ? '#e94560' : '#aaa' }}>
                              {line}
                            </div>
                          ))
                        )}
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
