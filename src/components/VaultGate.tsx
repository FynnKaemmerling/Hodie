import { useEffect, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api, type VaultInfo } from "../api";
import { errorMessageKey } from "../api/errors";
import { chooseVaultFolder, retryVault, switchVault } from "../vault";
import WindowControls from "./WindowControls";
import styles from "./VaultGate.module.css";

/** Shows the app once the vault is open; otherwise why not and the ways out. */
function VaultGate({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<VaultInfo | null>(null);

  useEffect(() => {
    api.getVault().then(setInfo, (error) => setInfo({ path: null, error: String(error) }));
  }, []);

  if (!info) return null;
  if (info.error === null) return children;
  return <VaultMissing info={info} onInfo={setInfo} />;
}

function VaultMissing({ info, onInfo }: { info: VaultInfo; onInfo: (info: VaultInfo) => void }) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);

  async function run(action: () => Promise<void>) {
    setBusy(true);
    setFailure(null);
    try {
      await action();
    } catch (error) {
      setFailure(t(errorMessageKey(error)));
    } finally {
      setBusy(false);
    }
  }

  const retry = () =>
    run(async () => {
      const next = await retryVault();
      onInfo(next);
      if (next.error !== null) setFailure(t("vault.missing.stillMissing"));
    });

  const choose = () =>
    run(async () => {
      const path = await chooseVaultFolder(t("settings.vault.dialogTitle"), info.path);
      if (path) await switchVault(path);
    });

  return (
    <div className={styles.screen}>
      <header className={styles.toolbar} data-tauri-drag-region>
        <span className={styles.spacer} data-tauri-drag-region />
        <WindowControls />
      </header>
      <main className={styles.content}>
        <h1 className={styles.title}>{t("vault.missing.title")}</h1>
        <p className={styles.text}>{t(errorMessageKey(info.error))}</p>
        {info.path && <p className={styles.path}>{info.path}</p>}
        <div className={styles.actions}>
          <button type="button" className={styles.primary} disabled={busy} onClick={retry}>
            {t("vault.missing.retry")}
          </button>
          <button type="button" className={styles.secondary} disabled={busy} onClick={choose}>
            {t("vault.missing.choose")}
          </button>
        </div>
        <p className={styles.failure} role="status">
          {failure}
        </p>
      </main>
    </div>
  );
}

export default VaultGate;
