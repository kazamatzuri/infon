import type { TournamentRound, TournamentStanding } from '../../api/client';
import { SingleEliminationBracket } from './SingleEliminationBracket';
import { RoundRobinView } from './RoundRobinView';
import { SwissRoundsView } from './SwissRoundsView';
import { MatchCard } from './MatchCard';

interface Props {
  format: string;
  rounds: TournamentRound[];
  standings: TournamentStanding[];
}

export function TournamentBracket({ format, rounds, standings }: Props) {
  if (rounds.length === 0) {
    return <p style={{ color: '#666' }}>No matches yet.</p>;
  }

  if (format === 'single_elimination') {
    return <SingleEliminationBracket rounds={rounds} />;
  }

  if (format === 'round_robin') {
    return <RoundRobinView rounds={rounds} />;
  }

  if (format.startsWith('swiss_')) {
    return <SwissRoundsView rounds={rounds} standings={standings} />;
  }

  // Fallback: generic round-by-round view
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
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
            {round.matches.map(m => (
              <MatchCard key={m.match_id} match={m} />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
