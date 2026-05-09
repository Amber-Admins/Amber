import { useEffect, useState } from "react";
import { CONTEXT_MAX_TOKENS, type ContextAssemblerScope } from "../constants/contextBudget";
import { debugBuildContext } from "../services/nodes";
import { AppError } from "../services/ipcResult";

type ScopeIndicatorProps = {
  selectedNodeIds: string[];
  scope: ContextAssemblerScope;
  onScopeChange: (scope: ContextAssemblerScope) => void;
};

function ScopeIndicator({ selectedNodeIds, scope, onScopeChange }: ScopeIndicatorProps) {
  const [tokenEstimate, setTokenEstimate] = useState(0);
  const [status, setStatus] = useState("");

  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const context = await debugBuildContext(selectedNodeIds, scope);
        if (!active) {
          return;
        }
        setTokenEstimate(Math.floor(context.length / 4));
        setStatus("");
      } catch (err) {
        if (!active) {
          return;
        }
        if (err instanceof AppError) {
          setStatus(err.message);
        } else {
          setStatus("Unable to estimate token usage.");
        }
      }
    })();
    return () => {
      active = false;
    };
  }, [scope, selectedNodeIds]);

  const maxTokens = CONTEXT_MAX_TOKENS;
  const overBudget = tokenEstimate > maxTokens;

  return (
    <section className="scope-indicator">
      <div className="scope-indicator-row scope-indicator-row-controls">
        <span className="scope-indicator-label">Assembler scope</span>
        <div
          className="scope-indicator-scope-toggle"
          role="group"
          aria-label="Assembler privacy scope"
        >
          <button
            type="button"
            className={`scope-scope-btn ${scope === "local" ? "active" : ""}`}
            onClick={() => onScopeChange("local")}
          >
            Local
          </button>
          <button
            type="button"
            className={`scope-scope-btn ${scope === "cloud" ? "active" : ""}`}
            onClick={() => onScopeChange("cloud")}
          >
            Cloud
          </button>
        </div>
      </div>
      <div className="scope-indicator-row">
        <span>Nodes in Context: {selectedNodeIds.length}</span>
        <span>
          Estimated Tokens: {tokenEstimate} / {maxTokens}
        </span>
      </div>
      <progress
        className={`scope-indicator-track ${overBudget ? "danger" : ""}`}
        value={Math.min(tokenEstimate, maxTokens)}
        max={maxTokens}
      />
      {status && <p className="scope-indicator-status">{status}</p>}
    </section>
  );
}

export default ScopeIndicator;
