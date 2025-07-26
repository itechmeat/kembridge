import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'

function App() {
  return (
    <Router>
      <div className="app">
        <header className="app-header">
          <h1>KEMBridge</h1>
          <p>Cross-Chain Intelligence Meets Quantum Security</p>
        </header>
        <main className="app-main">
          <Routes>
            <Route path="/" element={<Home />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

function Home() {
  return (
    <div className="home">
      <h2>Welcome to KEMBridge</h2>
      <p>Quantum-secured cross-chain bridge coming soon...</p>
    </div>
  )
}

export default App