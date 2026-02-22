import { useEffect, useState, useCallback } from 'react';
import { api } from '../api/client';
import type { LeaderboardEntry } from '../api/client';

type Tab = '1v1' | 'ffa' | '2v2';

const PAGE_SIZE = 50;

export function Leaderboard() {
  const [tab, setTab] = useState<Tab>('1v1');
  const [entries, setEntries] = useState<LeaderboardEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [offset, setOffset] = useState(0);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      let data: LeaderboardEntry[];
      switch (tab) {
        case '1v1':
          data = await api.leaderboard1v1(PAGE_SIZE, offset);
          break;
        case 'ffa':
          data = await api.leaderboardFfa(PAGE_SIZE, offset);
          break;
        case '2v2':
          data = await api.leaderboard2v2(PAGE_SIZE, offset);
          break;
      }
      setEntries(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load leaderboard');
    } finally {
      setLoading(false);
    }
  }, [tab, offset]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const switchTab = (t: Tab) => {
    setTab(t);
    setOffset(0);
  };

  return (
    <div style={{ padding: '24px', maxWidth: '960px', margin: '0 auto' }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: '24px' }}>Leaderboards</h2>

      {/* Tabs */}
      <div style={{ display: 'flex', gap: '4px', marginBottom: '24px' }}>
        {([['1v1', '1v1 Elo'], ['ffa', 'FFA'], ['2v2', '2v2 Teams']] as const).map(([key, label]) => (
          <button
            key={key}
            onClick={() => switchTab(key)}
            style={{
              padding: '8px 20px',
              borderRadius: '4px',
              border: 'none',
              cursor: 'pointer',
              fontWeight: 600,
              fontSize: '14px',
              background: tab === key ? '#16c79a' : '#1a1a2e',
              color: tab === key ? '#fff' : '#888',
            }}
          >
            {label}
          </button>
        ))}
      </div>

      {error && (
        <div style={{ padding: '12px', background: '#5c1a1a', border: '1px solid #e94560', borderRadius: '4px', marginBottom: '16px', color: '#ff8a8a' }}>
          {error}
        </div>
      )}

      {loading ? (
        <p style={{ color: '#888' }}>Loading...</p>
      ) : entries.length === 0 ? (
        <p style={{ color: '#888', textAlign: 'center', padding: '48px' }}>
          {tab === '2v2' ? 'Team leaderboards coming soon!' : 'No entries yet. Play some games to populate the leaderboard!'}
        </p>
      ) : (
        <>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #333' }}>
                <th style={thStyle}>#</th>
                <th style={thStyle}>Bot</th>
                <th style={thStyle}>Version</th>
                <th style={thStyle}>Owner</th>
                <th style={{ ...thStyle, textAlign: 'right' }}>Rating</th>
                <th style={{ ...thStyle, textAlign: 'right' }}>Games</th>
                <th style={{ ...thStyle, textAlign: 'right' }}>W / L</th>
                <th style={{ ...thStyle, textAlign: 'right' }}>Win Rate</th>
              </tr>
            </thead>
            <tbody>
              {entries.map(e => (
                <tr key={e.bot_version_id} style={{ borderBottom: '1px solid #222' }}>
                  <td style={{ ...tdStyle, color: '#888', fontWeight: 600 }}>{e.rank}</td>
                  <td style={{ ...tdStyle, color: '#16c79a', fontWeight: 600 }}>{e.bot_name}</td>
                  <td style={{ ...tdStyle, color: '#888' }}>v{e.version}</td>
                  <td style={tdStyle}>{e.owner_username}</td>
                  <td style={{ ...tdStyle, textAlign: 'right', fontWeight: 600, fontVariantNumeric: 'tabular-nums' }}>
                    {e.rating}
                  </td>
                  <td style={{ ...tdStyle, textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                    {e.games_played}
                  </td>
                  <td style={{ ...tdStyle, textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                    <span style={{ color: '#4caf50' }}>{e.wins}</span>
                    {' / '}
                    <span style={{ color: '#e94560' }}>{e.losses}</span>
                  </td>
                  <td style={{ ...tdStyle, textAlign: 'right', fontVariantNumeric: 'tabular-nums' }}>
                    {(e.win_rate * 100).toFixed(1)}%
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {/* Pagination */}
          <div style={{ display: 'flex', justifyContent: 'center', gap: '12px', marginTop: '20px' }}>
            <button
              onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
              disabled={offset === 0}
              style={paginationBtn}
            >
              Previous
            </button>
            <span style={{ color: '#888', fontSize: '14px', lineHeight: '36px' }}>
              {offset + 1} - {offset + entries.length}
            </span>
            <button
              onClick={() => setOffset(offset + PAGE_SIZE)}
              disabled={entries.length < PAGE_SIZE}
              style={paginationBtn}
            >
              Next
            </button>
          </div>
        </>
      )}
    </div>
  );
}

const thStyle: React.CSSProperties = {
  textAlign: 'left',
  padding: '10px 12px',
  color: '#aaa',
  fontSize: '13px',
  fontWeight: 600,
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
};

const tdStyle: React.CSSProperties = {
  padding: '10px 12px',
  color: '#e0e0e0',
};

const paginationBtn: React.CSSProperties = {
  background: '#1a1a2e',
  color: '#e0e0e0',
  border: '1px solid #333',
  padding: '6px 16px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '14px',
};
