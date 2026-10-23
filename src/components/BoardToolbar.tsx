import { Check, FunnelX, ListFilter, Search } from "lucide-react";
import { useState, type ReactNode, type RefObject } from "react";
import {
  Button,
  Checkbox,
  CheckboxGroup,
  Dialog,
  DialogTrigger,
  Label,
  Popover,
} from "react-aria-components";
import { useTranslation } from "react-i18next";
import type { Priority, Tag } from "../api";
import { isFilterActive, NO_FILTER, type CardFilter } from "../board/filter";
import { modShortcut } from "../platform";
import { iconButtonClass } from "./IconButton";
import IconButton from "./IconButton";
import TagChip from "./TagChip";
import styles from "./BoardToolbar.module.css";

const PRIORITIES: Priority[] = ["high", "medium", "low", "none"];

interface Props {
  title: string;
  tags: Tag[];
  filter: CardFilter;
  onFilterChange: (filter: CardFilter) => void;
  /** Visible and total cards while a filter is active. */
  counts: { visible: number; total: number } | null;
  searchRef: RefObject<HTMLInputElement | null>;
  /** Shown before the title, e.g. the button that brings back the sidebar. */
  leading?: ReactNode;
  /** Shown after the filter, e.g. the window controls. */
  trailing?: ReactNode;
}

function BoardToolbar({
  title,
  tags,
  filter,
  onFilterChange,
  counts,
  searchRef,
  leading,
  trailing,
}: Props) {
  const { t } = useTranslation();
  const choices = filter.tagIds.length + filter.priorities.length;
  // The search shows only its icon until it is focused or holds a query.
  const [searchFocused, setSearchFocused] = useState(false);
  const searchLabel = t("filter.searchPlaceholder", { shortcut: modShortcut(t, "F") });

  return (
    // The free space of the toolbar moves the frameless window.
    <header className={styles.toolbar} data-tauri-drag-region>
      {leading}
      <h1 className={styles.title} data-tauri-drag-region>
        {title}
      </h1>

      {counts && (
        <span className={styles.counts} role="status">
          {t("filter.counts", { visible: counts.visible, total: counts.total })}
        </span>
      )}
      {isFilterActive(filter) && (
        <IconButton label={t("filter.reset")} onClick={() => onFilterChange(NO_FILTER)}>
          <FunnelX aria-hidden="true" />
        </IconButton>
      )}

      <div
        className={styles.search}
        data-expanded={searchFocused || filter.query !== "" || undefined}
        title={searchFocused ? undefined : searchLabel}
      >
        <Search className={styles.searchIcon} aria-hidden="true" />
        <input
          ref={searchRef}
          type="search"
          className={styles.searchInput}
          value={filter.query}
          placeholder={searchLabel}
          aria-label={t("filter.search")}
          onFocus={() => setSearchFocused(true)}
          onBlur={() => setSearchFocused(false)}
          onChange={(event) => onFilterChange({ ...filter, query: event.target.value })}
          onKeyDown={(event) => {
            if (event.key === "Escape" && filter.query !== "") {
              event.preventDefault();
              onFilterChange({ ...filter, query: "" });
            }
          }}
        />
      </div>

      {/* React Aria's Button takes no title, so the tooltip sits around it. */}
      <span className={styles.trigger} title={t("filter.title")}>
        <DialogTrigger>
          <Button
            className={iconButtonClass}
            aria-label={
              choices > 0 ? t("filter.titleActive", { count: choices }) : t("filter.title")
            }
            data-active={choices > 0 || undefined}
          >
            <ListFilter aria-hidden="true" />
            {choices > 0 && (
              <span className={styles.badge} aria-hidden="true">
                {choices}
              </span>
            )}
          </Button>
          <Popover className={styles.popover} placement="bottom end" offset={6}>
            <Dialog className={styles.dialog} aria-label={t("filter.title")}>
              <CheckboxGroup
                className={styles.group}
                value={filter.priorities}
                onChange={(values) => onFilterChange({ ...filter, priorities: values as Priority[] })}
              >
                <Label className={styles.groupLabel}>{t("panel.priority")}</Label>
                {PRIORITIES.map((priority) => (
                  <Checkbox key={priority} value={priority} className={styles.option}>
                    <span className={styles.box} aria-hidden="true">
                      <Check />
                    </span>
                    <span className={styles.swatch} data-priority={priority} aria-hidden="true" />
                    {t(`panel.priorities.${priority}`)}
                  </Checkbox>
                ))}
              </CheckboxGroup>

              <CheckboxGroup
                className={styles.group}
                value={filter.tagIds}
                onChange={(tagIds) => onFilterChange({ ...filter, tagIds })}
              >
                <Label className={styles.groupLabel}>{t("panel.tags")}</Label>
                {tags.length === 0 && <p className={styles.hint}>{t("filter.noTags")}</p>}
                {tags.map((tag) => (
                  <Checkbox key={tag.id} value={String(tag.id)} className={styles.option}>
                    <span className={styles.box} aria-hidden="true">
                      <Check />
                    </span>
                    <TagChip color={tag.color}>{tag.name}</TagChip>
                  </Checkbox>
                ))}
              </CheckboxGroup>
            </Dialog>
          </Popover>
        </DialogTrigger>
      </span>
      {trailing}
    </header>
  );
}

export default BoardToolbar;
