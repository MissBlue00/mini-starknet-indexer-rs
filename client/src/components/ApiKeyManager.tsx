import React, { useState } from 'react';
import { useDeploymentClientManager } from '../hooks/useDeploymentClient';

interface ApiKeyManagerProps {
  deploymentId: string;
  deploymentName?: string;
}

export function ApiKeyManager({ deploymentId, deploymentName }: ApiKeyManagerProps) {
  const { setApiKey, getApiKey } = useDeploymentClientManager();
  const [apiKey, setApiKeyValue] = useState(getApiKey(deploymentId) || '');
  const [showApiKey, setShowApiKey] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const handleSaveApiKey = () => {
    if (!apiKey.trim()) {
      setMessage({ type: 'error', text: 'Please enter an API key' });
      return;
    }

    try {
      setApiKey(deploymentId, apiKey.trim());
      setMessage({ type: 'success', text: 'API key saved successfully!' });
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to save API key' });
    }
  };

  const handleClearApiKey = () => {
    setApiKeyValue('');
    setMessage(null);
  };

  const toggleApiKeyVisibility = () => {
    setShowApiKey(!showApiKey);
  };

  return (
    <div className="api-key-manager">
      <h3>API Key Management</h3>
      {deploymentName && <p>Deployment: {deploymentName}</p>}
      
      <div className="api-key-input">
        <label htmlFor="api-key">
          API Key:
        </label>
        <div className="input-group">
          <input
            id="api-key"
            type={showApiKey ? 'text' : 'password'}
            value={apiKey}
            onChange={(e) => setApiKeyValue(e.target.value)}
            placeholder="Enter your API key (starts with sk_)"
            className="api-key-input-field"
          />
          <button
            type="button"
            onClick={toggleApiKeyVisibility}
            className="toggle-visibility-btn"
            title={showApiKey ? 'Hide API key' : 'Show API key'}
          >
            {showApiKey ? '🙈' : '👁️'}
          </button>
        </div>
      </div>

      <div className="api-key-actions">
        <button
          onClick={handleSaveApiKey}
          className="save-btn"
          disabled={!apiKey.trim()}
        >
          Save API Key
        </button>
        <button
          onClick={handleClearApiKey}
          className="clear-btn"
        >
          Clear
        </button>
      </div>

      {message && (
        <div className={`message ${message.type}`}>
          {message.text}
        </div>
      )}

      <div className="api-key-help">
        <h4>How to get your API key:</h4>
        <ol>
          <li>Create a deployment using the main GraphQL endpoint</li>
          <li>Check the server console output for the generated API key</li>
          <li>Copy the key (starts with <code>sk_</code>) and paste it above</li>
          <li>Save the key securely - it won't be shown again!</li>
        </ol>
        
        <h4>Alternative header formats:</h4>
        <ul>
          <li><code>Authorization: Bearer sk_your_key_here</code></li>
          <li><code>Authorization: ApiKey sk_your_key_here</code></li>
          <li><code>X-API-Key: sk_your_key_here</code></li>
          <li><code>X-Auth-Token: sk_your_key_here</code></li>
        </ul>
      </div>

      <style jsx>{`
        .api-key-manager {
          max-width: 600px;
          margin: 20px 0;
          padding: 20px;
          border: 1px solid #ddd;
          border-radius: 8px;
          background: #f9f9f9;
        }

        .api-key-manager h3 {
          margin-top: 0;
          color: #333;
        }

        .api-key-manager h4 {
          color: #555;
          margin-bottom: 10px;
        }

        .api-key-input {
          margin: 15px 0;
        }

        .api-key-input label {
          display: block;
          margin-bottom: 5px;
          font-weight: bold;
          color: #333;
        }

        .input-group {
          display: flex;
          gap: 5px;
        }

        .api-key-input-field {
          flex: 1;
          padding: 8px 12px;
          border: 1px solid #ccc;
          border-radius: 4px;
          font-family: monospace;
          font-size: 14px;
        }

        .toggle-visibility-btn {
          padding: 8px 12px;
          border: 1px solid #ccc;
          border-radius: 4px;
          background: white;
          cursor: pointer;
          font-size: 16px;
        }

        .toggle-visibility-btn:hover {
          background: #f0f0f0;
        }

        .api-key-actions {
          display: flex;
          gap: 10px;
          margin: 15px 0;
        }

        .save-btn {
          padding: 8px 16px;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .save-btn:hover:not(:disabled) {
          background: #0056b3;
        }

        .save-btn:disabled {
          background: #ccc;
          cursor: not-allowed;
        }

        .clear-btn {
          padding: 8px 16px;
          background: #6c757d;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .clear-btn:hover {
          background: #545b62;
        }

        .message {
          padding: 10px;
          border-radius: 4px;
          margin: 10px 0;
        }

        .message.success {
          background: #d4edda;
          color: #155724;
          border: 1px solid #c3e6cb;
        }

        .message.error {
          background: #f8d7da;
          color: #721c24;
          border: 1px solid #f5c6cb;
        }

        .api-key-help {
          margin-top: 20px;
          padding: 15px;
          background: white;
          border-radius: 4px;
          border: 1px solid #ddd;
        }

        .api-key-help ol,
        .api-key-help ul {
          margin: 10px 0;
          padding-left: 20px;
        }

        .api-key-help li {
          margin: 5px 0;
        }

        .api-key-help code {
          background: #f1f1f1;
          padding: 2px 4px;
          border-radius: 3px;
          font-family: monospace;
          font-size: 12px;
        }
      `}</style>
    </div>
  );
}
