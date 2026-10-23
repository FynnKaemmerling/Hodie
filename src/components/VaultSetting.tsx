import { useEffect, useState } from "react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useTranslation } from "react-i18next";
import { api, type VaultInfo } from "../api";
import { errorMessageKey } from "../api/errors";
import { chooseVaultFolder, switchVault } from "../vault";
import styles from "./VaultSetting.module.css";

/** Shows the vault folder and opens another one. */
function VaultSetting() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<VaultInfo | null>(null);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    api.getVault().then(setInfo, (error) => setFailure(t(errorMessageKey(error))));
  }, [t]);

  async function choose() {
    setFailure(null);
    try {
      const path = await chooseVaultFolder(t("settings.vault.dialogTitle"), info?.path ?? null);
      // On success every window reloads with the new vault.
      if (path && path !== info?.path) await switchVault(path);
    } catch (error) {
      setFailure(t(errorMessageKey(error)));
    }
  }

  async function reveal(path: string) {
    try {
      await revealItemInDir(path);
    } catch {
      setFailure(t("settings.vault.revealFailed"));
    }
  }

  return (
    <div className={styles.setting}>
      <div className={styles.row}>
        <span className={styles.label}>{t("settings.vault.label")}</span>
        <span className={styles.path} title={info?.path ?? undefined}>
          <bdi>{info?.path ?? t("settings.info.unknown")}</bdi>
        </span>
      </div>

      <div className={styles.actions}>
        {info?.path && (
          <button type="button" className={styles.link} onClick={() => void reveal(info.path!)}>
            {t("settings.vault.reveal")}
          </button>
        )}
        <button type="button" className={styles.link} onClick={() => void choose()}>
          {t("settings.vault.choose")}
        </button>
      </div>

      <p className={styles.message} role="status">
        {failure && <span className={styles.error}>{failure}</span>}
      </p>
    </div>
  );
}

export default VaultSetting;
