import { Check, ListFilter, Search } from "lucide-react";
import type { RefObject } from "react";
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
import type { Id, Priority, Tag } from "../api";
import { isFilterActive, NO_FILTER, type CardFilter } from "../board/filter";
import { modShortcut } from "../platform";
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
}

function BoardToolbar({ title, tags, filter, onFilterChange, counts, searchRef }: Props) {
  const { t } = useTranslation();
  const choices = filter.tagIds.length + filter.priorities.length;

  return (
    <header className={styles.toolbar}>
      <h1 className={styles.title}>{title}</h1>

      {counts && (
        <span className={styles.counts} role="status">
          {t("filter.counts", { visible: counts.visible, total: counts.total })}
        </span>
      )}
      {isFilterActive(filter) && (
        <button type="button" className={styles.reset} onClick={() => onFilterChange(NO_FILTER)}>
          {t("filter.reset")}
        </button>
      )}

      <div className={styles.search}>
        <Search className={styles.searchIcon} aria-hidden="true" />
        <input
          ref={searchRef}
          type="search"
          className={styles.searchInput}
          value={filter.query}
          placeholder={t("filter.searchPlaceholder", { shortcut: modShortcut(t, "F") })}
          aria-label={t("filter.search")}
          onChange={(event) => onFilterChange({ ...filter, query: event.target.value })}
          onKeyDown={(event) => {
            if (event.key === "Escape" && filter.query !== "") {
              event.preventDefault();
              onFilterChange({ ...filter, query: "" });
            }
          }}
        />
      </div>

      <DialogTrigger>
        <Button className={styles.filterButton} data-active={choices > 0 || undefined}>
          <ListFilter aria-hidden="true" />
          {t("filter.button")}
          {choices > 0 && <span className={styles.badge}>{choices}</span>}
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
              value={filter.tagIds.map(String)}
              onChange={(values) => onFilterChange({ ...filter, tagIds: values.map(Number) as Id[] })}
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
    </header>
  );
}

export default BoardToolbar;
