import { useEffect, useState, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import type { Bot } from '../api/client';

type SortKey = 'name' | 'updated_at' | 'version_count' | 'latest_version' | 'latest_elo_1v1';
type SortDir = 'asc' | 'desc';

export function BotLibrary() {
  const [bots, setBots] = useState<Bot[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [sortKey, setSortKey] = useState<SortKey>('updated_at');
  const [sortDir, setSortDir] = useState<SortDir>('desc');
  const [confirmingDelete, setConfirmingDelete] = useState<number | null>(null);
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

  const handleDelete = async (id: number) => {
    if (confirmingDelete !== id) {
      setConfirmingDelete(id);
      return;
    }
    setConfirmingDelete(null);
    try {
      await api.deleteBot(id);
      setBots(bots.filter(b => b.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete bot');
    }
  };

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(d => (d === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortKey(key);
      setSortDir(key === 'name' ? 'asc' : 'desc');
    }
  };

  const sortedBots = useMemo(() => {
    const filtered = search
      ? bots.filter(b => b.name.toLowerCase().includes(search.toLowerCase()))
      : bots;

    return [...filtered].sort((a, b) => {
      const dir = sortDir === 'asc' ? 1 : -1;
      switch (sortKey) {
        case 'name':
          return dir * a.name.localeCompare(b.name);
        case 'updated_at':
          return dir * a.updated_at.localeCompare(b.updated_at);
        case 'version_count':
          return dir * ((a.version_count ?? 0) - (b.version_count ?? 0));
        case 'latest_version':
          return dir * ((a.latest_version ?? 0) - (b.latest_version ?? 0));
        case 'latest_elo_1v1':
          return dir * ((a.latest_elo_1v1 ?? 0) - (b.latest_elo_1v1 ?? 0));
        default:
          return 0;
      }
    });
  }, [bots, search, sortKey, sortDir]);

  const sortIndicator = (key: SortKey) => {
    if (sortKey !== key) return '';
    return sortDir === 'asc' ? ' ▲' : ' ▼';
  };

  return (
    <div style={{ padding: '24px', maxWidth: '1000px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
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

      {!loading && bots.length > 0 && (
        <div style={{ marginBottom: '16px' }}>
          <input
            type="text"
            placeholder="Search bots by name..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            style={searchStyle}
          />
        </div>
      )}

      {loading ? (
        <p style={{ color: '#888' }}>Loading bots...</p>
      ) : bots.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '48px', color: '#888' }}>
          <p>No bots yet. Create your first bot to get started!</p>
        </div>
      ) : sortedBots.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '32px', color: '#888' }}>
          <p>No bots match "{search}"</p>
        </div>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333' }}>
              <th style={thClickable} onClick={() => handleSort('name')}>
                Name{sortIndicator('name')}
              </th>
              <th style={thStyle}>Description</th>
              <th style={thClickable} onClick={() => handleSort('version_count')}>
                Versions{sortIndicator('version_count')}
              </th>
              <th style={thClickable} onClick={() => handleSort('latest_version')}>
                Latest{sortIndicator('latest_version')}
              </th>
              <th style={{ ...thClickable, textAlign: 'right' }} onClick={() => handleSort('latest_elo_1v1')}>
                Elo{sortIndicator('latest_elo_1v1')}
              </th>
              <th style={thClickable} onClick={() => handleSort('updated_at')}>
                Updated{sortIndicator('updated_at')}
              </th>
              <th style={thStyle}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {sortedBots.map(bot => (
              <tr key={bot.id} style={{ borderBottom: '1px solid #222' }}>
                <td style={tdStyle}>
                  <a
                    onClick={() => navigate(`/editor/${bot.id}`)}
                    style={{ color: '#16c79a', cursor: 'pointer', textDecoration: 'none' }}
                  >
                    {bot.name}
                  </a>
                </td>
                <td style={{ ...tdStyle, color: '#888', maxWidth: '200px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                  {bot.description || '-'}
                </td>
                <td style={{ ...tdStyle, textAlign: 'center', fontVariantNumeric: 'tabular-nums' }}>
                  {bot.version_count ?? 0}
                </td>
                <td style={{ ...tdStyle, textAlign: 'center', color: '#888', fontVariantNumeric: 'tabular-nums' }}>
                  {bot.latest_version != null ? `v${bot.latest_version}` : '-'}
                </td>
                <td style={{ ...tdStyle, textAlign: 'right', fontWeight: 600, fontVariantNumeric: 'tabular-nums' }}>
                  {bot.latest_elo_1v1 != null ? bot.latest_elo_1v1 : '-'}
                </td>
                <td style={{ ...tdStyle, color: '#888', fontSize: '13px' }}>
                  {new Date(bot.updated_at).toLocaleDateString()}
                </td>
                <td style={tdStyle}>
                  {confirmingDelete === bot.id ? (
                    <span style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
                      <button onClick={() => handleDelete(bot.id)} style={btnDangerConfirm}>
                        Confirm
                      </button>
                      <button onClick={() => setConfirmingDelete(null)} style={btnCancel}>
                        Cancel
                      </button>
                    </span>
                  ) : (
                    <button onClick={() => handleDelete(bot.id)} style={btnDanger}>
                      Delete
                    </button>
                  )}
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

const thClickable: React.CSSProperties = {
  ...thStyle,
  cursor: 'pointer',
  userSelect: 'none',
};

const tdStyle: React.CSSProperties = {
  padding: '10px 12px',
  color: '#e0e0e0',
};

const searchStyle: React.CSSProperties = {
  width: '100%',
  padding: '8px 12px',
  background: '#16213e',
  border: '1px solid #333',
  borderRadius: '4px',
  color: '#e0e0e0',
  fontSize: '14px',
  outline: 'none',
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

const btnDangerConfirm: React.CSSProperties = {
  background: '#e94560',
  color: '#fff',
  border: 'none',
  padding: '4px 12px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '13px',
  fontWeight: 600,
};

const btnCancel: React.CSSProperties = {
  background: 'transparent',
  color: '#888',
  border: '1px solid #555',
  padding: '4px 10px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '12px',
};
