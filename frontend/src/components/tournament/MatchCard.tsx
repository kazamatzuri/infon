import { useNavigate } from 'react-router-dom';
import type { TournamentMatchInfo } from '../../api/client';

interface MatchCardProps {
  match: TournamentMatchInfo;
}

export function MatchCard({ match }: MatchCardProps) {
  const navigate = useNavigate();
  const p1 = match.participants[0];
  const p2 = match.participants[1];
  const isFinished = match.status === 'finished';

  const isWinner = (bvid: number) => isFinished && match.winner_bot_version_id === bvid;
  const isDraw = isFinished && match.winner_bot_version_id === null;
  // A participant likely failed to load if they lost with score 0
  const isFaultyLoser = (p: typeof p1) =>
    isFinished && p && p.final_score === 0 && match.winner_bot_version_id !== null && match.winner_bot_version_id !== p.bot_version_id;

  return (
    <div
      onClick={() => navigate(`/matches/${match.match_id}`)}
      style={{
        background: '#16213e',
        border: '1px solid #333',
        borderRadius: '6px',
        padding: '10px 14px',
        cursor: 'pointer',
        minWidth: '200px',
        transition: 'border-color 0.15s',
      }}
      onMouseEnter={e => (e.currentTarget.style.borderColor = '#16c79a')}
      onMouseLeave={e => (e.currentTarget.style.borderColor = '#333')}
    >
      {p1 && p2 ? (
        <>
          <ParticipantRow
            name={p1.bot_name || `Bot #${p1.bot_version_id}`}
            score={p1.final_score}
            highlight={isWinner(p1.bot_version_id)}
            dim={isFinished && !isWinner(p1.bot_version_id) && !isDraw}
            faulty={isFaultyLoser(p1)}
          />
          <div style={{ borderTop: '1px solid #2a2a4a', margin: '4px 0' }} />
          <ParticipantRow
            name={p2.bot_name || `Bot #${p2.bot_version_id}`}
            score={p2.final_score}
            highlight={isWinner(p2.bot_version_id)}
            dim={isFinished && !isWinner(p2.bot_version_id) && !isDraw}
            faulty={isFaultyLoser(p2)}
          />
        </>
      ) : (
        <div style={{ color: '#666', fontSize: '13px', textAlign: 'center', padding: '8px 0' }}>TBD</div>
      )}
      {!isFinished && match.status !== 'pending' && (
        <div style={{ color: '#f5a623', fontSize: '11px', textAlign: 'center', marginTop: '4px' }}>
          {match.status}
        </div>
      )}
    </div>
  );
}

function ParticipantRow({ name, score, highlight, dim, faulty }: { name: string; score: number; highlight: boolean; dim: boolean; faulty?: boolean }) {
  return (
    <div style={{
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      gap: '12px',
      opacity: dim ? 0.5 : 1,
    }}>
      <span style={{
        color: faulty ? '#e94560' : highlight ? '#16c79a' : '#e0e0e0',
        fontWeight: highlight ? 700 : 400,
        fontSize: '13px',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}>
        {highlight && '\u2713 '}{name}{faulty && ' (DQ)'}
      </span>
      <span style={{ color: '#aaa', fontSize: '13px', fontWeight: 600, flexShrink: 0 }}>
        {score}
      </span>
    </div>
  );
}
