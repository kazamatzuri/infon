import { useState, useEffect, useCallback } from 'react';
import { useAuth } from '../context/AuthContext';
import { Navigate } from 'react-router-dom';

interface ApiKey {
  id: number;
  name: string;
  scopes: string;
  last_used_at: string | null;
  created_at: string;
}

const AVAILABLE_SCOPES = [
  'bots:read',
  'bots:write',
  'matches:read',
  'matches:write',
  'leaderboard:read',
];

export function ApiKeys() {
  const { user, token } = useAuth();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  // Create form
  const [newName, setNewName] = useState('');
  const [selectedScopes, setSelectedScopes] = useState<string[]>([
    'bots:read',
    'matches:read',
    'leaderboard:read',
  ]);
  const [creating, setCreating] = useState(false);

  // Newly created token (shown once)
  const [newToken, setNewToken] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const fetchKeys = useCallback(async () => {
    if (!token) return;
    try {
      const res = await fetch('/api/api-keys', {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (res.ok) {
        setKeys(await res.json());
      }
    } catch {
      setError('Failed to load API keys');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    fetchKeys();
  }, [fetchKeys]);

  if (!user) {
    return <Navigate to="/login" />;
  }

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    setCreating(true);
    setError('');
    setNewToken(null);

    try {
      const res = await fetch('/api/api-keys', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          name: newName.trim(),
          scopes: selectedScopes.join(','),
        }),
      });
      const data = await res.json();
      if (!res.ok) {
        setError(data.error || 'Failed to create key');
        return;
      }
      setNewToken(data.token);
      setNewName('');
      setCopied(false);
      fetchKeys();
    } catch {
      setError('Network error');
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm('Are you sure you want to revoke this API key?')) return;
    try {
      const res = await fetch(`/api/api-keys/${id}`, {
        method: 'DELETE',
        headers: { Authorization: `Bearer ${token}` },
      });
      if (res.ok || res.status === 204) {
        setKeys(keys.filter(k => k.id !== id));
      }
    } catch {
      setError('Failed to delete key');
    }
  };

  const handleCopy = async () => {
    if (newToken) {
      await navigator.clipboard.writeText(newToken);
      setCopied(true);
    }
  };

  const toggleScope = (scope: string) => {
    setSelectedScopes(prev =>
      prev.includes(scope)
        ? prev.filter(s => s !== scope)
        : [...prev, scope]
    );
  };

  return (
    <div style={{ maxWidth: 700, margin: '40px auto', padding: 24 }}>
      <h2>API Keys</h2>
      <p style={{ color: '#aaa', marginBottom: 24 }}>
        Create API keys for programmatic access to the Infon API.
      </p>

      {error && <p style={{ color: '#f44' }}>{error}</p>}

      {/* New token display */}
      {newToken && (
        <div
          style={{
            background: '#1a3a1a',
            border: '1px solid #2a5a2a',
            borderRadius: 8,
            padding: 16,
            marginBottom: 24,
          }}
        >
          <p style={{ fontWeight: 'bold', marginBottom: 8 }}>
            API key created! Copy it now -- you won't be able to see it again.
          </p>
          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <code
              style={{
                flex: 1,
                background: '#111',
                padding: 8,
                borderRadius: 4,
                wordBreak: 'break-all',
                fontSize: 13,
              }}
            >
              {newToken}
            </code>
            <button onClick={handleCopy} style={{ padding: '8px 16px', whiteSpace: 'nowrap' }}>
              {copied ? 'Copied!' : 'Copy'}
            </button>
          </div>
        </div>
      )}

      {/* Create form */}
      <form onSubmit={handleCreate} style={{ marginBottom: 32 }}>
        <h3>Create New Key</h3>
        <div style={{ marginBottom: 12 }}>
          <label>Name</label>
          <input
            type="text"
            value={newName}
            onChange={e => setNewName(e.target.value)}
            placeholder="e.g. My Bot Script"
            required
            style={{ display: 'block', width: '100%', padding: 8, marginTop: 4 }}
          />
        </div>
        <div style={{ marginBottom: 12 }}>
          <label>Scopes</label>
          <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', marginTop: 4 }}>
            {AVAILABLE_SCOPES.map(scope => (
              <label key={scope} style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                <input
                  type="checkbox"
                  checked={selectedScopes.includes(scope)}
                  onChange={() => toggleScope(scope)}
                />
                {scope}
              </label>
            ))}
          </div>
        </div>
        <button type="submit" disabled={creating} style={{ padding: '8px 24px' }}>
          {creating ? 'Creating...' : 'Create API Key'}
        </button>
      </form>

      {/* Keys list */}
      <h3>Your API Keys</h3>
      {loading ? (
        <p>Loading...</p>
      ) : keys.length === 0 ? (
        <p style={{ color: '#888' }}>No API keys yet.</p>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333' }}>
              <th style={{ textAlign: 'left', padding: 8 }}>Name</th>
              <th style={{ textAlign: 'left', padding: 8 }}>Scopes</th>
              <th style={{ textAlign: 'left', padding: 8 }}>Last Used</th>
              <th style={{ textAlign: 'left', padding: 8 }}>Created</th>
              <th style={{ padding: 8 }}></th>
            </tr>
          </thead>
          <tbody>
            {keys.map(k => (
              <tr key={k.id} style={{ borderBottom: '1px solid #222' }}>
                <td style={{ padding: 8 }}>{k.name}</td>
                <td style={{ padding: 8, fontSize: 12, color: '#aaa' }}>{k.scopes}</td>
                <td style={{ padding: 8, fontSize: 12, color: '#aaa' }}>
                  {k.last_used_at ? new Date(k.last_used_at + 'Z').toLocaleDateString() : 'Never'}
                </td>
                <td style={{ padding: 8, fontSize: 12, color: '#aaa' }}>
                  {new Date(k.created_at + 'Z').toLocaleDateString()}
                </td>
                <td style={{ padding: 8 }}>
                  <button
                    onClick={() => handleDelete(k.id)}
                    style={{ padding: '4px 12px', fontSize: 12, color: '#f44', cursor: 'pointer' }}
                  >
                    Revoke
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
