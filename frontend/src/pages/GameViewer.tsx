import { useState, useEffect } from 'react';
import { GameCanvas } from '../components/GameCanvas';
import { api } from '../api/client';
import type { Bot, BotVersion } from '../api/client';

const WS_URL = `ws://${window.location.host}/ws/game`;

interface PlayerSlot {
  botId: number | null;
  versionId: number | null;
  name: string;
}

export function GameViewer() {
  const [phase, setPhase] = useState<'loading' | 'setup' | 'running'>('loading');
  const [bots, setBots] = useState<Bot[]>([]);
  const [versions, setVersions] = useState<Record<number, BotVersion[]>>({});
  const [slots, setSlots] = useState<PlayerSlot[]>([
    { botId: null, versionId: null, name: 'Player 1' },
    { botId: null, versionId: null, name: 'Player 2' },
  ]);
  const [error, setError] = useState('');
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);

  // Check game status and load bots on mount
  useEffect(() => {
    (async () => {
      try {
        const [status, botList] = await Promise.all([
          api.gameStatus(),
          api.listBots(),
        ]);
        setBots(botList);
        if (status.running) {
          setPhase('running');
        } else {
          setPhase('setup');
        }
      } catch {
        setPhase('setup');
      }
    })();
  }, []);

  // Load versions when a bot is selected
  const loadVersions = async (botId: number) => {
    if (versions[botId]) return;
    try {
      const v = await api.listVersions(botId);
      setVersions(prev => ({ ...prev, [botId]: v }));
    } catch {
      // ignore
    }
  };

  const updateSlot = (index: number, updates: Partial<PlayerSlot>) => {
    setSlots(prev => prev.map((s, i) => i === index ? { ...s, ...updates } : s));
  };

  const handleBotSelect = async (index: number, botId: number) => {
    const bot = bots.find(b => b.id === botId);
    updateSlot(index, { botId, versionId: null, name: bot?.name || `Player ${index + 1}` });
    await loadVersions(botId);
    // Auto-select latest version
    const v = versions[botId];
    if (v && v.length > 0) {
      updateSlot(index, { botId, versionId: v[v.length - 1].id });
    }
  };

  // Re-select latest version once versions load
  useEffect(() => {
    setSlots(prev => prev.map(s => {
      if (s.botId && !s.versionId && versions[s.botId]?.length) {
        return { ...s, versionId: versions[s.botId][versions[s.botId].length - 1].id };
      }
      return s;
    }));
  }, [versions]);

  const addSlot = () => {
    setSlots(prev => [...prev, { botId: null, versionId: null, name: `Player ${prev.length + 1}` }]);
  };

  const removeSlot = (index: number) => {
    if (slots.length <= 2) return;
    setSlots(prev => prev.filter((_, i) => i !== index));
  };

  const handleStart = async () => {
    setError('');
    const players = slots
      .filter(s => s.versionId !== null)
      .map(s => ({ bot_version_id: s.versionId!, name: s.name || undefined }));

    if (players.length < 2) {
      setError('Select at least 2 bots to start a game.');
      return;
    }

    setStarting(true);
    try {
      await api.startGame(players);
      setPhase('running');
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Failed to start game');
    } finally {
      setStarting(false);
    }
  };

  const handleStop = async () => {
    setStopping(true);
    try {
      await api.stopGame();
      // Give server a moment to clean up
      setTimeout(() => {
        setPhase('setup');
        setStopping(false);
      }, 500);
    } catch {
      setStopping(false);
    }
  };

  if (phase === 'loading') {
    return <div style={{ padding: 40, color: '#888', textAlign: 'center' }}>Loading...</div>;
  }

  if (phase === 'running') {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ padding: '8px 24px', background: '#16213e', borderBottom: '1px solid #333', display: 'flex', alignItems: 'center', gap: '16px' }}>
          <span style={{ color: '#e0e0e0', fontWeight: 600, fontSize: '14px' }}>Live Game</span>
          <div style={{ flex: 1 }} />
          <button onClick={handleStop} disabled={stopping} style={btnStop}>
            {stopping ? 'Stopping...' : 'Stop Game'}
          </button>
        </div>
        <div style={{ flex: 1, minHeight: 0 }}>
          <GameCanvas wsUrl={WS_URL} />
        </div>
      </div>
    );
  }

  // Setup phase
  return (
    <div style={{ padding: '32px', maxWidth: '700px', margin: '0 auto' }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: '8px' }}>Start a Game</h2>
      <p style={{ color: '#888', fontSize: '14px', marginBottom: '24px' }}>
        Select bots from your library to compete against each other.
      </p>

      {error && (
        <div style={{ background: '#e9456020', border: '1px solid #e94560', borderRadius: '6px', padding: '10px 16px', marginBottom: '16px', color: '#e94560', fontSize: '13px' }}>
          {error}
        </div>
      )}

      {bots.length === 0 ? (
        <div style={{ background: '#16213e', borderRadius: '8px', padding: '32px', textAlign: 'center' }}>
          <p style={{ color: '#888', marginBottom: '8px' }}>No bots in your library yet.</p>
          <a href="/editor" style={{ color: '#f5a623' }}>Create a bot first</a>
        </div>
      ) : (
        <>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '20px' }}>
            {slots.map((slot, i) => (
              <div key={i} style={{ background: '#16213e', borderRadius: '8px', padding: '12px 16px', display: 'flex', alignItems: 'center', gap: '12px' }}>
                <input
                  type="text"
                  value={slot.name}
                  onChange={e => updateSlot(i, { name: e.target.value })}
                  style={{ ...inputStyle, width: '120px', flexShrink: 0 }}
                  placeholder="Name"
                />

                <select
                  value={slot.botId ?? ''}
                  onChange={e => {
                    const id = Number(e.target.value);
                    if (id) handleBotSelect(i, id);
                  }}
                  style={{ ...inputStyle, flex: 1 }}
                >
                  <option value="">-- Select Bot --</option>
                  {bots.map(b => (
                    <option key={b.id} value={b.id}>{b.name}</option>
                  ))}
                </select>

                {slot.botId && versions[slot.botId] && (
                  <select
                    value={slot.versionId ?? ''}
                    onChange={e => updateSlot(i, { versionId: Number(e.target.value) })}
                    style={{ ...inputStyle, width: '100px' }}
                  >
                    <option value="">Version</option>
                    {versions[slot.botId].map(v => (
                      <option key={v.id} value={v.id}>v{v.version}</option>
                    ))}
                  </select>
                )}

                {slots.length > 2 && (
                  <button onClick={() => removeSlot(i)} style={btnRemove} title="Remove player">
                    X
                  </button>
                )}
              </div>
            ))}
          </div>

          <div style={{ display: 'flex', gap: '12px' }}>
            <button onClick={addSlot} style={btnSecondary}>
              + Add Player
            </button>
            <div style={{ flex: 1 }} />
            <button onClick={handleStart} disabled={starting} style={btnStart}>
              {starting ? 'Starting...' : 'Start Game'}
            </button>
          </div>
        </>
      )}
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: '#0a0a1a',
  border: '1px solid #333',
  borderRadius: '4px',
  color: '#e0e0e0',
  padding: '6px 10px',
  fontSize: '13px',
};

const btnStart: React.CSSProperties = {
  background: '#16c79a',
  color: '#fff',
  border: 'none',
  padding: '10px 32px',
  borderRadius: '6px',
  cursor: 'pointer',
  fontWeight: 700,
  fontSize: '14px',
};

const btnStop: React.CSSProperties = {
  background: '#e94560',
  color: '#fff',
  border: 'none',
  padding: '6px 20px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontWeight: 600,
  fontSize: '13px',
};

const btnSecondary: React.CSSProperties = {
  background: 'transparent',
  color: '#f5a623',
  border: '1px solid #f5a623',
  padding: '8px 20px',
  borderRadius: '6px',
  cursor: 'pointer',
  fontWeight: 600,
  fontSize: '13px',
};

const btnRemove: React.CSSProperties = {
  background: 'transparent',
  color: '#e94560',
  border: '1px solid #e94560',
  borderRadius: '4px',
  padding: '4px 8px',
  cursor: 'pointer',
  fontSize: '12px',
  fontWeight: 700,
};
