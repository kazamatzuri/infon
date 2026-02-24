import type { TournamentRound } from '../../api/client';
import { MatchCard } from './MatchCard';

interface Props {
  rounds: TournamentRound[];
}

function roundLabel(round: number, totalRounds: number): string {
  if (round === totalRounds) return 'Final';
  if (round === totalRounds - 1) return 'Semifinal';
  return `Round ${round}`;
}

export function SingleEliminationBracket({ rounds }: Props) {
  const totalRounds = rounds.length;

  return (
    <div style={{ overflowX: 'auto', paddingBottom: '16px' }}>
      <div style={{ display: 'flex', gap: '0', minWidth: 'fit-content' }}>
        {rounds.map((round, ri) => (
          <div key={round.round} style={{ display: 'flex', flexDirection: 'column', minWidth: '240px' }}>
            <h4 style={{
              color: '#aaa',
              fontSize: '12px',
              textTransform: 'uppercase',
              letterSpacing: '0.5px',
              margin: '0 0 12px 0',
              textAlign: 'center',
            }}>
              {roundLabel(round.round, totalRounds)}
            </h4>
            <div style={{
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'space-around',
              flex: 1,
              gap: '8px',
              position: 'relative',
            }}>
              {round.matches.map((match, mi) => (
                <div key={match.match_id} style={{ display: 'flex', alignItems: 'center' }}>
                  <div style={{ flex: 1, padding: '4px 8px' }}>
                    <MatchCard match={match} />
                  </div>
                  {/* Connector line to next round */}
                  {ri < totalRounds - 1 && (
                    <div style={{
                      width: '24px',
                      position: 'relative',
                    }}>
                      <div style={{
                        position: 'absolute',
                        top: '50%',
                        left: 0,
                        width: '24px',
                        height: '1px',
                        background: '#444',
                      }} />
                      {mi % 2 === 0 && (
                        <div style={{
                          position: 'absolute',
                          top: '50%',
                          right: 0,
                          width: '1px',
                          height: `calc(${100 * Math.pow(2, ri)}% + 0px)`,
                          background: '#444',
                        }} />
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
