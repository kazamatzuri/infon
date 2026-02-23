import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';

type Section = 'getting-started' | 'lua-api' | 'strategy' | 'faq';

const sectionLabels: Record<Section, string> = {
  'getting-started': 'Getting Started',
  'lua-api': 'Lua API Reference',
  'strategy': 'Strategy Guide',
  'faq': 'FAQ / Troubleshooting',
};

function getInitialSection(): Section {
  const hash = window.location.hash.replace('#', '') as Section;
  if (hash && hash in sectionLabels) return hash;
  return 'getting-started';
}

export function Documentation() {
  const [activeSection, setActiveSection] = useState<Section>(getInitialSection);
  const [luaApiResult, setLuaApiResult] = useState<{ content?: string; error?: string } | null>(null);

  // Load Lua API content when that section is selected
  useEffect(() => {
    if (activeSection !== 'lua-api' || luaApiResult !== null) return;
    let cancelled = false;
    fetch('/api/docs/lua-api')
      .then(r => {
        if (!r.ok) throw new Error('Failed to load Lua API documentation');
        return r.text();
      })
      .then(text => { if (!cancelled) setLuaApiResult({ content: text }); })
      .catch(err => { if (!cancelled) setLuaApiResult({ error: err.message }); });
    return () => { cancelled = true; };
  }, [activeSection, luaApiResult]);

  const navigate = useNavigate();
  const switchSection = useCallback((section: Section) => {
    setActiveSection(section);
    navigate(`#${section}`, { replace: true });
  }, [navigate]);

  return (
    <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
      {/* Sidebar */}
      <nav style={{
        width: 220,
        minWidth: 220,
        background: '#0d1117',
        borderRight: '1px solid #1a3a5c',
        padding: '20px 0',
        overflowY: 'auto',
      }}>
        <h3 style={{ color: '#e0e0e0', padding: '0 16px', marginTop: 0, marginBottom: 16, fontSize: 15 }}>
          Documentation
        </h3>
        {(Object.keys(sectionLabels) as Section[]).map(key => (
          <button
            key={key}
            onClick={() => switchSection(key)}
            style={{
              display: 'block',
              width: '100%',
              padding: '10px 16px',
              background: activeSection === key ? 'rgba(22,199,154,0.12)' : 'transparent',
              border: 'none',
              borderLeft: activeSection === key ? '3px solid #16c79a' : '3px solid transparent',
              color: activeSection === key ? '#16c79a' : '#aaa',
              textAlign: 'left',
              cursor: 'pointer',
              fontSize: 14,
              fontFamily: 'inherit',
              transition: 'all 0.15s',
            }}
            onMouseOver={e => {
              if (activeSection !== key) e.currentTarget.style.color = '#e0e0e0';
            }}
            onMouseOut={e => {
              if (activeSection !== key) e.currentTarget.style.color = '#aaa';
            }}
          >
            {sectionLabels[key]}
          </button>
        ))}
      </nav>

      {/* Content */}
      <div style={{ flex: 1, overflowY: 'auto', padding: 32 }}>
        <div style={{ maxWidth: 800 }}>
          {activeSection === 'getting-started' && <GettingStarted />}
          {activeSection === 'lua-api' && (
            <LuaApiReference content={luaApiResult?.content ?? ''} loading={luaApiResult === null} error={luaApiResult?.error ?? null} />
          )}
          {activeSection === 'strategy' && <StrategyGuide />}
          {activeSection === 'faq' && <FAQ />}
        </div>
      </div>
    </div>
  );
}

/* ── Section Components ─────────────────────────────────────────────── */

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <>
      <h2 style={{ color: '#e0e0e0', marginTop: 0, marginBottom: 8 }}>{children}</h2>
      <div style={{
        width: 50,
        height: 3,
        background: 'linear-gradient(90deg, #16c79a, #f5a623)',
        borderRadius: 2,
        marginBottom: 24,
      }} />
    </>
  );
}

function Card({ title, children }: { title?: string; children: React.ReactNode }) {
  return (
    <div style={{
      background: '#16213e',
      borderRadius: 10,
      padding: 24,
      marginBottom: 20,
      border: '1px solid #1a3a5c',
    }}>
      {title && <h3 style={{ color: '#16c79a', marginTop: 0, marginBottom: 12, fontSize: 16 }}>{title}</h3>}
      {children}
    </div>
  );
}

function StepNumber({ n }: { n: number }) {
  return (
    <span style={{
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      width: 28,
      height: 28,
      borderRadius: '50%',
      background: '#16c79a',
      color: '#0a0a1a',
      fontWeight: 700,
      fontSize: 14,
      marginRight: 12,
      flexShrink: 0,
    }}>
      {n}
    </span>
  );
}

