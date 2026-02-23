import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { api } from '../api/client';
import type { Tournament } from '../api/client';

export function TournamentList() {
  const [tournaments, setTournaments] = useState<Tournament[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState('');
  const navigate = useNavigate();
  const { user } = useAuth();

  const loadTournaments = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.listTournaments();
      setTournaments(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tournaments');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTournaments();
  }, []);

  const handleCreate = async () => {
    if (!newName.trim()) return;
    try {
      setError(null);
      const t = await api.createTournament(newName.trim());
      setNewName('');
      navigate(`/tournaments/${t.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create tournament');
    }
  };

  return (
    <div style={{ padding: '24px', maxWidth: '900px', margin: '0 auto' }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: '24px' }}>Tournaments</h2>

      {error && (
        <div style={{ padding: '12px', background: '#5c1a1a', border: '1px solid #e94560', borderRadius: '4px', marginBottom: '16px', color: '#ff8a8a' }}>
          {error}
        </div>
      )}

      {/* Create form */}
      {user && (
        <div style={{ display: 'flex', gap: '12px', marginBottom: '24px' }}>
          <input
            value={newName}
            onChange={e => setNewName(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleCreate()}
            placeholder="Tournament name"
            style={inputStyle}
          />
          <button onClick={handleCreate} disabled={!newName.trim()} style={btnPrimary}>
            + New Tournament
          </button>
        </div>
      )}

      {loading ? (
        <p style={{ color: '#888' }}>Loading...</p>
      ) : tournaments.length === 0 ? (
        <p style={{ color: '#888', textAlign: 'center', padding: '48px' }}>
          No tournaments yet. Create one to get started!
        </p>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333' }}>
              <th style={thStyle}>Name</th>
              <th style={thStyle}>Status</th>
              <th style={thStyle}>Created</th>
            </tr>
          </thead>
          <tbody>
            {tournaments.map(t => (
              <tr key={t.id} style={{ borderBottom: '1px solid #222', cursor: 'pointer' }} onClick={() => navigate(`/tournaments/${t.id}`)}>
                <td style={{ ...tdStyle, color: '#16c79a' }}>{t.name}</td>
                <td style={tdStyle}>
                  <span style={{
                    padding: '2px 8px',
                    borderRadius: '10px',
                    fontSize: '12px',
                    fontWeight: 600,
                    background: t.status === 'finished' ? '#1a3a1a' : t.status === 'running' ? '#3a3a1a' : '#1a1a3a',
                    color: t.status === 'finished' ? '#16c79a' : t.status === 'running' ? '#f5a623' : '#6a6aff',
                  }}>
                    {t.status}
                  </span>
                </td>
                <td style={{ ...tdStyle, color: '#888', fontSize: '13px' }}>
                  {new Date(t.created_at).toLocaleDateString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
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

const inputStyle: React.CSSProperties = {
  background: '#0a0a1a',
  color: '#e0e0e0',
  border: '1px solid #333',
  borderRadius: '4px',
  padding: '8px 14px',
  fontSize: '14px',
  flex: 1,
  maxWidth: '400px',
};

const btnPrimary: React.CSSProperties = {
  background: '#16c79a',
  color: '#fff',
  border: 'none',
  padding: '8px 20px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontWeight: 600,
  fontSize: '14px',
};
