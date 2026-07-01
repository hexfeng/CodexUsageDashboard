import { Download, X } from "lucide-react";

interface UpdatePromptProps {
  version: string;
  notes?: string;
  installing?: boolean;
  error?: string | null;
  onInstall: () => void;
  onDismiss: () => void;
}

export function UpdatePrompt({ version, notes, installing = false, error, onInstall, onDismiss }: UpdatePromptProps) {
  return (
    <div className="update-prompt" role="status" aria-label="Update available">
      <div>
        <strong>Update {version} available</strong>
        {notes ? <p>{notes}</p> : null}
        {error ? <p className="diagnostics-error">{error}</p> : null}
      </div>
      <div className="update-actions">
        <button className="primary-button" type="button" onClick={onInstall} disabled={installing}>
          <Download size={14} />
          {installing ? "Installing" : "Update"}
        </button>
        <button className="icon-button" type="button" title="Later" aria-label="Dismiss update" onClick={onDismiss}>
          <X size={16} />
        </button>
      </div>
    </div>
  );
}
