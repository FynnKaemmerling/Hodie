import { X } from "lucide-react";
import {
  Button,
  Dialog,
  Heading,
  Modal,
  ModalOverlay,
  Tab,
  TabList,
  TabPanel,
  Tabs,
} from "react-aria-components";
import { useTranslation } from "react-i18next";
import type { Id, Tag, TagColor, TagPatch } from "../api";
import AppInfo from "./AppInfo";
import ColorPresets from "./ColorPresets";
import Preferences from "./Preferences";
import ShortcutSetting from "./ShortcutSetting";
import TagManager from "./TagManager";
import VaultSetting from "./VaultSetting";
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

          <Tabs className={styles.tabs}>
            <TabList className={styles.tabList} aria-label={t("settings.title")}>
              <Tab id="general" className={styles.tab}>
                {t("settings.general")}
              </Tab>
              <Tab id="colors" className={styles.tab}>
                {t("settings.colors.title")}
              </Tab>
              <Tab id="tags" className={styles.tab}>
                {t("settings.tags.title")}
              </Tab>
              <Tab id="info" className={styles.tab}>
                {t("settings.info.title")}
              </Tab>
            </TabList>

            <TabPanel id="general" className={styles.panel}>
              <section className={styles.section} aria-labelledby="settings-general">
                <h3 id="settings-general" className={styles.sectionTitle}>
                  {t("settings.general")}
                </h3>
                <Preferences />
              </section>

              <section className={styles.section} aria-labelledby="settings-vault">
                <h3 id="settings-vault" className={styles.sectionTitle}>
                  {t("settings.vault.title")}
                </h3>
                <p className={styles.description}>{t("settings.vault.description")}</p>
                <VaultSetting />
              </section>

              <section className={styles.section} aria-labelledby="settings-shortcut">
                <h3 id="settings-shortcut" className={styles.sectionTitle}>
                  {t("settings.shortcut.title")}
                </h3>
                <p className={styles.description}>{t("settings.shortcut.description")}</p>
                <ShortcutSetting />
              </section>
            </TabPanel>

            <TabPanel id="colors" className={styles.panel}>
              <section className={styles.section}>
                <p className={styles.description}>{t("settings.colors.description")}</p>
                <ColorPresets />
              </section>
            </TabPanel>

            <TabPanel id="tags" className={styles.panel}>
              <section className={styles.section}>
                <p className={styles.description}>{t("settings.tags.description")}</p>
                <TagManager
                  tags={tags}
                  onCreate={onCreateTag}
                  onUpdate={onUpdateTag}
                  onDelete={onDeleteTag}
                />
              </section>
            </TabPanel>

            <TabPanel id="info" className={styles.panel}>
              <section className={styles.section}>
                <AppInfo />
              </section>
            </TabPanel>
          </Tabs>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}

export default SettingsDialog;
