import { useState, useEffect, useCallback } from 'react';
import { useSearchParams, Link } from 'react-router-dom';
import { api } from '../api/client';
import type { Bot, BotVersion, MapInfo, ChallengeResult } from '../api/client';

export function Challenge() {
  const [searchParams] = useSearchParams();
  const opponentVersionIdParam = searchParams.get('opponent');
  const opponentName = searchParams.get('name');
  const opponentVer = searchParams.get('ver');
  const opponentRating = searchParams.get('rating');
  const opponentOwner = searchParams.get('owner');

  // My bots
  const [myBots, setMyBots] = useState<Bot[]>([]);
  const [selectedBotId, setSelectedBotId] = useState<number | null>(null);
  const [myVersions, setMyVersions] = useState<BotVersion[]>([]);
  const [selectedVersionId, setSelectedVersionId] = useState<number | null>(null);

  // Opponent (locked when arriving from leaderboard)
  const [opponentVersionId] = useState<number | null>(
    opponentVersionIdParam ? Number(opponentVersionIdParam) : null
  );

  // Options
  const [maps, setMaps] = useState<MapInfo[]>([]);
  const [selectedMap, setSelectedMap] = useState<string>('');
  const [headless, setHeadless] = useState(false);

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

  // Auto-select first bot and latest version
  useEffect(() => {
    if (myBots.length > 0 && !selectedBotId) {
      setSelectedBotId(myBots[0].id);
    }
  }, [myBots, selectedBotId]);

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
        format: '1v1',
        headless,
        map: selectedMap || undefined,
      });
      setResult(res);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create challenge');
    } finally {
      setSubmitting(false);
    }
  }, [selectedVersionId, opponentVersionId, headless, selectedMap]);

  if (loading) {
    return <div style={{ padding: 24, textAlign: 'center', color: '#888' }}>Loading...</div>;
  }

  const selectedBot = myBots.find(b => b.id === selectedBotId);
  const selectedVersion = myVersions.find(v => v.id === selectedVersionId);

  const canSubmit = !submitting && selectedVersionId && opponentVersionId;

  return (
    <div style={{ maxWidth: 900, margin: '40px auto', padding: 24 }}>
      <h2 style={{ color: '#e0e0e0', marginBottom: 8, textAlign: 'center' }}>1v1 Challenge</h2>
      <p style={{ color: '#666', marginBottom: 32, textAlign: 'center', fontSize: 14 }}>
        Choose your bot and fight!
      </p>

      {error && (
        <div style={{
          padding: 12, background: '#5c1a1a', border: '1px solid #e94560',
          borderRadius: 4, marginBottom: 16, color: '#ff8a8a',
        }}>
          {error}
        </div>
      )}

      {result && (
        <div style={{
          padding: 16, background: '#1a3a1a', border: '1px solid #2a5a2a',
          borderRadius: 8, marginBottom: 24, textAlign: 'center',
        }}>
          <p style={{ fontWeight: 'bold', color: '#4caf50', marginBottom: 8 }}>
            Challenge created!
          </p>
          <p style={{ color: '#aaa', fontSize: 13, marginBottom: 12 }}>
            {result.status === 'queued'
              ? 'Match is queued — replay will be available once it finishes.'
              : result.status === 'running'
                ? 'Game is running live!'
                : `Status: ${result.status ?? 'pending'}`
            }
          </p>
          <div style={{ display: 'flex', gap: 12, justifyContent: 'center' }}>
            {result.status === 'running' && (
              <Link
                to="/game"
                style={{
                  display: 'inline-block', padding: '8px 24px', background: '#16c79a',
                  color: '#fff', borderRadius: 4, textDecoration: 'none',
                  fontSize: 14, fontWeight: 600,
                }}
              >
                Watch Live
              </Link>
            )}
            <Link
              to={`/matches/${result.match_id ?? result.id}`}
              style={{
                display: 'inline-block', padding: '8px 24px',
                background: result.status === 'running' ? 'transparent' : '#16c79a',
                color: result.status === 'running' ? '#16c79a' : '#fff',
                border: result.status === 'running' ? '1px solid #16c79a' : 'none',
                borderRadius: 4, textDecoration: 'none',
                fontSize: 14, fontWeight: 600,
              }}
            >
              View Match
            </Link>
          </div>
        </div>
      )}

      {/* VS Layout */}
      <form onSubmit={handleSubmit}>
        <div style={{
          display: 'flex', alignItems: 'stretch', gap: 0,
          marginBottom: 24,
        }}>
          {/* Your Bot Card */}
          <div style={cardStyle}>
            <div style={cardHeaderStyle}>YOUR BOT</div>
            <div style={{ padding: '16px 20px', flex: 1, display: 'flex', flexDirection: 'column' }}>
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
                  <option value="">-- Select --</option>
                  {myBots.map(b => (
                    <option key={b.id} value={b.id}>{b.name}</option>
                  ))}
                </select>
              </div>
              <div style={{ marginBottom: 16 }}>
                <label style={labelStyle}>Version</label>
                <select
                  value={selectedVersionId ?? ''}
                  onChange={e => setSelectedVersionId(Number(e.target.value) || null)}
                  style={selectStyle}
                  disabled={myVersions.length === 0}
                >
                  <option value="">-- Select --</option>
                  {myVersions.map(v => (
                    <option key={v.id} value={v.id}>
                      v{v.version} (Elo: {v.elo_rating})
                    </option>
                  ))}
                </select>
              </div>
              {/* Selected bot stats */}
              <div style={{ flex: 1 }} />
              {selectedBot && selectedVersion ? (
                <div style={statsBoxStyle}>
                  <div style={botNameStyle}>{selectedBot.name}</div>
                  <div style={versionTagStyle}>v{selectedVersion.version}</div>
                  <div style={statRowStyle}>
                    <span style={{ color: '#888' }}>Elo</span>
                    <span style={{ color: '#e0e0e0', fontWeight: 600 }}>{selectedVersion.elo_rating}</span>
                  </div>
                  <div style={statRowStyle}>
                    <span style={{ color: '#888' }}>Record</span>
                    <span>
                      <span style={{ color: '#4caf50' }}>{selectedVersion.wins}W</span>
                      {' / '}
                      <span style={{ color: '#e94560' }}>{selectedVersion.losses}L</span>
                    </span>
                  </div>
                </div>
              ) : (
                <div style={{ ...statsBoxStyle, color: '#555', textAlign: 'center', padding: 24 }}>
                  Select a bot
                </div>
              )}
            </div>
          </div>

          {/* VS Divider */}
          <div style={{
            display: 'flex', flexDirection: 'column', alignItems: 'center',
            justifyContent: 'center', padding: '0 12px', flexShrink: 0,
          }}>
            <div style={{
              width: 2, flex: 1, background:
                'linear-gradient(to bottom, transparent, #e94560 40%, #e94560 60%, transparent)',
            }} />
            <div style={{
              fontSize: 28, fontWeight: 900, letterSpacing: 2,
              color: '#e94560', textShadow: '0 0 20px rgba(233,69,96,0.5)',
              padding: '16px 0', lineHeight: 1,
            }}>
              VS
            </div>
            <div style={{
              width: 2, flex: 1, background:
                'linear-gradient(to bottom, transparent, #e94560 40%, #e94560 60%, transparent)',
            }} />
          </div>

          {/* Opponent Card */}
          <div style={cardStyle}>
            <div style={{ ...cardHeaderStyle, background: '#3a1a1a', color: '#e94560' }}>OPPONENT</div>
            <div style={{ padding: '16px 20px', flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
              {opponentName ? (
                <div style={statsBoxStyle}>
                  <div style={botNameStyle}>{opponentName}</div>
                  <div style={versionTagStyle}>v{opponentVer}</div>
                  <div style={statRowStyle}>
                    <span style={{ color: '#888' }}>Elo</span>
                    <span style={{ color: '#e0e0e0', fontWeight: 600 }}>{opponentRating}</span>
                  </div>
                  {opponentOwner && (
                    <div style={statRowStyle}>
                      <span style={{ color: '#888' }}>Owner</span>
                      <span style={{ color: '#aaa' }}>{opponentOwner}</span>
                    </div>
                  )}
                </div>
              ) : opponentVersionId ? (
                <div style={{ ...statsBoxStyle, color: '#888', textAlign: 'center', padding: 24 }}>
                  Bot version #{opponentVersionId}
                </div>
              ) : (
                <div style={{ ...statsBoxStyle, color: '#555', textAlign: 'center', padding: 24 }}>
                  No opponent selected
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Options row */}
        <div style={{
          display: 'flex', gap: 16, marginBottom: 24, alignItems: 'flex-end',
          justifyContent: 'center',
        }}>
          <div>
            <label style={labelStyle}>Map</label>
            <select
              value={selectedMap}
              onChange={e => setSelectedMap(e.target.value)}
              style={{ ...selectStyle, width: 200 }}
            >
              <option value="">Default (random)</option>
              {maps.map(m => (
                <option key={m.name} value={m.name}>
                  {m.name} ({m.width}x{m.height})
                </option>
              ))}
            </select>
          </div>
          <label style={{ display: 'flex', alignItems: 'center', gap: 6, color: '#aaa', fontSize: 13, paddingBottom: 2 }}>
            <input
              type="checkbox"
              checked={headless}
              onChange={e => setHeadless(e.target.checked)}
            />
            Headless
          </label>
        </div>

        {/* Submit */}
        <div style={{ textAlign: 'center' }}>
          <button
            type="submit"
            disabled={!canSubmit}
            style={{
              padding: '12px 48px',
              background: canSubmit ? '#e94560' : '#333',
              color: '#fff',
              border: 'none',
              borderRadius: 6,
              fontSize: 16,
              fontWeight: 700,
              cursor: canSubmit ? 'pointer' : 'not-allowed',
              letterSpacing: 1,
              textTransform: 'uppercase',
              transition: 'background 0.15s',
            }}
          >
            {submitting ? 'Launching...' : 'Fight!'}
          </button>
        </div>
      </form>
    </div>
  );
}

const cardStyle: React.CSSProperties = {
  flex: 1,
  background: '#1a1a2e',
  border: '1px solid #333',
  borderRadius: 8,
  display: 'flex',
  flexDirection: 'column',
  minHeight: 280,
};

const cardHeaderStyle: React.CSSProperties = {
  padding: '10px 20px',
  background: '#0a2a3a',
  color: '#16c79a',
  fontWeight: 700,
  fontSize: 12,
  letterSpacing: 2,
  textTransform: 'uppercase',
  borderRadius: '8px 8px 0 0',
  textAlign: 'center',
};

const statsBoxStyle: React.CSSProperties = {
  background: '#0f0f23',
  border: '1px solid #2a2a4a',
  borderRadius: 6,
  padding: '16px',
};

const botNameStyle: React.CSSProperties = {
  color: '#16c79a',
  fontWeight: 700,
  fontSize: 18,
  marginBottom: 4,
};

const versionTagStyle: React.CSSProperties = {
  color: '#888',
  fontSize: 12,
  marginBottom: 12,
};

const statRowStyle: React.CSSProperties = {
  display: 'flex',
  justifyContent: 'space-between',
  fontSize: 13,
  padding: '4px 0',
  borderTop: '1px solid #1a1a2e',
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: '#aaa',
  fontSize: 12,
  marginBottom: 4,
  textTransform: 'uppercase',
  letterSpacing: 0.5,
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
