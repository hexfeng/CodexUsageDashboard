import { formatPercent, getLimitTone } from "../lib/format";
import type { LimitBucket } from "../types";

interface LimitBarProps {
  limit: LimitBucket;
}

export function LimitBar({ limit }: LimitBarProps) {
  const displayLabel = limit.label === "5h" ? "5 Hours" : "Weekly";

  if (!limit.available) {
    return (
      <section className="limit-row limit-row-unavailable" aria-label={`${limit.label} unavailable`}>
        <div className="limit-copy">
          <span className="limit-label">{displayLabel}</span>
          <span className="limit-value">Usage limit unavailable</span>
        </div>
        <div className="limit-track" aria-hidden="true">
          <span className="limit-fill tone-unavailable" style={{ width: "100%" }} />
        </div>
      </section>
    );
  }

  const usedPercent = limit.usedPercent ?? 0;
  const remainingPercent = Math.max(0, Math.round(limit.remainingPercent ?? 100 - usedPercent));
  const tone = getLimitTone(usedPercent);
  const activeSegments = Math.max(0, Math.min(10, Math.round(remainingPercent / 10)));

  return (
    <section className="limit-row" aria-label={`${limit.label} ${formatPercent(usedPercent)} used`}>
      <div className="limit-copy">
        <span className="limit-label">{displayLabel}</span>
        <div className="segment-bar" aria-label={`${displayLabel} remaining ${remainingPercent}%`}>
          {Array.from({ length: 10 }, (_, index) => (
            <span
              className={`segment segment-${tone} ${index < activeSegments ? "segment-active" : ""}`}
              key={index}
            />
          ))}
        </div>
        <span className="limit-value">Remaining {remainingPercent}%</span>
        <span className="limit-reset">{limit.resetLabel ?? "unknown"}</span>
      </div>
      {limit.unusual ? <div className="limit-note">unusual reset window</div> : null}
    </section>
  );
}
