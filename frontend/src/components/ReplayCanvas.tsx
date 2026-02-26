import { useRef, useEffect, useState, useCallback } from 'react';
import type { GameMessage, WorldMsg, SnapshotMsg, GameEndMsg, PlayerSnapshot } from '../api/client';
import {
  getTileSpriteForGfx, isSnowGfx,
  FOOD_SPRITES, SNOW_FOOD_SPRITES,
  CREATURE_BASE_SPRITES, CREATURE_OVERLAY_SPRITES,
  THOUGHT_SPRITES, CROWN_SPRITE, KOTH_SPRITE,
  type SpriteRect
} from '../sprites/spriteAtlas';

const TILE_SIZE = 256;

const PLAYER_COLORS = [
  '#e94560', '#0f3460', '#16c79a', '#f5a623',
  '#9b59b6', '#1abc9c', '#e67e22', '#3498db',
];

function tileHash(x: number, y: number): number {
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
  ctx.drawImage(sheet, baseRect.x, baseRect.y, baseRect.w, baseRect.h, 0, 0, size, size);
  ctx.globalCompositeOperation = 'source-atop';
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, size, size);
  ctx.globalCompositeOperation = 'source-over';
  ctx.drawImage(sheet, overlayRect.x, overlayRect.y, overlayRect.w, overlayRect.h, 0, 0, size, size);
  return canvas;
}

interface ReplayCanvasProps {
  messages: GameMessage[];
  tickCount: number;
}

