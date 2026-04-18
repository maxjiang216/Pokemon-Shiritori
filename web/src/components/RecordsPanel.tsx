import type { Records } from '../types';

export default function RecordsPanel({
  records,
  onWipeRecords,
}: {
  records: Records;
  onWipeRecords: () => void;
}) {
  const total = records.wins + records.losses;
  const pct = total > 0 ? Math.round((records.wins / total) * 100) : 0;
  const hasStats = total > 0 || records.bestStreak > 0;
  return (
    <div className="records-page">
      <div className="records-hero">
        <div className="records-hero-num">{pct}%</div>
        <div className="records-hero-label">Win rate</div>
      </div>
      <div className="records-row">
        <div className="stat-card" style={{ flex: 1 }}>
          <div className="stat-label">Wins</div>
          <div className="stat-value" style={{ color: 'var(--green)' }}>{records.wins}</div>
        </div>
        <div className="stat-card" style={{ flex: 1 }}>
          <div className="stat-label">Losses</div>
          <div className="stat-value" style={{ color: 'var(--red)' }}>{records.losses}</div>
        </div>
      </div>
      <div className="records-row">
        <div className="stat-card highlight" style={{ flex: 1 }}>
          <div className="stat-label">Current streak</div>
          <div className="stat-value">{records.streak}</div>
        </div>
        <div className="stat-card" style={{ flex: 1 }}>
          <div className="stat-label">Best streak</div>
          <div className="stat-value">{records.bestStreak}</div>
        </div>
      </div>

      <button
        type="button"
        className="btn-records-wipe"
        onClick={onWipeRecords}
        disabled={!hasStats}
      >
        Clear all records
      </button>
    </div>
  );
}
