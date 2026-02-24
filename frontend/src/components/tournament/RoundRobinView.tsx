import type { TournamentRound, TournamentMatchInfo } from '../../api/client';
import { MatchCard } from './MatchCard';

interface Props {
  rounds: TournamentRound[];
}

interface BotEntry {
  id: number;
  name: string;
}

export function RoundRobinView({ rounds }: Props) {
  // Flatten all matches
  const allMatches = rounds.flatMap(r => r.matches);

  // Collect unique bots
  const botMap = new Map<number, string>();
  for (const m of allMatches) {
    for (const p of m.participants) {
      if (!botMap.has(p.bot_version_id)) {
        botMap.set(p.bot_version_id, p.bot_name || `Bot #${p.bot_version_id}`);
      }
    }
  }
  const bots: BotEntry[] = Array.from(botMap.entries()).map(([id, name]) => ({ id, name }));

  // Build results lookup: key = "rowId-colId" => match
  const resultMap = new Map<string, TournamentMatchInfo>();
  for (const m of allMatches) {
    if (m.participants.length >= 2) {
      const a = m.participants[0].bot_version_id;
      const b = m.participants[1].bot_version_id;
      resultMap.set(`${a}-${b}`, m);
      resultMap.set(`${b}-${a}`, m);
    }
  }

  function getCellContent(rowBot: number, colBot: number): { text: string; color: string } | null {
    if (rowBot === colBot) return null;
    const m = resultMap.get(`${rowBot}-${colBot}`);
    if (!m || m.status !== 'finished') return { text: '-', color: '#666' };

    const rowP = m.participants.find(p => p.bot_version_id === rowBot);
    const colP = m.participants.find(p => p.bot_version_id === colBot);
    if (!rowP || !colP) return { text: '-', color: '#666' };

    const won = m.winner_bot_version_id === rowBot;
    const lost = m.winner_bot_version_id === colBot;
    return {
      text: `${rowP.final_score}-${colP.final_score}`,
      color: won ? '#4caf50' : lost ? '#e94560' : '#aaa',
    };
  }

  return (
    <div>
      {/* NxN Matrix */}
      <div style={{ overflowX: 'auto', marginBottom: '24px' }}>
        <table style={{ borderCollapse: 'collapse', fontSize: '13px' }}>
          <thead>
            <tr>
              <th style={{ padding: '8px', color: '#aaa' }} />
              {bots.map(b => (
                <th key={b.id} style={{
                  padding: '8px',
                  color: '#e0e0e0',
                  fontWeight: 600,
                  fontSize: '12px',
                  maxWidth: '100px',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}>
                  {b.name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {bots.map(rowBot => (
              <tr key={rowBot.id}>
                <td style={{
                  padding: '8px',
                  color: '#e0e0e0',
                  fontWeight: 600,
                  fontSize: '12px',
                  whiteSpace: 'nowrap',
                }}>
                  {rowBot.name}
                </td>
                {bots.map(colBot => {
                  if (rowBot.id === colBot.id) {
                    return (
                      <td key={colBot.id} style={{ padding: '8px', background: '#0a0a1a', textAlign: 'center' }} />
                    );
                  }
                  const cell = getCellContent(rowBot.id, colBot.id);
                  return (
                    <td key={colBot.id} style={{
                      padding: '8px',
                      textAlign: 'center',
                      color: cell?.color || '#666',
                      fontWeight: 600,
                      border: '1px solid #222',
                    }}>
                      {cell?.text || '-'}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* All match cards */}
      <h4 style={{ color: '#aaa', fontSize: '12px', textTransform: 'uppercase', marginBottom: '12px' }}>
        All Matches
      </h4>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
        {allMatches.map(m => (
          <MatchCard key={m.match_id} match={m} />
        ))}
      </div>
    </div>
  );
}
