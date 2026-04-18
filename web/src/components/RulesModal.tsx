interface Props {
  onClose: () => void;
}

export default function RulesModal({ onClose }: Props) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-sheet" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div className="modal-header-tag">Instruction Guide</div>
          <h2 className="modal-title">How to Play</h2>
          <button className="modal-close" onClick={onClose}>✕</button>
        </div>
        <div className="modal-body">
          <p>
            Two players alternate naming Pokémon from your chosen generations (national dex).
            Each name must start with the <strong>last letter</strong> of the previous name.
            No repeats allowed. The first player who cannot name a Pokémon loses.
          </p>

          <div className="live-example">
            <div className="live-example-tag">
              <span style={{ width: 8, height: 8, borderRadius: '50%', background: 'var(--blue)', display: 'inline-block' }}/>
              Live Example
            </div>
            <div className="live-example-chain">
              <span className="live-example-word">
                Bulbsau<span className="highlight-letter">r</span>
              </span>
              <span className="live-example-arrow">→</span>
              <span className="live-example-word">
                <span className="highlight-letter">R</span>aichu
              </span>
              <span className="live-example-arrow">→</span>
              <span className="live-example-word pending">U…</span>
            </div>
          </div>

          <p style={{ marginBottom: 8 }}>
            <strong>Tips:</strong>
          </p>
          <ul style={{ paddingLeft: 20, fontSize: 14, color: 'var(--text-2)', lineHeight: 1.7, marginBottom: 20 }}>
            <li>Some letters have no Pokémon names starting with them in your pool (e.g. in full Gen 1, <strong>Q, U, X, Y</strong>).</li>
            <li>Use the <strong>Picker</strong> mode to browse legal options.</li>
            <li>Toggle <strong>Hint</strong> to see the engine's suggested move.</li>
            <li>Special characters are ignored: Farfetch'd ends with <strong>D</strong>.</li>
          </ul>

          <button className="btn btn-primary" onClick={onClose}>
            Understood
          </button>
        </div>
      </div>
    </div>
  );
}