export function ReplayCanvas({ messages }: ReplayCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const worldRef = useRef<WorldMsg | null>(null);
  const snapshotRef = useRef<SnapshotMsg | null>(null);
  const [gameEnd, setGameEnd] = useState<GameEndMsg | null>(null);
  const [players, setPlayers] = useState<PlayerSnapshot[]>([]);
  const [gameTime, setGameTime] = useState(0);
  const animFrameRef = useRef<number>(0);

  const spriteSheetRef = useRef<HTMLImageElement | null>(null);
  const spriteLoadedRef = useRef(false);
  const animTickRef = useRef(0);
  const lastAnimTimeRef = useRef(0);
  const creatureCacheRef = useRef<Map<string, HTMLCanvasElement>>(new Map());
  const drawRef = useRef<() => void>(() => {});
  const lastWorldRef = useRef<WorldMsg | null>(null);

  // Viewport state for zoom/pan
  const viewportRef = useRef({ offsetX: 0, offsetY: 0, zoom: 1 });
    const isPanningRef = useRef(false);
  const lastPanRef = useRef({ x: 0, y: 0 });
  const spaceDownRef = useRef(false);

  // Playback state
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState(1);
  const [currentIndex, setCurrentIndex] = useState(0);
  const playIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Build snapshot indices: indices of messages that are snapshots
  const snapshotIndices = useRef<number[]>([]);
  useEffect(() => {
    const indices: number[] = [];
    for (let i = 0; i < messages.length; i++) {
      if (messages[i].type === 'snapshot') {
        indices.push(i);
      }
    }
    snapshotIndices.current = indices;
  }, [messages]);

  // Load sprite sheet
  useEffect(() => {
    const img = new Image();
    img.onload = () => {
      spriteSheetRef.current = img;
      spriteLoadedRef.current = true;
    };
    img.src = '/sprites/theme.png';
  }, []);

  // Apply messages when currentIndex changes
  /* eslint-disable react-hooks/set-state-in-effect -- Replay scrubbing requires synchronous state derivation from message index */
  useEffect(() => {
    worldRef.current = null;
    snapshotRef.current = null;
    setGameEnd(null);

    for (let i = 0; i <= currentIndex && i < messages.length; i++) {
      const msg = messages[i];
      switch (msg.type) {
        case 'world':
          worldRef.current = msg;
          creatureCacheRef.current.clear();
          lastWorldRef.current = null;
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
    }
  }, [currentIndex, messages]);
  /* eslint-enable react-hooks/set-state-in-effect */

  // Playback timer
  useEffect(() => {
    if (playing) {
      const intervalMs = Math.max(10, Math.round(100 / speed));
      playIntervalRef.current = setInterval(() => {
        setCurrentIndex(prev => {
          const next = prev + 1;
          if (next >= messages.length) {
            setPlaying(false);
            return prev;
          }
          return next;
        });
      }, intervalMs);
    } else {
      if (playIntervalRef.current) {
        clearInterval(playIntervalRef.current);
        playIntervalRef.current = null;
      }
    }
    return () => {
      if (playIntervalRef.current) {
        clearInterval(playIntervalRef.current);
      }
    };
  }, [playing, speed, messages.length]);

  // Draw function (same rendering as GameCanvas)
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
    const baseScale = Math.min(canvas.width / worldPixelWidth, canvas.height / worldPixelHeight);
    const vp = viewportRef.current;
    const scale = baseScale * vp.zoom;

    ctx.fillStyle = '#111';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const sheet = spriteSheetRef.current;
    if (!spriteLoadedRef.current || !sheet) {
      ctx.fillStyle = '#888';
      ctx.font = '16px monospace';
      ctx.textAlign = 'center';
      ctx.fillText('Loading sprites...', canvas.width / 2, canvas.height / 2);
      animFrameRef.current = requestAnimationFrame(drawRef.current);
      return;
    }

    ctx.imageSmoothingEnabled = false;

    ctx.save();
    ctx.translate(vp.offsetX, vp.offsetY);

    if (lastWorldRef.current !== world) {
      creatureCacheRef.current.clear();
      lastWorldRef.current = world;
    }

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
      if (tile.food > 0) {
        const level = Math.min(10, Math.ceil(tile.food / 1000));
        const foodSprites = isSnowGfx(tile.gfx) ? SNOW_FOOD_SPRITES : FOOD_SPRITES;
        const fs = foodSprites[level - 1];
        ctx.drawImage(sheet, fs.x, fs.y, fs.w, fs.h, px, py, tileSize, tileSize);
      }
    }

    // KOTH
    const kx = world.koth_x * TILE_SIZE * scale;
    const ky = world.koth_y * TILE_SIZE * scale;
    ctx.drawImage(sheet, KOTH_SPRITE.x, KOTH_SPRITE.y, KOTH_SPRITE.w, KOTH_SPRITE.h, kx, ky, tileSize, tileSize);
    ctx.fillStyle = 'rgba(255, 215, 0, 0.25)';
    ctx.fillRect(kx, ky, tileSize, tileSize);
    ctx.strokeStyle = 'rgba(255, 215, 0, 0.6)';
    ctx.lineWidth = 2;
    ctx.strokeRect(kx, ky, tileSize, tileSize);
    ctx.lineWidth = 1;

    // Creatures
    if (snapshot) {
      for (const c of snapshot.creatures) {
        const cx = c.x * scale;
        const cy = c.y * scale;
        const color = PLAYER_COLORS[c.player_id % PLAYER_COLORS.length];
        const animFrame = animTickRef.current % 2;
        const cacheKey = `${c.player_id}_${c.creature_type}_${animFrame}`;
        let cached = creatureCacheRef.current.get(cacheKey);
        if (!cached) {
          const baseRect = CREATURE_BASE_SPRITES[c.creature_type]?.[animFrame] ?? CREATURE_BASE_SPRITES[0][0];
          const overlayRect = CREATURE_OVERLAY_SPRITES[c.creature_type]?.[animFrame] ?? CREATURE_OVERLAY_SPRITES[0][0];
          cached = createTintedCreature(sheet, baseRect, overlayRect, color);
          creatureCacheRef.current.set(cacheKey, cached);
        }
        const renderSize = Math.max(12, Math.round(tileSize * 0.8));
        ctx.drawImage(cached, cx - renderSize / 2, cy - renderSize / 2, renderSize, renderSize);
        const thoughtSprite = THOUGHT_SPRITES[c.message ? 8 : c.state] ?? THOUGHT_SPRITES[0];
        ctx.globalAlpha = 0.33;
        const thoughtSize = renderSize * 0.5;
        ctx.drawImage(sheet, thoughtSprite.x, thoughtSprite.y, thoughtSprite.w, thoughtSprite.h,
          cx + renderSize * 0.3, cy - renderSize * 0.6, thoughtSize, thoughtSize);
        ctx.globalAlpha = 1.0;
        const barWidth = renderSize;
        const barHeight = 3;
        const barX = cx - barWidth / 2;
        const barY = cy - renderSize / 2 - barHeight - 2;
        ctx.fillStyle = '#333';
        ctx.fillRect(barX, barY, barWidth, barHeight);
        const healthPct = c.max_health > 0 ? c.health / c.max_health : 0;
        ctx.fillStyle = healthPct > 0.5 ? '#0f0' : healthPct > 0.25 ? '#ff0' : '#f00';
        ctx.fillRect(barX, barY, barWidth * healthPct, barHeight);
        if (c.message) {
          ctx.fillStyle = '#fff';
          ctx.font = '9px monospace';
          ctx.textAlign = 'center';
          ctx.fillText(c.message.substring(0, 20), cx, barY - 4);
        }
      }
      if (snapshot.king_player_id != null) {
        const crownX = kx + tileSize / 2;
        const bobY = Math.sin(Date.now() / 500) * 3;
        const crownSize = tileSize * 0.8;
        ctx.drawImage(sheet, CROWN_SPRITE.x, CROWN_SPRITE.y, CROWN_SPRITE.w, CROWN_SPRITE.h,
          crownX - crownSize / 2, ky - crownSize * 0.6 + bobY, crownSize, crownSize * (CROWN_SPRITE.h / CROWN_SPRITE.w));
      }
    }

    ctx.restore();

    // Zoom indicator
    if (vp.zoom > 1.01) {
      ctx.fillStyle = 'rgba(0,0,0,0.6)';
      ctx.fillRect(canvas.width - 60, 8, 52, 22);
      ctx.fillStyle = '#f5a623';
      ctx.font = '12px monospace';
      ctx.textAlign = 'right';
      ctx.fillText(`${vp.zoom.toFixed(1)}x`, canvas.width - 14, 23);
    }

    animFrameRef.current = requestAnimationFrame(drawRef.current);
  }, []);
  useEffect(() => { drawRef.current = draw; }, [draw]);

  // Zoom/pan event handlers
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const clampPan = () => {
      const world = worldRef.current;
      if (!world) return;
      const vp = viewportRef.current;
      const worldPixelW = world.width * TILE_SIZE;
      const worldPixelH = world.height * TILE_SIZE;
      const bs = Math.min(canvas.width / worldPixelW, canvas.height / worldPixelH);
      const s = bs * vp.zoom;
      const scaledW = worldPixelW * s;
      const scaledH = worldPixelH * s;
      vp.offsetX = Math.min(0, Math.max(canvas.width - scaledW, vp.offsetX));
      vp.offsetY = Math.min(0, Math.max(canvas.height - scaledH, vp.offsetY));
    };

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const vp = viewportRef.current;
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      const oldZoom = vp.zoom;
      const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
      vp.zoom = Math.min(10, Math.max(1, vp.zoom * factor));
      const zoomRatio = vp.zoom / oldZoom;
      vp.offsetX = mx - (mx - vp.offsetX) * zoomRatio;
      vp.offsetY = my - (my - vp.offsetY) * zoomRatio;
      clampPan();
    };

    const onMouseDown = (e: MouseEvent) => {
      if (e.button === 1 || (e.button === 0 && spaceDownRef.current)) {
        isPanningRef.current = true;
        lastPanRef.current = { x: e.clientX, y: e.clientY };
        e.preventDefault();
      }
    };

    const onMouseMove = (e: MouseEvent) => {
      if (!isPanningRef.current) return;
      const vp = viewportRef.current;
      vp.offsetX += e.clientX - lastPanRef.current.x;
      vp.offsetY += e.clientY - lastPanRef.current.y;
      lastPanRef.current = { x: e.clientX, y: e.clientY };
      clampPan();
    };

    const onMouseUp = () => { isPanningRef.current = false; };

    const onDblClick = () => {
      viewportRef.current = { offsetX: 0, offsetY: 0, zoom: 1 };
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.code === 'Space') { spaceDownRef.current = true; e.preventDefault(); }
    };
    const onKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') spaceDownRef.current = false;
    };

    canvas.addEventListener('wheel', onWheel, { passive: false });
    canvas.addEventListener('mousedown', onMouseDown);
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    canvas.addEventListener('dblclick', onDblClick);
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);

    return () => {
      canvas.removeEventListener('wheel', onWheel);
      canvas.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
      canvas.removeEventListener('dblclick', onDblClick);
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
    };
  }, []);

  // Render loop
  useEffect(() => {
    animFrameRef.current = requestAnimationFrame(drawRef.current);
    return () => cancelAnimationFrame(animFrameRef.current);
  }, [draw]);

  // Resize canvas
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

  const handleScrub = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = Number(e.target.value);
    setCurrentIndex(val);
  };

  const togglePlay = () => {
    if (currentIndex >= messages.length - 1) {
      // Reset to start
      setCurrentIndex(0);
      setPlaying(true);
    } else {
      setPlaying(!playing);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Canvas area */}
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        <div style={{ flex: 1, position: 'relative', minHeight: 0 }}>
          <canvas ref={canvasRef} style={{ display: 'block', width: '100%', height: '100%', background: '#111' }} />
          {gameEnd && (
            <div style={{
              position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)',
              background: 'rgba(0,0,0,0.85)', padding: '32px', borderRadius: '12px',
              color: '#e0e0e0', textAlign: 'center', border: '1px solid #333',
            }}>
              <h2 style={{ color: '#f5a623', margin: '0 0 16px 0' }}>Game Over</h2>
              {[...gameEnd.final_scores]
                .sort((a, b) => b.score - a.score)
                .map((p, i) => (
                  <div key={p.id} style={{ padding: '4px 0', color: i === 0 ? '#16c79a' : '#aaa' }}>
                    #{i + 1} {p.name}: {p.score} pts ({p.num_creatures} creatures)
                  </div>
                ))}
            </div>
          )}
        </div>

        {/* Sidebar */}
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
            <p style={{ color: '#666', fontSize: '13px' }}>No data yet</p>
          ) : (
            [...players]
              .sort((a, b) => b.score - a.score)
              .map(p => {
                const dead = p.num_creatures === 0;
                const playerColor = PLAYER_COLORS[p.id % PLAYER_COLORS.length];
                return (
                  <div key={p.id} style={{
                    padding: '8px', marginBottom: '8px', background: '#0a0a1a', borderRadius: '6px',
                    borderLeft: `3px solid ${dead ? '#444' : playerColor}`,
                    opacity: dead ? 0.5 : 1,
                  }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <span style={{
                        display: 'inline-block', width: '10px', height: '10px', borderRadius: '2px', flexShrink: 0,
                        background: dead ? '#444' : playerColor,
                      }} />
                      <span style={{ color: dead ? '#666' : '#e0e0e0', fontWeight: 600, fontSize: '14px' }}>{p.name}</span>
                      {dead && <span style={{ fontSize: '13px', marginLeft: '2px' }} title="Eliminated">&#x1F480;</span>}
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px' }}>
                      <span style={{ color: dead ? '#555' : '#16c79a', fontSize: '13px' }}>{p.score} pts</span>
                      <span style={{ color: dead ? '#555' : '#888', fontSize: '13px' }}>
                        {dead ? 'Eliminated' : `${p.num_creatures} units`}
                      </span>
                    </div>
                  </div>
                );
              })
          )}
        </div>
      </div>

      {/* Playback controls */}
      <div style={{
        padding: '12px 24px', background: '#16213e', borderTop: '1px solid #333',
        display: 'flex', alignItems: 'center', gap: '16px',
      }}>
        <button onClick={togglePlay} style={btnControl}>
          {playing ? 'Pause' : (currentIndex >= messages.length - 1 ? 'Restart' : 'Play')}
        </button>

        <input
          type="range"
          min={0}
          max={messages.length - 1}
          value={currentIndex}
          onChange={handleScrub}
          style={{ flex: 1, cursor: 'pointer' }}
        />

        <span style={{ color: '#888', fontSize: '12px', minWidth: '80px', textAlign: 'center' }}>
          {currentIndex + 1} / {messages.length}
        </span>

        <div style={{ display: 'flex', gap: '4px' }}>
          {[1, 2, 4].map(s => (
            <button
              key={s}
              onClick={() => setSpeed(s)}
              style={{
                ...btnSpeed,
                background: speed === s ? '#f5a623' : '#0a0a1a',
                color: speed === s ? '#000' : '#888',
              }}
            >
              {s}x
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

const btnControl: React.CSSProperties = {
  background: '#16c79a', color: '#fff', border: 'none', padding: '8px 24px',
  borderRadius: '4px', cursor: 'pointer', fontWeight: 700, fontSize: '13px', minWidth: '80px',
};

const btnSpeed: React.CSSProperties = {
  border: '1px solid #333', borderRadius: '4px', padding: '4px 10px',
  cursor: 'pointer', fontWeight: 600, fontSize: '12px',
};
