import { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import Editor from '@monaco-editor/react';
import { api } from '../api/client';
import type { Bot, BotVersion } from '../api/client';

const DEFAULT_CODE = `-- Your Infon bot
-- API: oo (Object-Oriented)

function Creature:onSpawned()
    -- Called when creature is created
end

function Creature:main()
    -- Main creature logic runs each tick
    if self:tile_food() > 0 and self:food() < self:max_food() then
        self:eat()
    elseif self:health() < 80 then
        self:heal()
    else
        local x1, y1, x2, y2 = world_size()
        self:moveto(math.random(x1, x2), math.random(y1, y2))
    end
end
`;

export function BotEditor() {
  const { botId } = useParams<{ botId: string }>();
  const navigate = useNavigate();

  const [bot, setBot] = useState<Bot | null>(null);
  const [versions, setVersions] = useState<BotVersion[]>([]);
  const [currentVersion, setCurrentVersion] = useState<BotVersion | null>(null);
  const [code, setCode] = useState(DEFAULT_CODE);
  const [apiType, setApiType] = useState('oo');
  const [botName, setBotName] = useState('');
  const [botDescription, setBotDescription] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  const loadBot = useCallback(async (id: number) => {
    try {
      setError(null);
      const [botData, versionsData] = await Promise.all([
        api.getBot(id),
        api.listVersions(id),
      ]);
      setBot(botData);
      setBotName(botData.name);
      setBotDescription(botData.description || '');
      setVersions(versionsData);

      if (versionsData.length > 0) {
        const latest = versionsData[versionsData.length - 1];
        setCurrentVersion(latest);
        setCode(latest.code);
        setApiType(latest.api_type || 'oo');
      } else {
        setCode(DEFAULT_CODE);
        setApiType('oo');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load bot');
    }
  }, []);

  useEffect(() => {
    if (botId) {
      loadBot(parseInt(botId, 10));
    }
  }, [botId, loadBot]);

  const handleSaveVersion = async () => {
    if (!bot) return;
    setSaving(true);
    setError(null);
    setSuccessMsg(null);
    try {
      // Update bot name/description if changed
      if (botName !== bot.name || botDescription !== (bot.description || '')) {
        await api.updateBot(bot.id, botName, botDescription);
        setBot({ ...bot, name: botName, description: botDescription });
      }

      const newVersion = await api.createVersion(bot.id, code, apiType);
      setVersions(prev => [...prev, newVersion]);
      setCurrentVersion(newVersion);
      setSuccessMsg(`Saved as version ${newVersion.version}`);
      setTimeout(() => setSuccessMsg(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save');
    } finally {
      setSaving(false);
    }
  };

  const handleVersionChange = async (versionId: number) => {
    if (!bot) return;
    try {
      const ver = await api.getVersion(bot.id, versionId);
      setCurrentVersion(ver);
      setCode(ver.code);
      setApiType(ver.api_type || 'oo');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load version');
    }
  };

  if (!botId) {
    return (
      <div style={{ padding: '24px', textAlign: 'center', color: '#888' }}>
        <p>Select a bot from the library or create a new one.</p>
        <button onClick={() => navigate('/')} style={btnPrimary}>
          Go to Bot Library
        </button>
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {/* Top bar */}
      <div style={{ padding: '12px 24px', background: '#16213e', borderBottom: '1px solid #333', display: 'flex', alignItems: 'center', gap: '16px', flexWrap: 'wrap' }}>
        <input
          value={botName}
          onChange={e => setBotName(e.target.value)}
          placeholder="Bot name"
          style={inputStyle}
        />
        <input
          value={botDescription}
          onChange={e => setBotDescription(e.target.value)}
          placeholder="Description"
          style={{ ...inputStyle, width: '250px' }}
        />
        <select
          value={apiType}
          onChange={e => setApiType(e.target.value)}
          style={selectStyle}
        >
          <option value="oo">API: oo</option>
          <option value="state">API: state</option>
        </select>
        <select
          value={currentVersion?.id || ''}
          onChange={e => handleVersionChange(parseInt(e.target.value, 10))}
          style={selectStyle}
          disabled={versions.length === 0}
        >
          {versions.length === 0 ? (
            <option value="">No versions</option>
          ) : (
            versions.map(v => (
              <option key={v.id} value={v.id}>
                Version {v.version} - {new Date(v.created_at).toLocaleString()}
              </option>
            ))
          )}
        </select>
        <button onClick={handleSaveVersion} disabled={saving} style={btnPrimary}>
          {saving ? 'Saving...' : 'Save Version'}
        </button>
      </div>

      {/* Messages */}
      {error && (
        <div style={{ padding: '8px 24px', background: '#5c1a1a', color: '#ff8a8a', fontSize: '13px' }}>
          {error}
        </div>
      )}
      {successMsg && (
        <div style={{ padding: '8px 24px', background: '#1a3a1a', color: '#16c79a', fontSize: '13px' }}>
          {successMsg}
        </div>
      )}

      {/* Editor */}
      <div style={{ flex: 1, minHeight: 0 }}>
        <Editor
          height="100%"
          defaultLanguage="lua"
          theme="vs-dark"
          value={code}
          onChange={value => setCode(value || '')}
          options={{
            fontSize: 14,
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            padding: { top: 12 },
            lineNumbers: 'on',
            renderLineHighlight: 'line',
            tabSize: 4,
            insertSpaces: true,
          }}
        />
      </div>
    </div>
  );
}

const inputStyle: React.CSSProperties = {
  background: '#0a0a1a',
  color: '#e0e0e0',
  border: '1px solid #333',
  borderRadius: '4px',
  padding: '6px 12px',
  fontSize: '14px',
  width: '180px',
};

const selectStyle: React.CSSProperties = {
  background: '#0a0a1a',
  color: '#e0e0e0',
  border: '1px solid #333',
  borderRadius: '4px',
  padding: '6px 12px',
  fontSize: '14px',
};

const btnPrimary: React.CSSProperties = {
  background: '#16c79a',
  color: '#fff',
  border: 'none',
  padding: '8px 20px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontWeight: 600,
  fontSize: '14px',
};
