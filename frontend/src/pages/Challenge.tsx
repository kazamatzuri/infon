import { useState, useEffect, useCallback } from 'react';
import { useSearchParams } from 'react-router-dom';
import { api } from '../api/client';
import type { Bot, BotVersion, MapInfo, ChallengeResult } from '../api/client';

export function Challenge() {
  const [searchParams] = useSearchParams();
  const prefilledOpponent = searchParams.get('opponent');

  // My bots
  const [myBots, setMyBots] = useState<Bot[]>([]);
  const [selectedBotId, setSelectedBotId] = useState<number | null>(null);
  const [myVersions, setMyVersions] = useState<BotVersion[]>([]);
  const [selectedVersionId, setSelectedVersionId] = useState<number | null>(null);

  // Opponent
  const [allBots, setAllBots] = useState<Bot[]>([]);
  const [opponentBotId, setOpponentBotId] = useState<number | null>(null);
  const [opponentVersions, setOpponentVersions] = useState<BotVersion[]>([]);
  const [opponentVersionId, setOpponentVersionId] = useState<number | null>(prefilledOpponent ? Number(prefilledOpponent) : null);

  // Options
  const [format, setFormat] = useState<string>('1v1');
  const [headless, setHeadless] = useState(false);
  const [maps, setMaps] = useState<MapInfo[]>([]);
  const [selectedMap, setSelectedMap] = useState<string>('');

  // State
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ChallengeResult | null>(null);

  // Load initial data
  useEffect(() => {
    async function load() {
      try {
        const [bots, mapList] = await Promise.all([
          api.listBots(),
          api.listMaps(),
        ]);
        setMyBots(bots);
        setAllBots(bots);
        setMaps(mapList);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load data');
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  // Load my bot versions when bot selected
  useEffect(() => {
    if (!selectedBotId) {
      setMyVersions([]);
      setSelectedVersionId(null);
      return;
    }
    api.listVersions(selectedBotId).then(setMyVersions).catch(() => setMyVersions([]));
  }, [selectedBotId]);

  // Load opponent versions when opponent bot selected
  useEffect(() => {
    if (!opponentBotId) {
      setOpponentVersions([]);
      if (!prefilledOpponent) setOpponentVersionId(null);
      return;
    }
    api.listVersions(opponentBotId).then(versions => {
      setOpponentVersions(versions);
      // If prefilled opponent version, keep it; otherwise select latest
      if (prefilledOpponent && versions.some(v => v.id === Number(prefilledOpponent))) {
        setOpponentVersionId(Number(prefilledOpponent));
      }
    }).catch(() => setOpponentVersions([]));
  }, [opponentBotId, prefilledOpponent]);

  // Auto-select first bot
  useEffect(() => {
    if (myBots.length > 0 && !selectedBotId) {
      setSelectedBotId(myBots[0].id);
    }
  }, [myBots, selectedBotId]);

  // Auto-select latest version
  useEffect(() => {
    if (myVersions.length > 0 && !selectedVersionId) {
      setSelectedVersionId(myVersions[myVersions.length - 1].id);
    }
  }, [myVersions, selectedVersionId]);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedVersionId || !opponentVersionId) return;

    setSubmitting(true);
    setError(null);
    setResult(null);

    try {
      const res = await api.createChallenge(selectedVersionId, opponentVersionId, {
        format,
        headless,
        map: selectedMap || undefined,
      });
      setResult(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create challenge');
    } finally {
      setSubmitting(false);
    }
  }, [selectedVersionId, opponentVersionId, format, headless, selectedMap]);

  if (loading) {
    return (
      <div style={{ padding: 24, textAlign: 'center', color: '#888' }}>Loading...</div>
    );
  }

  return (
    <div style={{ maxWidth: 640, margin: '40px auto', padding: 24 }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: 24 }}>Challenge</h2>
      <p style={{ color: '#aaa', marginBottom: 24 }}>
        Challenge another bot to a match.
      </p>

      {error && (
        <div style={{
          padding: 12,
          background: '#5c1a1a',
          border: '1px solid #e94560',
          borderRadius: 4,
          marginBottom: 16,
          color: '#ff8a8a',
        }}>
          {error}
        </div>
      )}

      {result && (
        <div style={{
          padding: 16,
          background: '#1a3a1a',
          border: '1px solid #2a5a2a',
          borderRadius: 8,
          marginBottom: 24,
        }}>
          <p style={{ fontWeight: 'bold', color: '#4caf50', marginBottom: 8 }}>
            Challenge created!
          </p>
          <p style={{ color: '#e0e0e0', fontSize: 14 }}>
            Match ID: <strong>{result.match_id ?? result.id ?? 'N/A'}</strong>
          </p>
          <p style={{ color: '#aaa', fontSize: 13 }}>
            Status: {result.status ?? 'pending'}
          </p>
        </div>
      )}

      <form onSubmit={handleSubmit}>
        {/* My Bot */}
        <fieldset style={fieldsetStyle}>
          <legend style={legendStyle}>Your Bot</legend>
          <div style={{ marginBottom: 12 }}>
            <label style={labelStyle}>Bot</label>
            <select
              value={selectedBotId ?? ''}
              onChange={e => {
                setSelectedBotId(Number(e.target.value) || null);
                setSelectedVersionId(null);
              }}
              style={selectStyle}
            >
              <option value="">-- Select a bot --</option>
              {myBots.map(b => (
                <option key={b.id} value={b.id}>{b.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label style={labelStyle}>Version</label>
            <select
              value={selectedVersionId ?? ''}
              onChange={e => setSelectedVersionId(Number(e.target.value) || null)}
              style={selectStyle}
              disabled={myVersions.length === 0}
            >
              <option value="">-- Select version --</option>
              {myVersions.map(v => (
                <option key={v.id} value={v.id}>
                  v{v.version} (Elo: {v.elo_rating})
                </option>
              ))}
            </select>
          </div>
        </fieldset>

        {/* Opponent */}
        <fieldset style={fieldsetStyle}>
          <legend style={legendStyle}>Opponent</legend>
          <div style={{ marginBottom: 12 }}>
            <label style={labelStyle}>Opponent Bot</label>
            <select
              value={opponentBotId ?? ''}
              onChange={e => {
                setOpponentBotId(Number(e.target.value) || null);
                setOpponentVersionId(null);
              }}
              style={selectStyle}
            >
              <option value="">-- Select opponent bot --</option>
              {allBots.map(b => (
                <option key={b.id} value={b.id}>{b.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label style={labelStyle}>Version</label>
            <select
              value={opponentVersionId ?? ''}
              onChange={e => setOpponentVersionId(Number(e.target.value) || null)}
              style={selectStyle}
              disabled={opponentVersions.length === 0}
            >
              <option value="">
                {prefilledOpponent && !opponentBotId
                  ? `Version ID: ${prefilledOpponent} (select bot to browse)`
                  : '-- Select version --'}
              </option>
              {opponentVersions.map(v => (
                <option key={v.id} value={v.id}>
                  v{v.version} (Elo: {v.elo_rating})
                </option>
              ))}
            </select>
          </div>
          {prefilledOpponent && (
            <p style={{ color: '#888', fontSize: 12, marginTop: 8 }}>
              Pre-selected opponent version ID: {prefilledOpponent}
            </p>
          )}
        </fieldset>

        {/* Options */}
        <fieldset style={fieldsetStyle}>
          <legend style={legendStyle}>Options</legend>
          <div style={{ marginBottom: 12 }}>
            <label style={labelStyle}>Format</label>
            <select
              value={format}
              onChange={e => setFormat(e.target.value)}
              style={selectStyle}
            >
              <option value="1v1">1v1</option>
              <option value="ffa">FFA (Free For All)</option>
            </select>
          </div>
          <div style={{ marginBottom: 12 }}>
            <label style={labelStyle}>Map</label>
            <select
              value={selectedMap}
              onChange={e => setSelectedMap(e.target.value)}
              style={selectStyle}
            >
              <option value="">Default</option>
              {maps.map(m => (
                <option key={m.name} value={m.name}>
                  {m.name} ({m.width}x{m.height})
                </option>
              ))}
            </select>
          </div>
          <div>
            <label style={{ ...labelStyle, display: 'flex', alignItems: 'center', gap: 8 }}>
              <input
                type="checkbox"
                checked={headless}
                onChange={e => setHeadless(e.target.checked)}
              />
              Headless (faster, no live view)
            </label>
          </div>
        </fieldset>

        <button
          type="submit"
          disabled={submitting || !selectedVersionId || !opponentVersionId}
          style={{
            padding: '10px 32px',
            background: submitting || !selectedVersionId || !opponentVersionId ? '#333' : '#16c79a',
            color: '#fff',
            border: 'none',
            borderRadius: 4,
            fontSize: 15,
            fontWeight: 600,
            cursor: submitting || !selectedVersionId || !opponentVersionId ? 'not-allowed' : 'pointer',
            marginTop: 8,
          }}
        >
          {submitting ? 'Submitting...' : 'Send Challenge'}
        </button>
      </form>
    </div>
  );
}

const fieldsetStyle: React.CSSProperties = {
  border: '1px solid #333',
  borderRadius: 8,
  padding: 16,
  marginBottom: 20,
  background: '#1a1a2e',
};

const legendStyle: React.CSSProperties = {
  color: '#16c79a',
  fontWeight: 600,
  fontSize: 14,
  padding: '0 8px',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: '#aaa',
  fontSize: 13,
  marginBottom: 4,
};

const selectStyle: React.CSSProperties = {
  width: '100%',
  padding: 8,
  background: '#0f0f23',
  color: '#e0e0e0',
  border: '1px solid #333',
  borderRadius: 4,
  fontSize: 14,
};
