import { useRef, useEffect, useState, useCallback } from 'react';
import type { WorldMsg, SnapshotMsg, GameEndMsg, PlayerSnapshot } from '../api/client';

const TILE_SIZE = 256; // game units per tile

const PLAYER_COLORS = [
  '#e94560', '#0f3460', '#16c79a', '#f5a623',
  '#9b59b6', '#1abc9c', '#e67e22', '#3498db',
];

interface GameCanvasProps {
  wsUrl: string;
}

export function GameCanvas({ wsUrl }: GameCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const worldRef = useRef<WorldMsg | null>(null);
  const snapshotRef = useRef<SnapshotMsg | null>(null);
  const [gameEnd, setGameEnd] = useState<GameEndMsg | null>(null);
  const [players, setPlayers] = useState<PlayerSnapshot[]>([]);
  const [gameTime, setGameTime] = useState(0);
  const [connected, setConnected] = useState(false);
  const animFrameRef = useRef<number>(0);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const world = worldRef.current;
    const snapshot = snapshotRef.current;
    if (!canvas || !world) {
      animFrameRef.current = requestAnimationFrame(draw);
      return;
    }

    const ctx = canvas.getContext('2d')!;
    const worldPixelWidth = world.width * TILE_SIZE;
    const worldPixelHeight = world.height * TILE_SIZE;
    const scale = Math.min(canvas.width / worldPixelWidth, canvas.height / worldPixelHeight);

    // Clear
    ctx.fillStyle = '#111';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw tiles
    for (const tile of world.tiles) {
      const px = tile.x * TILE_SIZE * scale;
      const py = tile.y * TILE_SIZE * scale;
      const size = TILE_SIZE * scale;

      // Color by type
      switch (tile.tile_type) {
        case 0: ctx.fillStyle = '#1a1a1a'; break; // solid
        case 1: ctx.fillStyle = '#1a2a1a'; break; // plain
        case 2: ctx.fillStyle = '#1a1a2e'; break; // water
        default: ctx.fillStyle = '#1a2a1a'; break;
      }
      ctx.fillRect(px, py, size, size);

      // Food overlay
      if (tile.food > 0) {
        const intensity = Math.min(tile.food / 5000, 1);
        ctx.fillStyle = `rgba(0, 200, 0, ${intensity * 0.4})`;
        ctx.fillRect(px, py, size, size);
      }

      // Tile border
      ctx.strokeStyle = 'rgba(255,255,255,0.04)';
      ctx.strokeRect(px, py, size, size);
    }

    // KOTH highlight
    const kx = world.koth_x * TILE_SIZE * scale;
    const ky = world.koth_y * TILE_SIZE * scale;
    ctx.fillStyle = 'rgba(255, 215, 0, 0.25)';
    ctx.fillRect(kx, ky, TILE_SIZE * scale, TILE_SIZE * scale);
    ctx.strokeStyle = 'rgba(255, 215, 0, 0.6)';
    ctx.lineWidth = 2;
    ctx.strokeRect(kx, ky, TILE_SIZE * scale, TILE_SIZE * scale);
    ctx.lineWidth = 1;

    // Draw creatures
    if (snapshot) {
      for (const c of snapshot.creatures) {
        const cx = c.x * scale;
        const cy = c.y * scale;
        const color = PLAYER_COLORS[c.player_id % PLAYER_COLORS.length];

        ctx.fillStyle = color;
        ctx.beginPath();
        if (c.creature_type === 0) {
          // Small - small circle
          ctx.arc(cx, cy, 4, 0, Math.PI * 2);
        } else if (c.creature_type === 1) {
          // Big - larger circle
          ctx.arc(cx, cy, 8, 0, Math.PI * 2);
        } else {
          // Flyer - triangle
          ctx.moveTo(cx, cy - 6);
          ctx.lineTo(cx - 5, cy + 4);
          ctx.lineTo(cx + 5, cy + 4);
          ctx.closePath();
        }
        ctx.fill();

        // Health bar background
        ctx.fillStyle = '#333';
        ctx.fillRect(cx - 8, cy - 14, 16, 3);
        // Health bar fill
        const healthPct = c.max_health > 0 ? c.health / c.max_health : 0;
        ctx.fillStyle = healthPct > 0.5 ? '#0f0' : healthPct > 0.25 ? '#ff0' : '#f00';
        ctx.fillRect(cx - 8, cy - 14, 16 * healthPct, 3);

        // Message
        if (c.message) {
          ctx.fillStyle = '#fff';
          ctx.font = '9px monospace';
          ctx.textAlign = 'center';
          ctx.fillText(c.message.substring(0, 20), cx, cy - 18);
        }
      }
    }

    animFrameRef.current = requestAnimationFrame(draw);
  }, []);

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
            break;
          case 'game_end':
            setGameEnd(msg);
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
    animFrameRef.current = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animFrameRef.current);
  }, [draw]);

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
        {gameEnd && (
          <div style={{
            position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)',
            background: 'rgba(0,0,0,0.85)', padding: '32px', borderRadius: '12px',
            color: '#e0e0e0', textAlign: 'center', border: '1px solid #333',
          }}>
            <h2 style={{ color: '#f5a623', margin: '0 0 16px 0' }}>Game Over</h2>
            {gameEnd.final_scores
              .sort((a, b) => b.score - a.score)
              .map((p, i) => (
                <div key={p.id} style={{ padding: '4px 0', color: i === 0 ? '#16c79a' : '#aaa' }}>
                  #{i + 1} {p.name}: {p.score} pts ({p.num_creatures} creatures)
                </div>
              ))}
          </div>
        )}
      </div>

      {/* Score panel */}
      <div style={{ width: '240px', background: '#16213e', borderLeft: '1px solid #333', padding: '16px', overflowY: 'auto' }}>
        <div style={{ marginBottom: '16px' }}>
          <div style={{ color: '#888', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
            Game Time
          </div>
          <div style={{ color: '#e0e0e0', fontSize: '24px', fontWeight: 700, fontFamily: 'monospace' }}>
            {Math.floor(gameTime / 1000)}s
          </div>
        </div>

        <div style={{ color: '#888', fontSize: '12px', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '8px' }}>
          Players
        </div>
        {players.length === 0 ? (
          <p style={{ color: '#666', fontSize: '13px' }}>Waiting for game data...</p>
        ) : (
          players
            .sort((a, b) => b.score - a.score)
            .map(p => (
              <div key={p.id} style={{
                padding: '8px',
                marginBottom: '8px',
                background: '#0a0a1a',
                borderRadius: '6px',
                borderLeft: `3px solid ${PLAYER_COLORS[p.id % PLAYER_COLORS.length]}`,
              }}>
                <div style={{ color: '#e0e0e0', fontWeight: 600, fontSize: '14px' }}>{p.name}</div>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px' }}>
                  <span style={{ color: '#16c79a', fontSize: '13px' }}>{p.score} pts</span>
                  <span style={{ color: '#888', fontSize: '13px' }}>{p.num_creatures} units</span>
                </div>
              </div>
            ))
        )}
      </div>
    </div>
  );
}
