import type { TournamentRound, TournamentStanding } from '../../api/client';
import { MatchCard } from './MatchCard';

interface Props {
  rounds: TournamentRound[];
  standings: TournamentStanding[];
}

export function SwissRoundsView({ rounds, standings }: Props) {
  return (
    <div>
      {rounds.map(round => (
        <div key={round.round} style={{ marginBottom: '24px' }}>
          <h4 style={{
            color: '#aaa',
            fontSize: '12px',
            textTransform: 'uppercase',
            letterSpacing: '0.5px',
            margin: '0 0 12px 0',
          }}>
            Round {round.round}
          </h4>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '16px' }}>
            {round.matches.map(m => (
              <MatchCard key={m.match_id} match={m} />
            ))}
          </div>
        </div>
      ))}

      {/* Final standings */}
      {standings.length > 0 && (
        <div style={{ marginTop: '16px' }}>
          <h4 style={{ color: '#aaa', fontSize: '12px', textTransform: 'uppercase', marginBottom: '8px' }}>
            Final Standings
          </h4>
          <table style={{ borderCollapse: 'collapse', width: '100%', fontSize: '13px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid #333' }}>
                <th style={thStyle}>#</th>
                <th style={thStyle}>Bot</th>
                <th style={thStyle}>Score</th>
                <th style={thStyle}>W</th>
                <th style={thStyle}>L</th>
              </tr>
            </thead>
            <tbody>
              {standings.map((s, i) => (
                <tr key={s.bot_version_id} style={{ borderBottom: '1px solid #222' }}>
                  <td style={tdStyle}>{i + 1}</td>
                  <td style={{ ...tdStyle, color: '#16c79a', fontWeight: 600 }}>{s.bot_name}</td>
                  <td style={tdStyle}>{s.total_score}</td>
                  <td style={{ ...tdStyle, color: '#4caf50' }}>{s.wins}</td>
                  <td style={{ ...tdStyle, color: '#e94560' }}>{s.losses}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

const thStyle: React.CSSProperties = {
  textAlign: 'left',
  padding: '8px 10px',
  color: '#aaa',
  fontSize: '12px',
  fontWeight: 600,
  textTransform: 'uppercase',
};

const tdStyle: React.CSSProperties = {
  padding: '8px 10px',
  color: '#e0e0e0',
};
