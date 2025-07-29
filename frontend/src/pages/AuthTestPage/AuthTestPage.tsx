/**
 * Authentication Test Page
 * Demonstrates new authentication components and flow
 */

import React from "react";
import { AuthStatus } from "../../components/auth";
import "./AuthTestPage.scss";

export const AuthTestPage: React.FC = () => {
  return (
    <div className="auth-test-page">
      <div className="auth-test-page__container">
        <header className="auth-test-page__header">
          <h1>Authentication Components Demo</h1>
          <p>Test the custom authentication UI components</p>
        </header>

        <div className="auth-test-page__content">
          {/* Authentication Status */}
          <section className="auth-test-page__section">
            <h2>Authentication Status</h2>
            <div className="auth-test-page__status-demo">
              <div className="status-example">
                <AuthStatus showFullStatus />
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
};
