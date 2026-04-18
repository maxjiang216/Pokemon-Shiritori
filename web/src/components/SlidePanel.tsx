import type { ReactNode } from 'react';

interface Props {
  title: string;
  children: ReactNode;
  onClose: () => void;
}

export default function SlidePanel({ title, children, onClose }: Props) {
  return (
    <div
      className="slide-panel-overlay"
      role="presentation"
      onClick={onClose}
    >
      <div
        className="slide-panel-sheet"
        role="dialog"
        aria-modal="true"
        aria-labelledby="slide-panel-title"
        onClick={e => e.stopPropagation()}
      >
        <div className="slide-panel-header">
          <h2 id="slide-panel-title" className="slide-panel-title">
            {title}
          </h2>
          <button
            type="button"
            className="slide-panel-close"
            onClick={onClose}
            aria-label="Close"
          >
            ✕
          </button>
        </div>
        <div className="slide-panel-body">{children}</div>
      </div>
    </div>
  );
}