function CodeBlock({ children }: { children: string }) {
  return (
    <pre style={{
      background: '#0a0a1a',
      border: '1px solid #333',
      borderRadius: 6,
      padding: 16,
      color: '#ccc',
      fontSize: 13,
      lineHeight: 1.5,
      overflowX: 'auto',
      whiteSpace: 'pre-wrap',
      wordWrap: 'break-word',
      margin: '12px 0',
    }}>
      {children}
    </pre>
  );
}

const textStyle: React.CSSProperties = { color: '#ccc', lineHeight: 1.7, margin: '0 0 12px 0' };

/* ── Getting Started ────────────────────────────────────────────────── */

function GettingStarted() {
  return (
    <>
      <SectionTitle>Getting Started</SectionTitle>

      <Card title="Welcome to Infon Battle Arena">
        <p style={textStyle}>
          Infon is a competitive programming game where you write Lua scripts to control swarms
          of creatures. Your bots eat food, grow, fight enemies, and compete for territory --
          all autonomously based on the code you write.
        </p>
      </Card>

      <Card title="Quick Start">
        <div style={{ display: 'flex', alignItems: 'flex-start', marginBottom: 16 }}>
          <StepNumber n={1} />
          <div>
            <strong style={{ color: '#e0e0e0' }}>Create an Account</strong>
            <p style={{ ...textStyle, marginTop: 4 }}>
              Register for a free account to start creating bots and competing.
            </p>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'flex-start', marginBottom: 16 }}>
          <StepNumber n={2} />
          <div>
            <strong style={{ color: '#e0e0e0' }}>Create a Bot</strong>
            <p style={{ ...textStyle, marginTop: 4 }}>
              Go to the Bot Library and click "New Bot". Give it a name and description.
            </p>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'flex-start', marginBottom: 16 }}>
          <StepNumber n={3} />
          <div>
            <strong style={{ color: '#e0e0e0' }}>Write Your Code</strong>
            <p style={{ ...textStyle, marginTop: 4 }}>
              Open the Editor and write Lua code. The simplest bot uses the Object-Oriented API:
            </p>
            <CodeBlock>{`function Creature:main()
    while true do
        if self:tile_food() > 0 then
            self:eat()
        else
            local x1, y1, x2, y2 = world_size()
            self:moveto(math.random(x1, x2), math.random(y1, y2))
        end
    end
end`}</CodeBlock>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'flex-start', marginBottom: 16 }}>
          <StepNumber n={4} />
          <div>
            <strong style={{ color: '#e0e0e0' }}>Save a Version</strong>
            <p style={{ ...textStyle, marginTop: 4 }}>
              Click "Save Version" in the editor. Each save creates a new version you can use in matches.
            </p>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'flex-start' }}>
          <StepNumber n={5} />
          <div>
            <strong style={{ color: '#e0e0e0' }}>Start a Match</strong>
            <p style={{ ...textStyle, marginTop: 4 }}>
              Go to the Game page, select your bot and an opponent, pick a map, and start a match.
              Watch the game unfold in real-time with the built-in viewer.
            </p>
          </div>
        </div>
      </Card>

      <Card title="Two API Styles">
        <p style={textStyle}>
          Infon supports two Lua API styles. Pick whichever feels more natural:
        </p>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
          <div>
            <h4 style={{ color: '#f5a623', marginTop: 0, marginBottom: 8, fontSize: 14 }}>
              Object-Oriented (OO)
            </h4>
            <p style={{ ...textStyle, fontSize: 13 }}>
              Define <code style={{ color: '#16c79a' }}>Creature:main()</code> as a coroutine.
              Use blocking methods like <code style={{ color: '#16c79a' }}>self:eat()</code> and{' '}
              <code style={{ color: '#16c79a' }}>self:moveto(x, y)</code>.
            </p>
          </div>
          <div>
            <h4 style={{ color: '#f5a623', marginTop: 0, marginBottom: 8, fontSize: 14 }}>
              State Machine
            </h4>
            <p style={{ ...textStyle, fontSize: 13 }}>
              Define <code style={{ color: '#16c79a' }}>bot()</code> with state functions and
              event handlers like <code style={{ color: '#16c79a' }}>onIdle()</code> and{' '}
              <code style={{ color: '#16c79a' }}>onTileFood()</code>.
            </p>
          </div>
        </div>
      </Card>
    </>
  );
}

/* ── Lua API Reference ──────────────────────────────────────────────── */

