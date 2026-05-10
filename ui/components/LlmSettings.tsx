import { useEffect, useState } from "react";
import { DEV_ONBOARDING_CHANGED } from "../constants/devEvents";
import { getLlmModels } from "../services/nodes";
import { AppError } from "../services/ipcResult";
import { getOnboardingComplete, setOnboardingComplete } from "../services/settings";
import {
  getLlmModel,
  getLlmProvider,
  getLmStudioEndpoint,
  getOllamaEndpoint,
  setLlmModel,
  setLlmProvider,
  setLmStudioEndpoint,
  setOllamaEndpoint,
} from "../utils/settings";

type Provider = "ollama" | "lmstudio";

function LlmSettings() {
  const showDevOnboardingTools = import.meta.env.DEV;
  const [onboardingCompleteLabel, setOnboardingCompleteLabel] = useState<string>("…");
  const [onboardingDevBusy, setOnboardingDevBusy] = useState(false);

  const [provider, setProvider] = useState<Provider>(() => {
    return getLlmProvider() === "lmstudio" ? "lmstudio" : "ollama";
  });
  const [ollamaEndpoint, setOllamaEndpointState] = useState(() => getOllamaEndpoint());
  const [lmStudioEndpoint, setLmStudioEndpointState] = useState(() => getLmStudioEndpoint());
  const [models, setModels] = useState<string[]>([]);
  const [selectedModel, setSelectedModel] = useState(() => getLlmModel());
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const endpoint = provider === "ollama" ? ollamaEndpoint : lmStudioEndpoint;

  useEffect(() => {
    if (!showDevOnboardingTools) {
      return;
    }
    void (async () => {
      try {
        const done = await getOnboardingComplete();
        setOnboardingCompleteLabel(done ? "complete" : "not complete");
      } catch {
        setOnboardingCompleteLabel("error");
      }
    })();
  }, [showDevOnboardingTools]);

  async function refreshOnboardingDevState() {
    try {
      const done = await getOnboardingComplete();
      setOnboardingCompleteLabel(done ? "complete" : "not complete");
    } catch {
      setOnboardingCompleteLabel("error");
    }
    window.dispatchEvent(new CustomEvent(DEV_ONBOARDING_CHANGED));
  }

  async function onTestConnection() {
    setIsLoading(true);
    setStatus("");
    try {
      const fetchedModels = await getLlmModels(provider, endpoint.trim());
      setModels(fetchedModels);
      if (fetchedModels.length === 0) {
        setStatus("Connected, but no models were returned.");
      } else {
        const nextModel =
          selectedModel && fetchedModels.includes(selectedModel) ? selectedModel : fetchedModels[0];
        setSelectedModel(nextModel);
        setLlmModel(nextModel);
        setStatus(`Connected. Found ${fetchedModels.length} model(s).`);
      }
    } catch (err) {
      if (err instanceof AppError) {
        setStatus(err.message);
      } else {
        setStatus("Failed to connect to endpoint.");
      }
    }
    setIsLoading(false);
  }

  function onSaveSettings() {
    setLlmProvider(provider);
    if (provider === "ollama") {
      setOllamaEndpoint(ollamaEndpoint);
      setStatus("Saved Ollama settings.");
    } else {
      setLmStudioEndpoint(lmStudioEndpoint);
      setStatus("Saved LM Studio settings.");
    }
  }

  function onSelectModel(model: string) {
    setSelectedModel(model);
    setLlmModel(model);
    setStatus("Saved model.");
  }

  function onProviderChange(nextProvider: Provider) {
    setProvider(nextProvider);
    setLlmProvider(nextProvider);
    setModels([]);
    setStatus("");
  }

  return (
    <aside className="pane pane-right llm-settings">
      <div className="pane-header">
        <h3>⚙️ LLM Settings</h3>
      </div>

      <div className="provider-toggle" role="radiogroup" aria-label="LLM provider">
        <label>
          <input
            type="radio"
            name="llm-provider"
            checked={provider === "ollama"}
            onChange={() => onProviderChange("ollama")}
          />
          Ollama
        </label>
        <label>
          <input
            type="radio"
            name="llm-provider"
            checked={provider === "lmstudio"}
            onChange={() => onProviderChange("lmstudio")}
          />
          LM Studio
        </label>
      </div>

      <label className="settings-field">
        <span>Endpoint URL</span>
        <input
          type="text"
          value={endpoint}
          onChange={(event) => {
            const nextValue = event.target.value;
            if (provider === "ollama") {
              setOllamaEndpointState(nextValue);
            } else {
              setLmStudioEndpointState(nextValue);
            }
          }}
          placeholder={provider === "ollama" ? "http://localhost:11434" : "http://localhost:1234"}
        />
      </label>

      <button type="button" className="settings-action" onClick={() => void onTestConnection()}>
        {isLoading ? "Testing..." : "Test Connection & Fetch Models"}
      </button>

      <label className="settings-field">
        <span>Model</span>
        <select
          value={selectedModel}
          onChange={(event) => onSelectModel(event.target.value)}
          disabled={models.length === 0}
        >
          {models.length === 0 ? (
            <option value="">No models loaded</option>
          ) : (
            models.map((model) => (
              <option key={model} value={model}>
                {model}
              </option>
            ))
          )}
        </select>
      </label>

      <button type="button" className="settings-action save" onClick={onSaveSettings}>
        Save
      </button>

      {showDevOnboardingTools ? (
        <div className="llm-settings-dev" aria-label="Developer onboarding shortcuts">
          <h4 className="llm-settings-dev-title">Developer</h4>
          <p className="llm-settings-dev-line">
            Onboarding: <strong>{onboardingCompleteLabel}</strong>
          </p>
          <div className="llm-settings-dev-actions">
            <button
              type="button"
              className="settings-action"
              disabled={onboardingDevBusy}
              onClick={() => {
                setOnboardingDevBusy(true);
                void (async () => {
                  try {
                    await setOnboardingComplete(false);
                    await refreshOnboardingDevState();
                    setStatus("Onboarding reset: wizard should appear.");
                  } catch (err) {
                    setStatus(err instanceof Error ? err.message : String(err));
                  } finally {
                    setOnboardingDevBusy(false);
                  }
                })();
              }}
            >
              Reset onboarding
            </button>
            <button
              type="button"
              className="settings-action"
              disabled={onboardingDevBusy}
              onClick={() => {
                setOnboardingDevBusy(true);
                void (async () => {
                  try {
                    await setOnboardingComplete(true);
                    await refreshOnboardingDevState();
                    setStatus("Onboarding marked complete.");
                  } catch (err) {
                    setStatus(err instanceof Error ? err.message : String(err));
                  } finally {
                    setOnboardingDevBusy(false);
                  }
                })();
              }}
            >
              Mark onboarding done
            </button>
          </div>
          <p className="llm-settings-dev-note">Shown only in dev builds.</p>
        </div>
      ) : null}

      {status && <p className="pane-status">{status}</p>}
    </aside>
  );
}

export default LlmSettings;
