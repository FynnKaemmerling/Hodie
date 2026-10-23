import { useEffect, useState } from "react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useTranslation } from "react-i18next";
import { api, type VaultInfo, type VaultProblem } from "../api";
import { errorMessageKey } from "../api/errors";
import { chooseVaultFolder, switchVault } from "../vault";
import styles from "./VaultSetting.module.css";

/** Shows the vault folder, files it had to skip, and opens another folder. */
function VaultSetting() {
  const { t } = useTranslation();
  const [info, setInfo] = useState<VaultInfo | null>(null);
  const [problems, setProblems] = useState<VaultProblem[]>([]);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    api.getVault().then(setInfo, (error) => setFailure(t(errorMessageKey(error))));
    api.listVaultProblems().then(setProblems, () => setProblems([]));
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

      {problems.length > 0 && (
        <div className={styles.problems}>
          <p className={styles.problemsTitle}>
            {t("settings.vault.problems", { count: problems.length })}
          </p>
          <ul className={styles.problemList}>
            {problems.map((problem) => (
              <li key={problem.path} className={styles.problem}>
                <span className={styles.problemText}>
                  <bdi className={styles.problemPath}>{problem.path}</bdi>
                  <span className={styles.problemKind}>
                    {problem.kind === "invalid"
                      ? t("settings.vault.invalid")
                      : t("settings.vault.conflictOf", { original: problem.original })}
                  </span>
                </span>
                <button
                  type="button"
                  className={styles.link}
                  onClick={() => void reveal(problem.fullPath)}
                >
                  {t("settings.vault.revealProblem")}
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      <p className={styles.message} role="status">
        {failure && <span className={styles.error}>{failure}</span>}
      </p>
    </div>
  );
}

export default VaultSetting;
