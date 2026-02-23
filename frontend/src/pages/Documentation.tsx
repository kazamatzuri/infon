import { useState, useEffect } from 'react';

export function Documentation() {
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch('/api/docs/lua-api')
      .then(r => {
        if (!r.ok) throw new Error('Failed to load documentation');
        return r.text();
      })
      .then(setContent)
      .catch(err => setError(err.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div style={{ padding: 24, color: '#888' }}>Loading documentation...</div>;
  if (error) return <div style={{ padding: 24, color: '#e94560' }}>{error}</div>;

  return (
    <div style={{ padding: 24, maxWidth: 900, margin: '0 auto' }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: 24 }}>Lua API Reference</h2>
      <pre style={{ whiteSpace: 'pre-wrap', wordWrap: 'break-word', color: '#ccc', lineHeight: 1.6, fontSize: 14, background: '#0a0a1a', padding: 24, borderRadius: 8, border: '1px solid #333' }}>
        {content}
      </pre>
    </div>
  );
}
