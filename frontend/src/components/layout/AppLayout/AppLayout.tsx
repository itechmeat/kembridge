import { FC, ReactNode } from "react";
import cn from "classnames";
import { ErrorBoundary } from "../../ui/ErrorBoundary/ErrorBoundary";
import styles from "./AppLayout.module.scss";

interface AppLayoutProps {
  children: ReactNode;
  className?: string;
}

export const AppLayout: FC<AppLayoutProps> = ({ children, className = "" }) => {
  return (
    <ErrorBoundary>
      <div className={cn(styles.appLayout, className?.trim())}>
        <main className={styles.main}>{children}</main>
      </div>
    </ErrorBoundary>
  );
};
