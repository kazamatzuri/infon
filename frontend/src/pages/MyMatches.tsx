import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { api } from '../api/client';
import type { MatchDetail } from '../api/client';

type Match = MatchDetail['match'];

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const seconds = Math.floor((now - then) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function statusColor(status: string): string {
  switch (status) {
    case 'finished': return '#4caf50';
    case 'pending':
    case 'queued': return '#ff9800';
    case 'running': return '#2196f3';
    default: return '#aaa';
  }
}

export function MyMatches() {
  const [matches, setMatches] = useState<Match[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const limit = 50;

  useEffect(() => {
    api.listMyMatches(limit, 0)
      .then(data => {
        setMatches(data);
        setHasMore(data.length === limit);
      })
      .catch(err => setError(err instanceof Error ? err.message : 'Failed to load matches'))
      .finally(() => setLoading(false));
  }, []);

  const loadMore = async () => {
    const newOffset = offset + limit;
    setLoadingMore(true);
    try {
      const data = await api.listMyMatches(limit, newOffset);
      setMatches(prev => [...prev, ...data]);
      setOffset(newOffset);
      setHasMore(data.length === limit);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more matches');
    } finally {
      setLoadingMore(false);
    }
  };

  if (loading) {
    return <div style={{ padding: 24, textAlign: 'center', color: '#888' }}>Loading...</div>;
  }

  return (
    <div style={{ maxWidth: 900, margin: '40px auto', padding: 24 }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: 24 }}>My Matches</h2>

      {error && (
        <div style={{ padding: 12, background: '#5c1a1a', border: '1px solid #e94560', borderRadius: 4, marginBottom: 16, color: '#ff8a8a' }}>
          {error}
        </div>
      )}

      {matches.length === 0 ? (
        <p style={{ color: '#888' }}>No matches yet. <Link to="/challenge" style={{ color: '#16c79a' }}>Challenge someone!</Link></p>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333', color: '#aaa', fontSize: 13 }}>
              <th style={thStyle}>Match</th>
              <th style={thStyle}>Format</th>
              <th style={thStyle}>Map</th>
              <th style={thStyle}>Status</th>
              <th style={thStyle}>Date</th>
              <th style={thStyle}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {matches.map(m => (
              <tr key={m.id} style={{ borderBottom: '1px solid #222' }}>
                <td style={tdStyle}>
                  <Link to={`/matches/${m.id}`} style={{ color: '#16c79a' }}>#{m.id}</Link>
                </td>
                <td style={tdStyle}>{m.format}</td>
                <td style={tdStyle}>{m.map}</td>
                <td style={tdStyle}>
                  <span style={{ color: statusColor(m.status), fontWeight: 600 }}>{m.status}</span>
                </td>
                <td style={{ ...tdStyle, color: '#888' }}>{timeAgo(m.created_at)}</td>
                <td style={tdStyle}>
                  {m.status === 'finished' ? (
                    <Link to={`/matches/${m.id}`} style={{ color: '#16c79a', fontSize: 13 }}>Watch Replay</Link>
                  ) : (
                    <span style={{ color: '#555', fontSize: 13 }}>—</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {hasMore && matches.length > 0 && (
        <div style={{ textAlign: 'center', marginTop: 20 }}>
          <button
            onClick={loadMore}
            disabled={loadingMore}
            style={{
              padding: '8px 24px',
              background: loadingMore ? '#333' : '#1a1a2e',
              color: '#e0e0e0',
              border: '1px solid #333',
              borderRadius: 4,
              cursor: loadingMore ? 'not-allowed' : 'pointer',
              fontSize: 14,
            }}
          >
            {loadingMore ? 'Loading...' : 'Load More'}
          </button>
        </div>
      )}
    </div>
  );
}

const thStyle: React.CSSProperties = {
  textAlign: 'left',
  padding: '8px 12px',
  fontWeight: 600,
};

const tdStyle: React.CSSProperties = {
  padding: '8px 12px',
  color: '#e0e0e0',
  fontSize: 14,
};