function LuaApiReference({ content, loading, error }: {
  content: string;
  loading: boolean;
  error: string | null;
}) {
  if (loading) return <div style={{ padding: 24, color: '#888' }}>Loading Lua API documentation...</div>;
  if (error) return <div style={{ padding: 24, color: '#e94560' }}>{error}</div>;

  // Parse markdown into styled sections
  return (
    <>
      <SectionTitle>Lua API Reference</SectionTitle>
      <Card>
        <div style={{ color: '#ccc', lineHeight: 1.7 }}>
          <MarkdownContent content={content} />
        </div>
      </Card>
    </>
  );
}

/** Simple markdown-to-JSX renderer for the Lua API docs */
function MarkdownContent({ content }: { content: string }) {
  const lines = content.split('\n');
  const elements: React.ReactNode[] = [];
  let i = 0;
  let key = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Code block
    if (line.startsWith('```')) {
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].startsWith('```')) {
        codeLines.push(lines[i]);
        i++;
      }
      i++; // skip closing ```
      elements.push(<CodeBlock key={key++}>{codeLines.join('\n')}</CodeBlock>);
      continue;
    }

    // Headers
    if (line.startsWith('### ')) {
      elements.push(
        <h4 key={key++} style={{ color: '#f5a623', marginTop: 20, marginBottom: 8, fontSize: 15 }}>
          {line.replace('### ', '')}
        </h4>
      );
      i++;
      continue;
    }
    if (line.startsWith('## ')) {
      elements.push(
        <h3 key={key++} style={{ color: '#16c79a', marginTop: 28, marginBottom: 12, fontSize: 18 }}>
          {line.replace('## ', '')}
        </h3>
      );
      i++;
      continue;
    }
    if (line.startsWith('# ')) {
      // Skip top-level header (we have our own)
      i++;
      continue;
    }

    // Horizontal rule
    if (line.match(/^---+$/)) {
      elements.push(<hr key={key++} style={{ border: 'none', borderTop: '1px solid #333', margin: '20px 0' }} />);
      i++;
      continue;
    }

    // Table
    if (line.includes('|') && i + 1 < lines.length && lines[i + 1].includes('---')) {
      const tableLines: string[] = [line];
      i++;
      while (i < lines.length && lines[i].includes('|')) {
        tableLines.push(lines[i]);
        i++;
      }
      elements.push(<MarkdownTable key={key++} lines={tableLines} />);
      continue;
    }

    // Empty line
    if (line.trim() === '') {
      i++;
      continue;
    }

    // Regular paragraph (may contain inline formatting)
    elements.push(
      <p key={key++} style={{ ...textStyle }}>
        <InlineMarkdown text={line} />
      </p>
    );
    i++;
  }

  return <>{elements}</>;
}

function InlineMarkdown({ text }: { text: string }) {
  // Handle **bold**, `code`, and plain text
  const parts: React.ReactNode[] = [];
  let remaining = text;
  let k = 0;

  while (remaining.length > 0) {
    // Bold
    const boldMatch = remaining.match(/^(.*?)\*\*(.*?)\*\*(.*)/s);
    if (boldMatch) {
      if (boldMatch[1]) parts.push(<InlineMarkdown key={k++} text={boldMatch[1]} />);
      parts.push(<strong key={k++} style={{ color: '#e0e0e0' }}>{boldMatch[2]}</strong>);
      remaining = boldMatch[3];
      continue;
    }

    // Inline code
    const codeMatch = remaining.match(/^(.*?)`(.*?)`(.*)/s);
    if (codeMatch) {
      if (codeMatch[1]) parts.push(<span key={k++}>{codeMatch[1]}</span>);
      parts.push(
        <code key={k++} style={{
          color: '#16c79a',
          background: 'rgba(22,199,154,0.1)',
          padding: '1px 5px',
          borderRadius: 3,
          fontSize: '0.9em',
        }}>
          {codeMatch[2]}
        </code>
      );
      remaining = codeMatch[3];
      continue;
    }

    // Plain text
    parts.push(<span key={k++}>{remaining}</span>);
    break;
  }

  return <>{parts}</>;
}

