import { useEffect, useState, useCallback } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { api } from '../api/client';
import type { ActiveGameInfo } from '../api/client';

interface RecentMatch {
  id: number;
  format: string;
  map: string;
  status: string;
  winner_bot_version_id: number | null;
  created_at: string;
  finished_at: string | null;
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffSec = Math.floor((now - then) / 1000);
  if (diffSec < 60) return 'just now';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  return `${Math.floor(diffSec / 86400)}d ago`;
}

export function GameList() {
  const navigate = useNavigate();
  const [activeGames, setActiveGames] = useState<ActiveGameInfo[]>([]);
  const [recentMatches, setRecentMatches] = useState<RecentMatch[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    try {
      setError(null);
      const [active, recent] = await Promise.all([
        api.listActiveGames(),
        api.listMatches(20),
      ]);
      setActiveGames(active);
      setRecentMatches(recent);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load games');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, [loadData]);

  return (
    <div style={{ padding: '24px', maxWidth: '960px', margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '24px' }}>
        <h2 style={{ color: '#e0e0e0', margin: 0 }}>Games</h2>
        <Link
          to="/game"
          style={{
            padding: '8px 20px',
            background: '#16c79a',
            color: '#fff',
            borderRadius: '4px',
            textDecoration: 'none',
            fontWeight: 600,
            fontSize: '14px',
          }}
        >
          New Game
        </Link>
      </div>

      {error && (
        <div style={{
          padding: '12px',
          background: '#5c1a1a',
          border: '1px solid #e94560',
          borderRadius: '4px',
          marginBottom: '16px',
          color: '#ff8a8a',
        }}>
          {error}
        </div>
      )}

      {/* Active Games */}
      <h3 style={{ color: '#16c79a', marginBottom: '12px', fontSize: '16px' }}>
        Live Games {activeGames.length > 0 && (
          <span style={{
            background: '#16c79a',
            color: '#0a0a1a',
            borderRadius: '10px',
            padding: '2px 8px',
            fontSize: '12px',
            fontWeight: 700,
            marginLeft: '8px',
          }}>
            {activeGames.length}
          </span>
        )}
      </h3>

      {activeGames.length === 0 ? (
        <div style={{
          padding: '32px',
          background: '#16213e',
          borderRadius: '8px',
          textAlign: 'center',
          color: '#666',
          marginBottom: '32px',
        }}>
          No games currently running
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '32px' }}>
          {activeGames.map((game, i) => (
            <div
              key={game.match_id ?? i}
              style={{
                padding: '16px 20px',
                background: '#16213e',
                borderRadius: '8px',
                border: '1px solid #16c79a33',
                display: 'flex',
                alignItems: 'center',
                gap: '16px',
              }}
            >
              {/* Live indicator */}
              <div style={{
                width: '10px',
                height: '10px',
                borderRadius: '50%',
                background: '#16c79a',
                boxShadow: '0 0 6px #16c79a',
                flexShrink: 0,
              }} />

              {/* Players */}
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ color: '#e0e0e0', fontWeight: 600, fontSize: '15px', marginBottom: '4px' }}>
                  {game.player_names.join(' vs ')}
                </div>
                <div style={{ color: '#888', fontSize: '13px', display: 'flex', gap: '16px', flexWrap: 'wrap' }}>
                  <span>{game.format.toUpperCase()}</span>
                  <span>Map: {game.map}</span>
                  <span>Duration: {formatDuration(game.game_time_seconds)}</span>
                  <span style={{ color: '#f5a623' }}>
                    {game.spectator_count} {game.spectator_count === 1 ? 'spectator' : 'spectators'}
                  </span>
                </div>
              </div>

              {/* Watch button */}
              <button
                onClick={() => navigate('/game')}
                style={{
                  padding: '8px 20px',
                  borderRadius: '4px',
                  border: 'none',
                  cursor: 'pointer',
                  fontWeight: 600,
                  fontSize: '14px',
                  background: '#16c79a',
                  color: '#fff',
                  flexShrink: 0,
                }}
              >
                Watch
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Recent Matches */}
      <h3 style={{ color: '#e0e0e0', marginBottom: '12px', fontSize: '16px' }}>Recent Matches</h3>

      {loading && recentMatches.length === 0 ? (
        <div style={{ color: '#888', padding: '16px' }}>Loading...</div>
      ) : recentMatches.length === 0 ? (
        <div style={{
          padding: '32px',
          background: '#16213e',
          borderRadius: '8px',
          textAlign: 'center',
          color: '#666',
        }}>
          No matches yet
        </div>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333' }}>
              <th style={{ textAlign: 'left', padding: '8px 12px', color: '#888', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase' }}>ID</th>
              <th style={{ textAlign: 'left', padding: '8px 12px', color: '#888', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase' }}>Format</th>
              <th style={{ textAlign: 'left', padding: '8px 12px', color: '#888', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase' }}>Map</th>
              <th style={{ textAlign: 'left', padding: '8px 12px', color: '#888', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase' }}>Status</th>
              <th style={{ textAlign: 'left', padding: '8px 12px', color: '#888', fontSize: '12px', fontWeight: 600, textTransform: 'uppercase' }}>When</th>
              <th style={{ padding: '8px 12px' }}></th>
            </tr>
          </thead>
          <tbody>
            {recentMatches.map(m => (
              <tr
                key={m.id}
                style={{
                  borderBottom: '1px solid #1a1a2e',
                  cursor: 'pointer',
                }}
                onClick={() => navigate(`/matches/${m.id}`)}
                onMouseOver={e => (e.currentTarget.style.background = '#1a1a2e')}
                onMouseOut={e => (e.currentTarget.style.background = 'transparent')}
              >
                <td style={{ padding: '10px 12px', color: '#aaa', fontSize: '14px' }}>#{m.id}</td>
                <td style={{ padding: '10px 12px', color: '#e0e0e0', fontSize: '14px' }}>{m.format.toUpperCase()}</td>
                <td style={{ padding: '10px 12px', color: '#aaa', fontSize: '14px' }}>{m.map}</td>
                <td style={{ padding: '10px 12px', fontSize: '14px' }}>
                  <span style={{
                    color: m.status === 'finished' ? '#16c79a' : m.status === 'pending' ? '#f5a623' : '#e94560',
                    fontWeight: 500,
                  }}>
                    {m.status}
                  </span>
                </td>
                <td style={{ padding: '10px 12px', color: '#888', fontSize: '13px' }}>
                  {timeAgo(m.finished_at ?? m.created_at)}
                </td>
                <td style={{ padding: '10px 12px', textAlign: 'right' }}>
                  <button
                    onClick={e => {
                      e.stopPropagation();
                      navigate(`/matches/${m.id}`);
                    }}
                    style={{
                      padding: '4px 12px',
                      borderRadius: '4px',
                      border: '1px solid #333',
                      cursor: 'pointer',
                      fontSize: '12px',
                      background: 'transparent',
                      color: '#aaa',
                    }}
                  >
                    Details
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
