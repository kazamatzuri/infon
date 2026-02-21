import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import type { Bot } from '../api/client';

export function BotLibrary() {
  const [bots, setBots] = useState<Bot[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  const loadBots = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.listBots();
      setBots(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load bots');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadBots();
  }, []);

  const handleCreate = async () => {
    try {
      const bot = await api.createBot('New Bot', 'A new Infon bot');
      navigate(`/editor/${bot.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create bot');
    }
  };

  const handleDelete = async (id: number, name: string) => {
    if (!confirm(`Delete bot "${name}"? This cannot be undone.`)) return;
    try {
      await api.deleteBot(id);
      setBots(bots.filter(b => b.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete bot');
    }
  };

  return (
    <div style={{ padding: '24px', maxWidth: '900px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h2 style={{ margin: 0, color: '#e0e0e0' }}>Bot Library</h2>
        <button onClick={handleCreate} style={btnPrimary}>
          + New Bot
        </button>
      </div>

      {error && (
        <div style={{ padding: '12px', background: '#5c1a1a', border: '1px solid #e94560', borderRadius: '4px', marginBottom: '16px', color: '#ff8a8a' }}>
          {error}
        </div>
      )}

      {loading ? (
        <p style={{ color: '#888' }}>Loading bots...</p>
      ) : bots.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '48px', color: '#888' }}>
          <p>No bots yet. Create your first bot to get started!</p>
        </div>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333' }}>
              <th style={thStyle}>Name</th>
              <th style={thStyle}>Description</th>
              <th style={thStyle}>Updated</th>
              <th style={thStyle}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {bots.map(bot => (
              <tr key={bot.id} style={{ borderBottom: '1px solid #222' }}>
                <td style={tdStyle}>
                  <a
                    onClick={() => navigate(`/editor/${bot.id}`)}
                    style={{ color: '#16c79a', cursor: 'pointer', textDecoration: 'none' }}
                  >
                    {bot.name}
                  </a>
                </td>
                <td style={{ ...tdStyle, color: '#888' }}>{bot.description || '-'}</td>
                <td style={{ ...tdStyle, color: '#888', fontSize: '13px' }}>
                  {new Date(bot.updated_at).toLocaleDateString()}
                </td>
                <td style={tdStyle}>
                  <button
                    onClick={() => handleDelete(bot.id, bot.name)}
                    style={btnDanger}
                  >
                    Delete
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

const btnDanger: React.CSSProperties = {
  background: 'transparent',
  color: '#e94560',
  border: '1px solid #e94560',
  padding: '4px 12px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '13px',
};