function MarkdownTable({ lines }: { lines: string[] }) {
  const parseRow = (line: string) =>
    line.split('|').map(c => c.trim()).filter(c => c.length > 0);

  const headers = parseRow(lines[0]);
  // Skip separator line (index 1)
  const rows = lines.slice(2).map(parseRow);

  return (
    <div style={{ overflowX: 'auto', margin: '12px 0' }}>
      <table style={{
        width: '100%',
        borderCollapse: 'collapse',
        fontSize: 13,
      }}>
        <thead>
          <tr>
            {headers.map((h, i) => (
              <th key={i} style={{
                textAlign: 'left',
                padding: '8px 12px',
                borderBottom: '2px solid #333',
                color: '#f5a623',
                fontWeight: 600,
              }}>
                <InlineMarkdown text={h} />
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, ri) => (
            <tr key={ri}>
              {row.map((cell, ci) => (
                <td key={ci} style={{
                  padding: '6px 12px',
                  borderBottom: '1px solid #1a3a5c',
                  color: '#ccc',
                }}>
                  <InlineMarkdown text={cell} />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/* ── Strategy Guide ─────────────────────────────────────────────────── */

function StrategyGuide() {
  return (
    <>
      <SectionTitle>Strategy Guide</SectionTitle>

      <Card title="Creature Types">
        <p style={textStyle}>
          Understanding the three creature types is fundamental to success:
        </p>
        <div style={{ marginBottom: 16 }}>
          <h4 style={{ color: '#f5a623', marginTop: 0, marginBottom: 4, fontSize: 14 }}>
            Type 0 - Small (Balanced)
          </h4>
          <p style={{ ...textStyle, fontSize: 13 }}>
            Your starting type. Fast enough, decent food capacity (10,000). Can only attack Flyers.
            Convert to Big when you have enough food (8,000) to start spawning, or to Flyer (5,000)
            for scouting.
          </p>
        </div>
        <div style={{ marginBottom: 16 }}>
          <h4 style={{ color: '#f5a623', marginTop: 0, marginBottom: 4, fontSize: 14 }}>
            Type 1 - Big (Tank)
          </h4>
          <p style={{ ...textStyle, fontSize: 13 }}>
            The only type that can spawn new creatures (Type 0). High HP (20,000) and attacks
            everything for 1,500 damage. Essential for growing your swarm. Costs 5,000 food + 20% HP to spawn.
          </p>
        </div>
        <div>
          <h4 style={{ color: '#f5a623', marginTop: 0, marginBottom: 4, fontSize: 14 }}>
            Type 2 - Flyer (Scout)
          </h4>
          <p style={{ ...textStyle, fontSize: 13 }}>
            Fastest type (800 units/sec) and can fly over walls. Cannot attack at all.
            Great for scouting food and feeding other creatures. Vulnerable to Small attacks.
          </p>
        </div>
      </Card>

      <Card title="Food Management">
        <ul style={{ color: '#ccc', lineHeight: 2, paddingLeft: 20, margin: 0 }}>
          <li>Always prioritize eating when food is on your tile -- do not waste it.</li>
          <li>Heal before eating if health is low; dead creatures eat nothing.</li>
          <li>Creatures drain health constantly. Without eating, they starve and die.</li>
          <li>Use Flyers to scout for food-rich areas, then send Smalls to eat.</li>
          <li>Feed struggling allies using the FEED action (max 256 unit range, 400 food/sec).</li>
        </ul>
      </Card>

      <Card title="Attacking vs Defending">
        <ul style={{ color: '#ccc', lineHeight: 2, paddingLeft: 20, margin: 0 }}>
          <li>Only Big creatures can attack ground units. Convert to Big before engaging.</li>
          <li>Small creatures are only useful against Flyers (1,000 damage in 768 range).</li>
          <li>
            Big vs Big fights are symmetrical (1,500 dmg each). Win by having more food/health going in.
          </li>
          <li>Avoid fights when low on food -- you need food to heal afterward.</li>
          <li>King of the Hill requires IDLE state. Guard your king with nearby Bigs.</li>
        </ul>
      </Card>

      <Card title="Growth Strategy">
        <ol style={{ color: '#ccc', lineHeight: 2, paddingLeft: 20, margin: 0 }}>
          <li>Start by eating food as Small creatures to build reserves.</li>
          <li>Convert to Big (8,000 food) when you have enough.</li>
          <li>Spawn new Smalls from your Big (costs 5,000 food + 20% HP).</li>
          <li>New Smalls repeat the cycle: eat, convert, spawn.</li>
          <li>Exponential growth is the key to domination.</li>
        </ol>
      </Card>

      <Card title="King of the Hill Tips">
        <ul style={{ color: '#ccc', lineHeight: 2, paddingLeft: 20, margin: 0 }}>
          <li>
            Use <code style={{ color: '#16c79a' }}>get_koth_pos()</code> to find the KotH tile.
          </li>
          <li>A creature must be IDLE on the tile to score. Moving or eating does not count.</li>
          <li>Keep a well-fed Big nearby to defend your king creature.</li>
          <li>
            Check <code style={{ color: '#16c79a' }}>king_player()</code> to see who currently holds the hill.
          </li>
        </ul>
      </Card>
    </>
  );
}

/* ── FAQ / Troubleshooting ──────────────────────────────────────────── */

function FAQ() {
  const faqs: { q: string; a: React.ReactNode }[] = [
    {
      q: 'Why isn\'t my bot moving?',
      a: (
        <>
          <p style={textStyle}>Common causes:</p>
          <ul style={{ color: '#ccc', lineHeight: 1.8, paddingLeft: 20, margin: 0 }}>
            <li>
              You are not calling <code style={{ color: '#16c79a' }}>self:moveto(x, y)</code> or{' '}
              <code style={{ color: '#16c79a' }}>set_path(id, x, y)</code>.
            </li>
            <li>The destination is inside a wall (TILE_SOLID). Check with <code style={{ color: '#16c79a' }}>get_tile_type()</code>.</li>
            <li>Your creature is in a different state (eating, healing, etc.). Set state to WALK first.</li>
            <li>Your main loop exited. Use <code style={{ color: '#16c79a' }}>while true do ... end</code>.</li>
          </ul>
        </>
      ),
    },
    {
      q: 'I get a Lua error "attempt to call a nil value"',
      a: (
        <p style={textStyle}>
          This usually means you misspelled a function name, or you are using the State API functions
          (like <code style={{ color: '#16c79a' }}>eat()</code>) in an OO-style bot. In the OO API,
          use <code style={{ color: '#16c79a' }}>self:eat()</code>. Check the API Reference for the correct function names.
        </p>
      ),
    },
    {
      q: 'My creatures keep dying of starvation',
      a: (
        <p style={textStyle}>
          All creatures continuously lose health (50-70 HP/sec depending on type). You need to
          eat food and heal regularly. Prioritize eating when food is available, and heal when health
          drops below 50%.
        </p>
      ),
    },
    {
      q: 'How do I spawn new creatures?',
      a: (
        <p style={textStyle}>
          Only Type 1 (Big) creatures can spawn. Convert a Type 0 to Type 1 using{' '}
          <code style={{ color: '#16c79a' }}>self:convert(1)</code> (costs 8,000 food), then call{' '}
          <code style={{ color: '#16c79a' }}>self:spawn()</code> (costs 5,000 food + 20% health).
          The new creature will be Type 0.
        </p>
      ),
    },
    {
      q: 'What\'s the difference between the OO and State API?',
      a: (
        <p style={textStyle}>
          Both achieve the same thing. The OO API uses <code style={{ color: '#16c79a' }}>Creature:main()</code> as
          a coroutine with blocking methods (moveto, eat, etc.). The State API uses{' '}
          <code style={{ color: '#16c79a' }}>bot()</code> with named state functions and event callbacks.
          Choose whichever feels more natural to you.
        </p>
      ),
    },
    {
      q: 'How do I attack other players\' creatures?',
      a: (
        <>
          <p style={textStyle}>
            Use <code style={{ color: '#16c79a' }}>self:attack(target)</code> (OO) or{' '}
            <code style={{ color: '#16c79a' }}>attack(target)</code> (State). Note:
          </p>
          <ul style={{ color: '#ccc', lineHeight: 1.8, paddingLeft: 20, margin: 0 }}>
            <li>Type 0 (Small) can only attack Type 2 (Flyers).</li>
            <li>Type 1 (Big) can attack everything.</li>
            <li>Type 2 (Flyers) cannot attack at all.</li>
          </ul>
          <p style={{ ...textStyle, marginTop: 8 }}>
            Find enemies with <code style={{ color: '#16c79a' }}>self:nearest_enemy()</code>.
          </p>
        </>
      ),
    },
    {
      q: 'My bot code seems correct but nothing happens',
      a: (
        <p style={textStyle}>
          Make sure you saved a version (not just edited code). The game uses the saved bot version,
          not the editor contents. Also verify your bot has no syntax errors using the editor's
          validation feature.
        </p>
      ),
    },
    {
      q: 'Can I use external Lua libraries?',
      a: (
        <p style={textStyle}>
          No. Bots run in a sandboxed Lua 5.1 environment. Standard Lua libraries (math, string, table)
          are available, but you cannot require external modules. File I/O, OS access, and networking
          are disabled for security.
        </p>
      ),
    },
  ];

  return (
    <>
      <SectionTitle>FAQ / Troubleshooting</SectionTitle>

      {faqs.map((faq, i) => (
        <Card key={i}>
          <h4 style={{ color: '#f5a623', marginTop: 0, marginBottom: 12, fontSize: 15 }}>
            {faq.q}
          </h4>
          {faq.a}
        </Card>
      ))}
    </>
  );
}
