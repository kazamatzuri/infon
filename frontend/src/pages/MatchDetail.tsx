import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { api } from '../api/client';
import type { MatchDetail as MatchDetailType, ReplayData } from '../api/client';
import { ReplayCanvas } from '../components/ReplayCanvas';

export function MatchDetail() {
  const { id } = useParams<{ id: string }>();
  const matchId = Number(id);
  const [matchData, setMatchData] = useState<MatchDetailType | null>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(true);

  // Replay state
  const [replayData, setReplayData] = useState<ReplayData | null>(null);
  const [replayLoading, setReplayLoading] = useState(false);
  const [replayError, setReplayError] = useState('');
  const [showReplay, setShowReplay] = useState(false);

  useEffect(() => {
    if (!matchId) return;
    setLoading(true);
    api.getMatch(matchId)
      .then(data => { setMatchData(data); setError(''); })
      .catch(e => setError(e.message))
      .finally(() => setLoading(false));
  }, [matchId]);

  const loadReplay = async () => {
    setReplayLoading(true);
    setReplayError('');
    try {
      const data = await api.getReplay(matchId);
      setReplayData(data);
      setShowReplay(true);
    } catch (e: unknown) {
      setReplayError(e instanceof Error ? e.message : 'Failed to load replay');
    } finally {
      setReplayLoading(false);
    }
  };

  if (loading) {
    return <div style={{ padding: 40, color: '#888', textAlign: 'center' }}>Loading match...</div>;
  }

  if (error) {
    return (
      <div style={{ padding: 40, textAlign: 'center' }}>
        <div style={{ color: '#e94560', marginBottom: 16 }}>{error}</div>
        <Link to="/game" style={{ color: '#f5a623' }}>Back to Games</Link>
      </div>
    );
  }

  if (!matchData) return null;

  const { match: m, participants } = matchData;

  if (showReplay && replayData) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ padding: '8px 24px', background: '#16213e', borderBottom: '1px solid #333', display: 'flex', alignItems: 'center', gap: '16px' }}>
          <span style={{ color: '#e0e0e0', fontWeight: 600, fontSize: '14px' }}>
            Replay - Match #{m.id}
          </span>
          <div style={{ flex: 1 }} />
          <button onClick={() => setShowReplay(false)} style={btnBack}>
            Back to Details
          </button>
        </div>
        <div style={{ flex: 1, minHeight: 0 }}>
          <ReplayCanvas messages={replayData.messages} tickCount={replayData.tick_count} />
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: '32px', maxWidth: '700px', margin: '0 auto' }}>
      <Link to="/game" style={{ color: '#f5a623', textDecoration: 'none', fontSize: '13px' }}>
        &larr; Back to Games
      </Link>

      <h2 style={{ color: '#e0e0e0', marginTop: '16px', marginBottom: '8px' }}>
        Match #{m.id}
      </h2>

      <div style={{ background: '#16213e', borderRadius: '8px', padding: '16px', marginBottom: '16px' }}>
        <div style={metaRow}>
          <span style={metaLabel}>Format</span>
          <span style={metaValue}>{m.format}</span>
        </div>
        <div style={metaRow}>
          <span style={metaLabel}>Map</span>
          <span style={metaValue}>{m.map}</span>
        </div>
        <div style={metaRow}>
          <span style={metaLabel}>Status</span>
          <span style={{ ...metaValue, color: m.status === 'finished' ? '#16c79a' : '#f5a623' }}>
            {m.status}
          </span>
        </div>
        <div style={metaRow}>
          <span style={metaLabel}>Created</span>
          <span style={metaValue}>{m.created_at}</span>
        </div>
        {m.finished_at && (
          <div style={metaRow}>
            <span style={metaLabel}>Finished</span>
            <span style={metaValue}>{m.finished_at}</span>
          </div>
        )}
      </div>

      <h3 style={{ color: '#e0e0e0', marginBottom: '12px' }}>Participants</h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '24px' }}>
        {[...participants]
          .sort((a, b) => a.player_slot - b.player_slot)
          .map(p => (
            <div key={p.id} style={{
              background: '#16213e', borderRadius: '8px', padding: '12px 16px',
              borderLeft: p.bot_version_id === m.winner_bot_version_id ? '3px solid #16c79a' : '3px solid #333',
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
                <span style={{ color: '#e0e0e0', fontWeight: 600 }}>
                  {p.bot_name ?? `Bot Version #${p.bot_version_id}`}
                  {p.owner_name && (
                    <span style={{ color: '#888', fontWeight: 400, marginLeft: '8px', fontSize: '13px' }}>
                      by {p.owner_name}
                    </span>
                  )}
                  {p.bot_version_id === m.winner_bot_version_id && (
                    <span style={{ color: '#16c79a', marginLeft: '8px' }}>Winner</span>
                  )}
                </span>
                <span style={{ color: '#16c79a', fontWeight: 600 }}>{p.final_score} pts</span>
              </div>
              <div style={{ display: 'flex', gap: '16px', fontSize: '12px', color: '#888' }}>
                {p.placement != null && <span>#{p.placement}</span>}
                <span>Spawned: {p.creatures_spawned}</span>
                <span>Killed: {p.creatures_killed}</span>
                <span>Lost: {p.creatures_lost}</span>
                {p.elo_before != null && p.elo_after != null && (
                  <span>
                    Elo: {p.elo_before} &rarr; {p.elo_after}
                    <span style={{ color: p.elo_after > p.elo_before ? '#16c79a' : '#e94560', marginLeft: '4px' }}>
                      ({p.elo_after > p.elo_before ? '+' : ''}{p.elo_after - p.elo_before})
                    </span>
                  </span>
                )}
              </div>
            </div>
          ))}
      </div>

      {m.status === 'running' && (
        <div>
          <Link to="/game" style={btnWatchLive}>
            Watch Live
          </Link>
        </div>
      )}

      {(m.status === 'finished' || m.status === 'abandoned') && (
        <div>
          {replayError && (
            <div style={{ color: '#e94560', marginBottom: '8px', fontSize: '13px' }}>{replayError}</div>
          )}
          <button onClick={loadReplay} disabled={replayLoading} style={btnReplay}>
            {replayLoading ? 'Loading Replay...' : 'Watch Replay'}
          </button>
        </div>
      )}
    </div>
  );
}

const metaRow: React.CSSProperties = {
  display: 'flex', justifyContent: 'space-between', padding: '4px 0',
};
const metaLabel: React.CSSProperties = {
  color: '#888', fontSize: '13px',
};
const metaValue: React.CSSProperties = {
  color: '#e0e0e0', fontSize: '13px', fontWeight: 600,
};
const btnReplay: React.CSSProperties = {
  background: '#f5a623', color: '#000', border: 'none', padding: '10px 32px',
  borderRadius: '6px', cursor: 'pointer', fontWeight: 700, fontSize: '14px',
};
const btnWatchLive: React.CSSProperties = {
  display: 'inline-block', background: '#16c79a', color: '#fff', border: 'none',
  padding: '10px 32px', borderRadius: '6px', cursor: 'pointer', fontWeight: 700,
  fontSize: '14px', textDecoration: 'none',
};
const btnBack: React.CSSProperties = {
  background: 'transparent', color: '#f5a623', border: '1px solid #f5a623',
  padding: '6px 16px', borderRadius: '4px', cursor: 'pointer', fontWeight: 600, fontSize: '13px',
};
