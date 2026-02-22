import { BrowserRouter, Routes, Route, NavLink } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import { BotLibrary } from './pages/BotLibrary';
import { BotEditor } from './pages/BotEditor';
import { TournamentList } from './pages/TournamentList';
import { TournamentDetail } from './pages/TournamentDetail';
import { GameViewer } from './pages/GameViewer';
import { Login } from './pages/Login';
import { Register } from './pages/Register';
import { Leaderboard } from './pages/Leaderboard';
import { ApiKeys } from './pages/ApiKeys';
import { MatchDetail } from './pages/MatchDetail';
import './App.css';

function NavBar() {
  const { user, logout } = useAuth();

  return (
    <nav className="app-nav">
      <h1 className="app-title">Infon Arena</h1>
      <NavLink to="/" end className={navLinkClass}>Bot Library</NavLink>
      <NavLink to="/editor" className={navLinkClass}>Editor</NavLink>
      <NavLink to="/leaderboard" className={navLinkClass}>Leaderboard</NavLink>
      <NavLink to="/tournaments" className={navLinkClass}>Tournaments</NavLink>
      <NavLink to="/game" className={navLinkClass}>Game</NavLink>
      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
        {user ? (
          <>
            <NavLink to="/api-keys" className={navLinkClass}>API Keys</NavLink>
            <span style={{ color: '#aaa', fontSize: 14 }}>{user.username}</span>
            <button onClick={logout} style={{ padding: '4px 12px', fontSize: 13, cursor: 'pointer' }}>
              Logout
            </button>
          </>
        ) : (
          <>
            <NavLink to="/login" className={navLinkClass}>Login</NavLink>
            <NavLink to="/register" className={navLinkClass}>Register</NavLink>
          </>
        )}
      </div>
    </nav>
  );
}

function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
          <NavBar />
          <main style={{ flex: 1, overflow: 'auto', display: 'flex', flexDirection: 'column' }}>
            <Routes>
              <Route path="/" element={<BotLibrary />} />
              <Route path="/editor" element={<BotEditor />} />
              <Route path="/editor/:botId" element={<BotEditor />} />
              <Route path="/leaderboard" element={<Leaderboard />} />
              <Route path="/tournaments" element={<TournamentList />} />
              <Route path="/tournaments/:id" element={<TournamentDetail />} />
              <Route path="/game" element={<GameViewer />} />
              <Route path="/matches/:id" element={<MatchDetail />} />
              <Route path="/api-keys" element={<ApiKeys />} />
              <Route path="/login" element={<Login />} />
              <Route path="/register" element={<Register />} />
            </Routes>
          </main>
        </div>
      </AuthProvider>
    </BrowserRouter>
  );
}

function navLinkClass({ isActive }: { isActive: boolean }): string {
  return isActive ? 'nav-link nav-link-active' : 'nav-link';
}

export default App;
