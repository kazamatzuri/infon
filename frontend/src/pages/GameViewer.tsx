import { useState } from 'react';
import { GameCanvas } from '../components/GameCanvas';
import { api } from '../api/client';

const WS_URL = 'ws://localhost:3000/ws/game';

export function GameViewer() {
  const [stopping, setStopping] = useState(false);

  const handleStop = async () => {
    setStopping(true);
    try {
      await api.stopGame();
    } catch {
      // Game may already be stopped
    } finally {
      setStopping(false);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Controls bar */}
      <div style={{ padding: '8px 24px', background: '#16213e', borderBottom: '1px solid #333', display: 'flex', alignItems: 'center', gap: '16px' }}>
        <span style={{ color: '#e0e0e0', fontWeight: 600, fontSize: '14px' }}>Live Game</span>
        <div style={{ flex: 1 }} />
        <button onClick={handleStop} disabled={stopping} style={btnStop}>
          {stopping ? 'Stopping...' : 'Stop Game'}
        </button>
      </div>

      {/* Game canvas */}
      <div style={{ flex: 1, minHeight: 0 }}>
        <GameCanvas wsUrl={WS_URL} />
      </div>
    </div>
  );
}

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
