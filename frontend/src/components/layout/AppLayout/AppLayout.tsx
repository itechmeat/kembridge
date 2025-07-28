/**
 * Main Application Layout
 * Provides consistent structure for all pages
 */

import { FC, ReactNode } from "react";
import { ErrorBoundary } from "../../ui/ErrorBoundary/ErrorBoundary";
import "./AppLayout.scss";

interface AppLayoutProps {
  children: ReactNode;
  className?: string;
}

export const AppLayout: FC<AppLayoutProps> = ({ children, className = "" }) => {
  return (
    <ErrorBoundary>
      <div className={`app-layout ${className}`}>
        <main className="app-layout__main">{children}</main>
      </div>
    </ErrorBoundary>
  );
};
