import { BrowserRouter, Routes, Route, NavLink } from 'react-router-dom';
import { BotLibrary } from './pages/BotLibrary';
import { BotEditor } from './pages/BotEditor';
import { TournamentList } from './pages/TournamentList';
import { TournamentDetail } from './pages/TournamentDetail';
import { GameViewer } from './pages/GameViewer';
import './App.css';

function App() {
  return (
    <BrowserRouter>
      <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
        <nav className="app-nav">
          <h1 className="app-title">Infon Arena</h1>
          <NavLink to="/" end className={navLinkClass}>Bot Library</NavLink>
          <NavLink to="/editor" className={navLinkClass}>Editor</NavLink>
          <NavLink to="/tournaments" className={navLinkClass}>Tournaments</NavLink>
          <NavLink to="/game" className={navLinkClass}>Game</NavLink>
        </nav>
        <main style={{ flex: 1, overflow: 'auto', display: 'flex', flexDirection: 'column' }}>
          <Routes>
            <Route path="/" element={<BotLibrary />} />
            <Route path="/editor" element={<BotEditor />} />
            <Route path="/editor/:botId" element={<BotEditor />} />
            <Route path="/tournaments" element={<TournamentList />} />
            <Route path="/tournaments/:id" element={<TournamentDetail />} />
            <Route path="/game" element={<GameViewer />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}

function navLinkClass({ isActive }: { isActive: boolean }): string {
  return isActive ? 'nav-link nav-link-active' : 'nav-link';
}

export default App;
