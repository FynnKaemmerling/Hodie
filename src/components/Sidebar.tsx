import { useState } from "react";
import {
  closestCenter,
  DndContext,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type Announcements,
  type DragEndEvent,
  type UniqueIdentifier,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Archive, GripVertical, Plus, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Board, Id } from "../api";
import logo from "../../src-tauri/icons/app-icon.svg";
import ActionMenu, { useContextMenu, type ActionMenuItem } from "./ActionMenu";
import InlineInput from "./InlineInput";
import styles from "./Sidebar.module.css";

interface Props {
  boards: Board[];
  selectedId: Id | null;
  onSelect: (id: Id) => void;
  onCreate: (name: string) => void;
  onRename: (id: Id, name: string) => void;
  onReorder: (id: Id, index: number) => void;
  onDelete: (board: Board) => void;
  showingArchive: boolean;
  onShowArchive: () => void;
  onOpenSettings: () => void;
}

function Sidebar({
  boards,
  selectedId,
  onSelect,
  onCreate,
  onRename,
  onReorder,
  onDelete,
  showingArchive,
  onShowArchive,
  onOpenSettings,
}: Props) {
  const { t } = useTranslation();
  const [creating, setCreating] = useState(false);
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  function nameOf(id: UniqueIdentifier) {
    return boards.find((board) => board.id === id)?.name ?? "";
  }

  function positionOf(id: UniqueIdentifier) {
    const index = boards.findIndex((board) => board.id === id);
    return t("dnd.listPosition", { position: index + 1, total: boards.length });
  }

  const announcements: Announcements = {
    onDragStart: ({ active }) => t("dnd.pickedUp", { name: nameOf(active.id) }),
    onDragOver: ({ active, over }) =>
      over
        ? t("dnd.over", { name: nameOf(active.id), target: positionOf(over.id) })
        : t("dnd.notOver", { name: nameOf(active.id) }),
    onDragEnd: ({ active }) => t("dnd.dropped", { name: nameOf(active.id) }),
    onDragCancel: ({ active }) => t("dnd.cancelled", { name: nameOf(active.id) }),
  };

  function handleDragEnd({ active, over }: DragEndEvent) {
    if (!over || over.id === active.id) return;
    const index = boards.findIndex((board) => board.id === over.id);
    if (index !== -1) onReorder(Number(active.id), index);
  }

  return (
    <nav className={styles.sidebar} aria-label={t("nav.title")}>
      <div className={styles.header} data-tauri-drag-region>
        <img className={styles.logo} src={logo} alt="" data-tauri-drag-region />
        <span className={styles.appName} data-tauri-drag-region>
          {t("app.name")}
        </span>
      </div>

      <h2 className={styles.sectionTitle}>{t("nav.boards")}</h2>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
        accessibility={{
          announcements,
          screenReaderInstructions: { draggable: t("dnd.instructions") },
        }}
      >
        <SortableContext items={boards.map((board) => board.id)} strategy={verticalListSortingStrategy}>
          <ul className={styles.items}>
            {boards.map((board) => (
              <BoardRow
                key={board.id}
                board={board}
                current={board.id === selectedId && !showingArchive}
                onSelect={onSelect}
                onRename={onRename}
                onDelete={onDelete}
              />
            ))}
          </ul>
        </SortableContext>
      </DndContext>

      {creating ? (
        <InlineInput
          label={t("board.name")}
          placeholder={t("board.namePlaceholder")}
          onSubmit={onCreate}
          onClose={() => setCreating(false)}
        />
      ) : (
        <button type="button" className={styles.add} onClick={() => setCreating(true)}>
          <Plus aria-hidden="true" /> {t("board.add")}
        </button>
      )}

      {selectedId !== null && (
        <button
          type="button"
          className={`${styles.item} ${styles.archive}`}
          aria-current={showingArchive ? "page" : undefined}
          onClick={onShowArchive}
        >
          <Archive aria-hidden="true" />
          {t("nav.archive")}
        </button>
      )}

      <div className={styles.footer}>
        <button type="button" className={styles.add} onClick={onOpenSettings}>
          <Settings aria-hidden="true" /> {t("settings.open")}
        </button>
      </div>
    </nav>
  );
}

interface RowProps {
  board: Board;
  current: boolean;
  onSelect: (id: Id) => void;
  onRename: (id: Id, name: string) => void;
  onDelete: (board: Board) => void;
}

/** One board: a drag handle, the board itself and its menu. */
function BoardRow({ board, current, onSelect, onRename, onDelete }: RowProps) {
  const { t } = useTranslation();
  const [renaming, setRenaming] = useState(false);
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } =
    useSortable({ id: board.id, disabled: renaming });
  const menuItems: ActionMenuItem[] = [
    { id: "rename", label: t("actions.rename"), onAction: () => setRenaming(true) },
    {
      id: "delete",
      label: t("actions.delete"),
      danger: true,
      onAction: () => onDelete(board),
    },
  ];
  const contextMenu = useContextMenu(menuItems);

  return (
    <li
      ref={setNodeRef}
      onContextMenu={contextMenu.onContextMenu}
      className={styles.row}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      data-dragging={isDragging || undefined}
    >
      {renaming ? (
        <InlineInput
          label={t("board.name")}
          initialValue={board.name}
          onSubmit={(name) => onRename(board.id, name)}
          onClose={() => setRenaming(false)}
        />
      ) : (
        <>
          <button
            type="button"
            ref={setActivatorNodeRef}
            className={styles.handle}
            {...attributes}
            {...listeners}
            aria-label={t("board.move", { name: board.name })}
          >
            <GripVertical aria-hidden="true" />
          </button>
          <button
            type="button"
            className={styles.item}
            aria-current={current ? "page" : undefined}
            onClick={() => onSelect(board.id)}
            onDoubleClick={() => setRenaming(true)}
            onKeyDown={(event) => {
              if (event.key === "F2") {
                event.preventDefault();
                setRenaming(true);
              }
            }}
          >
            {board.name}
          </button>
          <div className={styles.menu}>
            <ActionMenu
              label={t("actions.more", { name: board.name })}
              items={menuItems}
            />
          </div>
        </>
      )}
      {contextMenu.menu}
    </li>
  );
}

export default Sidebar;
