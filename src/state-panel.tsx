import type { MouseEventHandler } from 'react';
import './state-panel.css';

export type StatePanelTone = 'neutral' | 'warning' | 'danger' | 'info';

export default function StatePanel({
  title,
  body,
  tone = 'neutral',
  badge,
  busy = false,
  action,
  onAction,
  headingLevel = 3,
}: {
  title: string;
  body: string;
  tone?: StatePanelTone;
  badge?: string;
  busy?: boolean;
  action?: string;
  onAction?: MouseEventHandler<HTMLButtonElement>;
  headingLevel?: 2 | 3;
}): JSX.Element {
  const Heading = headingLevel === 2 ? 'h2' : 'h3';
  return (
    <section
      className={`state-panel state-panel--${tone}${badge ? ' state-panel--operation' : ''}`}
      role={tone === 'danger' ? 'alert' : 'status'}
      aria-busy={busy || undefined}
    >
      {badge && <span className="state-panel__badge" dir="ltr" lang="en">{badge}</span>}
      <Heading className="t-card-title state-panel__title">{title}</Heading>
      <p className="t-body-sec state-panel__body">{body}</p>
      {busy && <span className="spinner state-panel__spinner" aria-hidden="true" />}
      {action && onAction && (
        <button
          type="button"
          className={`btn state-panel__action${tone === 'warning' ? ' state-panel__action--warning' : ''}${tone === 'danger' ? ' state-panel__action--danger' : ''}`}
          onClick={onAction}
        >
          {action}
        </button>
      )}
    </section>
  );
}
