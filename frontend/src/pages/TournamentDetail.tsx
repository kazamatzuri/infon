import { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { api } from '../api/client';
import type { Tournament, TournamentEntry, TournamentResult, TournamentStanding, Bot, BotVersion } from '../api/client';

const FORMAT_OPTIONS = [
  { value: 'round_robin', label: 'Round Robin' },
  { value: 'single_elimination', label: 'Single Elimination' },
  { value: 'swiss_3', label: 'Swiss (3 rounds)' },
  { value: 'swiss_5', label: 'Swiss (5 rounds)' },
];

function formatLabel(format: string): string {
  const found = FORMAT_OPTIONS.find(f => f.value === format);
  if (found) return found.label;
  if (format.startsWith('swiss_')) {
    const rounds = format.replace('swiss_', '');
    return `Swiss (${rounds} rounds)`;
  }
  return format;
}

export function TournamentDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();

  const [tournament, setTournament] = useState<Tournament | null>(null);
  const [entries, setEntries] = useState<TournamentEntry[]>([]);
  const [results, setResults] = useState<TournamentResult[]>([]);
  const [standings, setStandings] = useState<TournamentStanding[]>([]);
  const [bots, setBots] = useState<Bot[]>([]);
  const [versions, setVersions] = useState<BotVersion[]>([]);
  const [selectedBotId, setSelectedBotId] = useState<number | ''>('');
  const [selectedVersionId, setSelectedVersionId] = useState<number | ''>('');
  const [slotName, setSlotName] = useState('');
  const [selectedFormat, setSelectedFormat] = useState('round_robin');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const tournamentId = id ? parseInt(id, 10) : 0;

  const loadData = useCallback(async () => {
    if (!tournamentId) return;
    try {
      setLoading(true);
      setError(null);
      const [t, e, b] = await Promise.all([
        api.getTournament(tournamentId),
        api.listEntries(tournamentId),
        api.listBots(),
      ]);
      setTournament(t);
      setEntries(e);
      setBots(b);
      setSelectedFormat(t.format || 'round_robin');

      // Load standings if tournament has been run
      if (t.status === 'finished' || t.status === 'running') {
        try {
          const [r, s] = await Promise.all([
            api.getResults(tournamentId),
            api.getStandings(tournamentId),
          ]);
          setResults(r);
          setStandings(s);
        } catch {
          // Results/standings may not be available yet
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tournament');
    } finally {
      setLoading(false);
    }
  }, [tournamentId]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Load versions when bot is selected
  useEffect(() => {
    if (selectedBotId === '') {
      setVersions([]);
      setSelectedVersionId('');
      return;
    }
    api.listVersions(selectedBotId as number).then(setVersions).catch(() => setVersions([]));
  }, [selectedBotId]);

  const handleAddEntry = async () => {
    if (selectedVersionId === '') return;
    try {
      setError(null);
      await api.addEntry(tournamentId, selectedVersionId as number, slotName || undefined);
      setSelectedBotId('');
      setSelectedVersionId('');
      setSlotName('');
      const e = await api.listEntries(tournamentId);
      setEntries(e);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add entry');
    }
  };

  const handleRemoveEntry = async (entryId: number) => {
    try {
      setError(null);
      await api.removeEntry(tournamentId, entryId);
      setEntries(entries.filter(e => e.id !== entryId));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove entry');
    }
  };

  const handleFormatChange = async (format: string) => {
    setSelectedFormat(format);
    try {
      setError(null);
      const updated = await api.updateTournament(tournamentId, { format });
      setTournament(updated);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update format');
    }
  };

  const handleRun = async () => {
    try {
      setError(null);
      await api.runTournament(tournamentId);
      navigate('/game');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start match');
    }
  };

  if (loading) return <p style={{ padding: '24px', color: '#888' }}>Loading...</p>;
  if (!tournament) return <p style={{ padding: '24px', color: '#888' }}>Tournament not found</p>;

  return (
    <div style={{ padding: '24px', maxWidth: '900px', margin: '0 auto' }}>
      <button onClick={() => navigate('/tournaments')} style={{ ...btnLink, marginBottom: '16px' }}>
        &larr; Back to Tournaments
      </button>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <div>
          <h2 style={{ margin: 0, color: '#e0e0e0' }}>{tournament.name}</h2>
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginTop: '4px' }}>
            <span style={{
              padding: '2px 8px',
              borderRadius: '10px',
              fontSize: '12px',
              fontWeight: 600,
              background: tournament.status === 'finished' ? '#1a3a1a' : tournament.status === 'running' ? '#3a3a1a' : '#1a1a3a',
              color: tournament.status === 'finished' ? '#16c79a' : tournament.status === 'running' ? '#f5a623' : '#6a6aff',
            }}>
              {tournament.status}
            </span>
            <span style={{ color: '#888', fontSize: '12px' }}>
              {formatLabel(tournament.format)}
            </span>
          </div>
        </div>
        {user && (tournament.status === 'pending' || tournament.status === 'created') && entries.length >= 2 && (
          <button onClick={handleRun} style={btnRun}>
            Run Match
          </button>
        )}
      </div>

      {error && (
        <div style={{ padding: '12px', background: '#5c1a1a', border: '1px solid #e94560', borderRadius: '4px', marginBottom: '16px', color: '#ff8a8a' }}>
          {error}
        </div>
      )}

      {/* Format selector - only when tournament is editable */}
      {(tournament.status === 'pending' || tournament.status === 'created') && (
        <div style={{ padding: '16px', background: '#16213e', borderRadius: '8px', marginBottom: '24px' }}>
          <h4 style={{ color: '#aaa', margin: '0 0 12px 0', fontSize: '13px', textTransform: 'uppercase' }}>
            Tournament Format
          </h4>
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            {FORMAT_OPTIONS.map(opt => (
              <button
                key={opt.value}
                onClick={() => handleFormatChange(opt.value)}
                style={{
                  ...btnFormat,
                  background: selectedFormat === opt.value ? '#16c79a' : '#0a0a1a',
                  color: selectedFormat === opt.value ? '#fff' : '#aaa',
                  borderColor: selectedFormat === opt.value ? '#16c79a' : '#333',
                }}
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Entries */}
      <h3 style={{ color: '#aaa', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '12px' }}>
        Entries ({entries.length})
      </h3>

      {entries.length === 0 ? (
        <p style={{ color: '#666', marginBottom: '24px' }}>No entries yet. Add bots below.</p>
      ) : (
        <table style={{ width: '100%', borderCollapse: 'collapse', marginBottom: '24px' }}>
          <thead>
            <tr style={{ borderBottom: '1px solid #333' }}>
              <th style={thStyle}>Bot</th>
              <th style={thStyle}>Version</th>
              <th style={thStyle}>Slot</th>
              <th style={thStyle}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {entries.map(entry => (
              <tr key={entry.id} style={{ borderBottom: '1px solid #222' }}>
                <td style={tdStyle}>{entry.bot_name || `Bot #${entry.bot_version_id}`}</td>
                <td style={tdStyle}>v{entry.version || entry.bot_version_id}</td>
                <td style={{ ...tdStyle, color: '#888' }}>{entry.slot_name || '-'}</td>
                <td style={tdStyle}>
                  {user && (tournament.status === 'pending' || tournament.status === 'created') && (
                    <button onClick={() => handleRemoveEntry(entry.id)} style={btnDanger}>
                      Remove
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {/* Add entry form */}
      {user && (tournament.status === 'pending' || tournament.status === 'created') && (
        <div style={{ padding: '16px', background: '#16213e', borderRadius: '8px', marginBottom: '24px' }}>
          <h4 style={{ color: '#aaa', margin: '0 0 12px 0', fontSize: '13px', textTransform: 'uppercase' }}>
            Add Entry
          </h4>
          <div style={{ display: 'flex', gap: '12px', alignItems: 'center', flexWrap: 'wrap' }}>
            <select
              value={selectedBotId}
              onChange={e => setSelectedBotId(e.target.value ? parseInt(e.target.value, 10) : '')}
              style={selectStyle}
            >
              <option value="">Select bot...</option>
              {bots.map(b => (
                <option key={b.id} value={b.id}>{b.name}</option>
              ))}
            </select>
            <select
              value={selectedVersionId}
              onChange={e => setSelectedVersionId(e.target.value ? parseInt(e.target.value, 10) : '')}
              style={selectStyle}
              disabled={versions.length === 0}
            >
              <option value="">Select version...</option>
              {versions.map(v => (
                <option key={v.id} value={v.id}>v{v.version} - {new Date(v.created_at).toLocaleString()}</option>
              ))}
            </select>
            <input
              value={slotName}
              onChange={e => setSlotName(e.target.value)}
              placeholder="Slot name (optional)"
              style={inputStyle}
            />
            <button onClick={handleAddEntry} disabled={selectedVersionId === ''} style={btnPrimary}>
              Add
            </button>
          </div>
        </div>
      )}

      {/* Standings */}
      {standings.length > 0 && (
        <>
          <h3 style={{ color: '#aaa', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '12px' }}>
            Standings
          </h3>
          <table style={{ width: '100%', borderCollapse: 'collapse', marginBottom: '24px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #333' }}>
                <th style={thStyle}>Rank</th>
                <th style={thStyle}>Bot</th>
                <th style={thStyle}>Score</th>
                <th style={thStyle}>Played</th>
                <th style={thStyle}>W</th>
                <th style={thStyle}>L</th>
              </tr>
            </thead>
            <tbody>
              {standings.map((s, i) => (
                <tr key={s.bot_version_id} style={{ borderBottom: '1px solid #222' }}>
                  <td style={tdStyle}>#{i + 1}</td>
                  <td style={{ ...tdStyle, color: '#16c79a', fontWeight: 600 }}>{s.bot_name}</td>
                  <td style={tdStyle}>{s.total_score}</td>
                  <td style={tdStyle}>{s.matches_played}</td>
                  <td style={{ ...tdStyle, color: '#4caf50' }}>{s.wins}</td>
                  <td style={{ ...tdStyle, color: '#e94560' }}>{s.losses}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}

      {/* Results */}
      {results.length > 0 && (
        <>
          <h3 style={{ color: '#aaa', fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '12px' }}>
            Results
          </h3>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #333' }}>
                <th style={thStyle}>Rank</th>
                <th style={thStyle}>Bot</th>
                <th style={thStyle}>Score</th>
                <th style={thStyle}>Spawned</th>
                <th style={thStyle}>Kills</th>
              </tr>
            </thead>
            <tbody>
              {[...results]
                .sort((a, b) => b.final_score - a.final_score)
                .map((r, i) => (
                  <tr key={r.id} style={{ borderBottom: '1px solid #222' }}>
                    <td style={tdStyle}>#{i + 1}</td>
                    <td style={{ ...tdStyle, color: '#16c79a' }}>Slot {r.player_slot} (v#{r.bot_version_id})</td>
                    <td style={tdStyle}>{r.final_score}</td>
                    <td style={tdStyle}>{r.creatures_spawned}</td>
                    <td style={tdStyle}>{r.creatures_killed}</td>
                  </tr>
                ))}
            </tbody>
          </table>
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

const selectStyle: React.CSSProperties = {
  background: '#0a0a1a',
  color: '#e0e0e0',
  border: '1px solid #333',
  borderRadius: '4px',
  padding: '8px 12px',
  fontSize: '14px',
  minWidth: '180px',
};

const inputStyle: React.CSSProperties = {
  background: '#0a0a1a',
  color: '#e0e0e0',
  border: '1px solid #333',
  borderRadius: '4px',
  padding: '8px 12px',
  fontSize: '14px',
  width: '160px',
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

const btnRun: React.CSSProperties = {
  background: '#e94560',
  color: '#fff',
  border: 'none',
  padding: '10px 28px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontWeight: 700,
  fontSize: '16px',
};

const btnLink: React.CSSProperties = {
  background: 'none',
  border: 'none',
  color: '#16c79a',
  cursor: 'pointer',
  fontSize: '14px',
  padding: 0,
};

const btnFormat: React.CSSProperties = {
  border: '1px solid #333',
  padding: '6px 16px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '13px',
  fontWeight: 600,
};
