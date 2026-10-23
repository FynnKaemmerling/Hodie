import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import { modShortcut } from "../platform";
import styles from "./SidebarToggle.module.css";

interface Props {
  collapsed: boolean;
  onToggle: () => void;
}

/** Collapses the sidebar from its header, or brings it back from a toolbar. */
function SidebarToggle({ collapsed, onToggle }: Props) {
  const { t } = useTranslation();
  const label = collapsed ? t("nav.expand") : t("nav.collapse");
  const Icon = collapsed ? PanelLeftOpen : PanelLeftClose;

  return (
    <button
      type="button"
      className={styles.toggle}
      data-collapsed={collapsed || undefined}
      aria-label={label}
      aria-expanded={!collapsed}
      title={`${label} (${modShortcut(t, "B")})`}
      onClick={onToggle}
    >
      <Icon aria-hidden="true" />
    </button>
  );
}

export default SidebarToggle;
