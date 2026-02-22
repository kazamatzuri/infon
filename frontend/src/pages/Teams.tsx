import { useState, useEffect, useCallback } from 'react';
import { useAuth } from '../context/AuthContext';
import { Navigate } from 'react-router-dom';
import { api } from '../api/client';
import type { Team, TeamVersion, Bot, BotVersion } from '../api/client';

export function Teams() {
  const { user } = useAuth();
  const [teams, setTeams] = useState<Team[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  // Create team form
  const [newTeamName, setNewTeamName] = useState('');
  const [creating, setCreating] = useState(false);

  // Selected team detail
  const [selectedTeam, setSelectedTeam] = useState<Team | null>(null);
  const [teamVersions, setTeamVersions] = useState<TeamVersion[]>([]);
  const [loadingVersions, setLoadingVersions] = useState(false);

  // Create team version form
  const [bots, setBots] = useState<Bot[]>([]);
  const [botVersionsA, setBotVersionsA] = useState<BotVersion[]>([]);
  const [botVersionsB, setBotVersionsB] = useState<BotVersion[]>([]);
  const [selectedBotA, setSelectedBotA] = useState<number | ''>('');
  const [selectedBotB, setSelectedBotB] = useState<number | ''>('');
  const [selectedVersionA, setSelectedVersionA] = useState<number | ''>('');
  const [selectedVersionB, setSelectedVersionB] = useState<number | ''>('');
  const [creatingVersion, setCreatingVersion] = useState(false);

  const fetchTeams = useCallback(async () => {
    try {
      const data = await api.listTeams();
      setTeams(data);
    } catch {
      setError('Failed to load teams');
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchBots = useCallback(async () => {
    try {
      const data = await api.listBots();
      setBots(data);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    if (user) {
      fetchTeams();
      fetchBots();
    }
  }, [user, fetchTeams, fetchBots]);

  const loadTeamDetail = async (team: Team) => {
    setSelectedTeam(team);
    setLoadingVersions(true);
    try {
      const versions = await api.listTeamVersions(team.id);
      setTeamVersions(versions);
    } catch {
      setError('Failed to load team versions');
    } finally {
      setLoadingVersions(false);
    }
  };

  const handleCreateTeam = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTeamName.trim()) return;
    setCreating(true);
    setError('');
    try {
      const team = await api.createTeam(newTeamName.trim());
      setTeams(prev => [...prev, team]);
      setNewTeamName('');
    } catch {
      setError('Failed to create team');
    } finally {
      setCreating(false);
    }
  };

  const handleDeleteTeam = async (id: number) => {
    if (!confirm('Are you sure you want to delete this team?')) return;
    try {
      await api.deleteTeam(id);
      setTeams(prev => prev.filter(t => t.id !== id));
      if (selectedTeam?.id === id) {
        setSelectedTeam(null);
        setTeamVersions([]);
      }
    } catch {
      setError('Failed to delete team');
    }
  };

  // Load bot versions when a bot is selected for team version creation
  useEffect(() => {
    if (selectedBotA !== '') {
      api.listVersions(selectedBotA as number).then(setBotVersionsA).catch(() => {});
      setSelectedVersionA('');
    } else {
      setBotVersionsA([]);
      setSelectedVersionA('');
    }
  }, [selectedBotA]);

  useEffect(() => {
    if (selectedBotB !== '') {
      api.listVersions(selectedBotB as number).then(setBotVersionsB).catch(() => {});
      setSelectedVersionB('');
    } else {
      setBotVersionsB([]);
      setSelectedVersionB('');
    }
  }, [selectedBotB]);

  const handleCreateTeamVersion = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedTeam || selectedVersionA === '' || selectedVersionB === '') return;
    setCreatingVersion(true);
    setError('');
    try {
      const version = await api.createTeamVersion(
        selectedTeam.id,
        selectedVersionA as number,
        selectedVersionB as number,
      );
      setTeamVersions(prev => [...prev, version]);
      setSelectedBotA('');
      setSelectedBotB('');
      setSelectedVersionA('');
      setSelectedVersionB('');
    } catch {
      setError('Failed to create team version');
    } finally {
      setCreatingVersion(false);
    }
  };

  if (!user) {
    return <Navigate to="/login" />;
  }

  return (
    <div style={{ maxWidth: 800, margin: '40px auto', padding: 24 }}>
      <h2>Teams</h2>
      <p style={{ color: '#aaa', marginBottom: 24 }}>
        Create teams of two bots for 2v2 matches.
      </p>

      {error && <p style={{ color: '#f44' }}>{error}</p>}

      {/* Create team form */}
      <form onSubmit={handleCreateTeam} style={{ display: 'flex', gap: 8, marginBottom: 32 }}>
        <input
          type="text"
          value={newTeamName}
          onChange={e => setNewTeamName(e.target.value)}
          placeholder="Team name"
          required
          style={{ flex: 1, padding: 8 }}
        />
        <button type="submit" disabled={creating} style={{ padding: '8px 24px' }}>
          {creating ? 'Creating...' : 'Create Team'}
        </button>
      </form>

      <div style={{ display: 'flex', gap: 24 }}>
        {/* Team list */}
        <div style={{ flex: 1 }}>
          <h3>Your Teams</h3>
          {loading ? (
            <p>Loading...</p>
          ) : teams.length === 0 ? (
            <p style={{ color: '#888' }}>No teams yet.</p>
          ) : (
            <ul style={{ listStyle: 'none', padding: 0 }}>
              {teams.map(team => (
                <li
                  key={team.id}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '8px 12px',
                    marginBottom: 4,
                    background: selectedTeam?.id === team.id ? '#1a3a4a' : '#1a1a2e',
                    borderRadius: 4,
                    cursor: 'pointer',
                    border: selectedTeam?.id === team.id ? '1px solid #3a6a8a' : '1px solid #333',
                  }}
                >
                  <span
                    onClick={() => loadTeamDetail(team)}
                    style={{ flex: 1 }}
                  >
                    {team.name}
                  </span>
                  <button
                    onClick={(e) => { e.stopPropagation(); handleDeleteTeam(team.id); }}
                    style={{ padding: '2px 8px', fontSize: 12, color: '#f44', cursor: 'pointer' }}
                  >
                    Delete
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Team detail */}
        {selectedTeam && (
          <div style={{ flex: 2 }}>
            <h3>{selectedTeam.name}</h3>
            <p style={{ fontSize: 12, color: '#888' }}>
              Created: {new Date(selectedTeam.created_at + 'Z').toLocaleDateString()}
            </p>

            {/* Team versions */}
            <h4>Team Versions</h4>
            {loadingVersions ? (
              <p>Loading versions...</p>
            ) : teamVersions.length === 0 ? (
              <p style={{ color: '#888' }}>No versions yet. Create one below.</p>
            ) : (
              <table style={{ width: '100%', borderCollapse: 'collapse', marginBottom: 16 }}>
                <thead>
                  <tr style={{ borderBottom: '1px solid #333' }}>
                    <th style={{ textAlign: 'left', padding: 6 }}>Ver</th>
                    <th style={{ textAlign: 'left', padding: 6 }}>Bot Version A</th>
                    <th style={{ textAlign: 'left', padding: 6 }}>Bot Version B</th>
                    <th style={{ textAlign: 'left', padding: 6 }}>Elo</th>
                    <th style={{ textAlign: 'left', padding: 6 }}>W/L/D</th>
                  </tr>
                </thead>
                <tbody>
                  {teamVersions.map(tv => (
                    <tr key={tv.id} style={{ borderBottom: '1px solid #222' }}>
                      <td style={{ padding: 6 }}>v{tv.version}</td>
                      <td style={{ padding: 6, fontSize: 13 }}>#{tv.bot_version_a}</td>
                      <td style={{ padding: 6, fontSize: 13 }}>#{tv.bot_version_b}</td>
                      <td style={{ padding: 6 }}>{tv.elo_rating}</td>
                      <td style={{ padding: 6, fontSize: 13 }}>
                        {tv.wins}/{tv.losses}/{tv.draws}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}

            {/* Create team version form */}
            <h4>Create Team Version</h4>
            <form onSubmit={handleCreateTeamVersion}>
              <div style={{ display: 'flex', gap: 12, marginBottom: 12 }}>
                <div style={{ flex: 1 }}>
                  <label style={{ display: 'block', marginBottom: 4, fontSize: 13 }}>Bot A</label>
                  <select
                    value={selectedBotA}
                    onChange={e => setSelectedBotA(e.target.value ? Number(e.target.value) : '')}
                    style={{ width: '100%', padding: 6 }}
                  >
                    <option value="">Select bot...</option>
                    {bots.map(b => (
                      <option key={b.id} value={b.id}>{b.name}</option>
                    ))}
                  </select>
                  {botVersionsA.length > 0 && (
                    <select
                      value={selectedVersionA}
                      onChange={e => setSelectedVersionA(e.target.value ? Number(e.target.value) : '')}
                      style={{ width: '100%', padding: 6, marginTop: 4 }}
                    >
                      <option value="">Select version...</option>
                      {botVersionsA.map(v => (
                        <option key={v.id} value={v.id}>v{v.version} (Elo: {v.elo_rating})</option>
                      ))}
                    </select>
                  )}
                </div>
                <div style={{ flex: 1 }}>
                  <label style={{ display: 'block', marginBottom: 4, fontSize: 13 }}>Bot B</label>
                  <select
                    value={selectedBotB}
                    onChange={e => setSelectedBotB(e.target.value ? Number(e.target.value) : '')}
                    style={{ width: '100%', padding: 6 }}
                  >
                    <option value="">Select bot...</option>
                    {bots.map(b => (
                      <option key={b.id} value={b.id}>{b.name}</option>
                    ))}
                  </select>
                  {botVersionsB.length > 0 && (
                    <select
                      value={selectedVersionB}
                      onChange={e => setSelectedVersionB(e.target.value ? Number(e.target.value) : '')}
                      style={{ width: '100%', padding: 6, marginTop: 4 }}
                    >
                      <option value="">Select version...</option>
                      {botVersionsB.map(v => (
                        <option key={v.id} value={v.id}>v{v.version} (Elo: {v.elo_rating})</option>
                      ))}
                    </select>
                  )}
                </div>
              </div>
              <button
                type="submit"
                disabled={creatingVersion || selectedVersionA === '' || selectedVersionB === ''}
                style={{ padding: '8px 24px' }}
              >
                {creatingVersion ? 'Creating...' : 'Create Team Version'}
              </button>
            </form>
          </div>
        )}
      </div>
    </div>
  );
}
