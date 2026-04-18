import type { GameHandle } from '../wasm-pkg/pokemon_shiritori';
import type { Records } from '../App';

interface HistEntry { name: string; by_human: boolean; }

interface Props {
  game: GameHandle;
  records: Records;
  elapsedMs: number;
  onRestart: () => void;
}

function formatTime(ms: number): string {
  const s = Math.floor(ms / 1000);
  const m = Math.floor(s / 60);
  const rem = s % 60;
  return `${String(m).padStart(2, '0')}:${String(rem).padStart(2, '0')}`;
}

export default function VictoryScreen({ game, records, elapsedMs, onRestart }: Props) {
  const won = game.human_won();
  const totalMoves = game.used_count();
  const history = (game.history_json() as HistEntry[]).map(e => e.name);
  const cpuName = game.cpu_name();

  // The losing side is whoever couldn't move; the last entry in history played the
  // winning move, and the next player had no response.
  const lastMove = history[history.length - 1];
  const loserIsHuman = !won;
  const lastLetter = lastMove
    ? lastMove.replace(/[^a-zA-Z]/g, '').slice(-1).toUpperCase()
    : '?';

  const recentChain = history.slice(-6);

  return (
    <div className="victory-screen">
      <div className="trophy-wrap">{won ? '🏆' : '😔'}</div>

      <h1 className="result-title">{won ? 'You Won!' : 'You Lost!'}</h1>
      <p className="result-subtitle">
        {won ? 'Master Class Victory' : `${cpuName} wins this round`}
      </p>

      <div className="stats-row">
        <div className="stat-card">
          <div className="stat-icon">📋</div>
          <div className="stat-label">Total Moves</div>
          <div className="stat-value">{totalMoves}</div>
        </div>
        <div className="stat-card">
          <div className="stat-icon">⏱</div>
          <div className="stat-label">Total Time</div>
          <div className="stat-value">{formatTime(elapsedMs)}</div>
        </div>
        <div className="stat-card highlight">
          <div className="stat-icon">⭐</div>
          <div className="stat-label">Win Streak</div>
          <div className="stat-value">{records.streak}</div>
        </div>
      </div>

      {history.length > 0 && (
        <div className="chain-replay">
          <div className="chain-replay-title">
            🔗 Final Word Chain
          </div>
          <div className="chain-replay-chips">
            {recentChain.map((name, i) => {
              const isLast = i === recentChain.length - 1;
              return (
                <span key={i} style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
                  <span className={`chain-chip ${isLast ? 'last' : ''}`}>{name}</span>
                  {!isLast && <span className="chain-chip-arrow">→</span>}
                </span>
              );
            })}
          </div>
          <p className="chain-replay-msg">
            {loserIsHuman
              ? `You couldn't find a Pokémon starting with '${lastLetter}'!`
              : `${cpuName} couldn't find a Pokémon starting with '${lastLetter}'!`}
          </p>
        </div>
      )}

      <button className="btn btn-blue" onClick={onRestart} style={{ width: '100%' }}>
        ← Back to Setup
      </button>
    </div>
  );
}
