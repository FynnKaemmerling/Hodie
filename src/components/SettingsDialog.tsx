import { X } from "lucide-react";
import { Button, Dialog, Heading, Modal, ModalOverlay } from "react-aria-components";
import { useTranslation } from "react-i18next";
import type { Id, Tag, TagColor, TagPatch } from "../api";
import Preferences from "./Preferences";
import ShortcutSetting from "./ShortcutSetting";
import TagManager from "./TagManager";
import styles from "./SettingsDialog.module.css";

interface Props {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  tags: Tag[];
  onCreateTag: (name: string, color: TagColor) => void;
  onUpdateTag: (tagId: Id, patch: TagPatch) => void;
  onDeleteTag: (tag: Tag) => void;
}

function SettingsDialog({
  isOpen,
  onOpenChange,
  tags,
  onCreateTag,
  onUpdateTag,
  onDeleteTag,
}: Props) {
  const { t } = useTranslation();

  return (
    <ModalOverlay className={styles.overlay} isOpen={isOpen} onOpenChange={onOpenChange} isDismissable>
      <Modal className={styles.modal}>
        <Dialog className={styles.dialog}>
          <header className={styles.header}>
            <Heading slot="title" className={styles.title}>
              {t("settings.title")}
            </Heading>
            <Button slot="close" className={styles.close} aria-label={t("settings.close")}>
              <X aria-hidden="true" />
            </Button>
          </header>

          <section className={styles.section} aria-labelledby="settings-general">
            <h3 id="settings-general" className={styles.sectionTitle}>
              {t("settings.general")}
            </h3>
            <Preferences />
          </section>

          <section className={styles.section} aria-labelledby="settings-shortcut">
            <h3 id="settings-shortcut" className={styles.sectionTitle}>
              {t("settings.shortcut.title")}
            </h3>
            <p className={styles.description}>{t("settings.shortcut.description")}</p>
            <ShortcutSetting />
          </section>

          <section className={styles.section} aria-labelledby="settings-tags">
            <h3 id="settings-tags" className={styles.sectionTitle}>
              {t("settings.tags.title")}
            </h3>
            <p className={styles.description}>{t("settings.tags.description")}</p>
            <TagManager
              tags={tags}
              onCreate={onCreateTag}
              onUpdate={onUpdateTag}
              onDelete={onDeleteTag}
            />
          </section>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}

export default SettingsDialog;
