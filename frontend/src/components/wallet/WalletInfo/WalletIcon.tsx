import { FC } from "react";
import cn from "classnames";
import metamaskIcon from "../../../assets/icons/metamask.svg";
import nearIcon from "../../../assets/icons/near-green.svg";
import styles from "./WalletIcon.module.scss";

interface WalletIconProps {
  type: "metamask" | "near";
  size?: "sm" | "md" | "lg";
  className?: string;
}

export const WalletIcon: FC<WalletIconProps> = ({
  type,
  size = "sm",
  className = "",
}) => {
  const getIconSrc = () => {
    switch (type) {
      case "metamask":
        return metamaskIcon;
      case "near":
        return nearIcon;
      default:
        return "";
    }
  };

  return (
    <img
      src={getIconSrc()}
      alt={`${type} wallet`}
      className={cn(styles.icon, styles[size], className)}
    />
  );
};
