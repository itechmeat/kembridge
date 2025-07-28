import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { Button } from "./components/ui/Button";
import { Spinner } from "./components/ui/Spinner";
import "./styles/main.scss";

function App() {
  return (
    <Router>
      <div className="app">
        <Routes>
          <Route path="/" element={<Home />} />
        </Routes>
      </div>
    </Router>
  );
}

function Home() {
  return (
    <div className="home">
      {/* Hero Section */}
      <header className="hero">
        <div className="hero__container">
          <div className="hero__branding">
            <h1 className="hero__title">
              <span className="hero__title-main">KEMBridge</span>
              <span className="hero__title-quantum">⚛️</span>
            </h1>
            <p className="hero__subtitle">
              Cross-Chain Intelligence Meets Quantum Security
            </p>
          </div>

          <div className="hero__description">
            <p className="hero__text">
              The world's first quantum-secured cross-chain bridge powered by
              <strong> ML-KEM-1024</strong> post-quantum cryptography and AI
              risk analysis.
            </p>
          </div>

          <div className="hero__actions">
            <Button variant="primary" size="lg" className="hero__cta">
              Launch Bridge
            </Button>
            <Button variant="secondary" size="lg" className="hero__demo">
              View Demo
            </Button>
          </div>

          <div className="hero__status">
            <div className="status-indicator">
              <Spinner size="sm" color="primary" />
              <span className="status-text">Quantum Protection Active</span>
            </div>
          </div>
        </div>
      </header>

      {/* Features Section */}
      <section className="features">
        <div className="features__container">
          <h2 className="features__title">Revolutionary Security</h2>

          <div className="features__grid">
            <div className="feature-card">
              <div className="feature-card__icon">🔐</div>
              <h3 className="feature-card__title">Quantum-Safe</h3>
              <p className="feature-card__description">
                ML-KEM-1024 post-quantum cryptography protects against future
                quantum attacks
              </p>
            </div>

            <div className="feature-card">
              <div className="feature-card__icon">🤖</div>
              <h3 className="feature-card__title">AI Risk Engine</h3>
              <p className="feature-card__description">
                Real-time transaction analysis with machine learning threat
                detection
              </p>
            </div>

            <div className="feature-card">
              <div className="feature-card__icon">⚡</div>
              <h3 className="feature-card__title">NEAR 1Click</h3>
              <p className="feature-card__description">
                Simplified cross-chain swaps with atomic transaction guarantees
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="stats">
        <div className="stats__container">
          <div className="stats__grid">
            <div className="stat-item">
              <div className="stat-item__value">256-bit</div>
              <div className="stat-item__label">Quantum Security</div>
            </div>
            <div className="stat-item">
              <div className="stat-item__value">&lt;2s</div>
              <div className="stat-item__label">Bridge Time</div>
            </div>
            <div className="stat-item">
              <div className="stat-item__value">99.9%</div>
              <div className="stat-item__label">Uptime</div>
            </div>
            <div className="stat-item">
              <div className="stat-item__value">2</div>
              <div className="stat-item__label">Supported Chains</div>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="footer">
        <div className="footer__container">
          <div className="footer__content">
            <div className="footer__branding">
              <h3>KEMBridge</h3>
              <p>Quantum-secured cross-chain bridge</p>
            </div>

            <div className="footer__links">
              <div className="footer__section">
                <h4>Protocol</h4>
                <ul>
                  <li>
                    <a href="#docs">Documentation</a>
                  </li>
                  <li>
                    <a href="#security">Security</a>
                  </li>
                  <li>
                    <a href="#audits">Audits</a>
                  </li>
                </ul>
              </div>

              <div className="footer__section">
                <h4>Community</h4>
                <ul>
                  <li>
                    <a href="#discord">Discord</a>
                  </li>
                  <li>
                    <a href="#twitter">Twitter</a>
                  </li>
                  <li>
                    <a href="#github">GitHub</a>
                  </li>
                </ul>
              </div>
            </div>
          </div>

          <div className="footer__bottom">
            <p>&copy; 2024 KEMBridge. Quantum-secured future.</p>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
