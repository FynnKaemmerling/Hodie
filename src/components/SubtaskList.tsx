import { Check, GripVertical, Plus, X } from "lucide-react";
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
import { Checkbox } from "react-aria-components";
import { useTranslation } from "react-i18next";
import type { Id, Subtask } from "../api";
import InlineInput from "./InlineInput";
import styles from "./SubtaskList.module.css";

interface Props {
  subtasks: Subtask[];
  onAdd: (title: string) => void;
  onToggle: (subtask: Subtask, done: boolean) => void;
  onRename: (subtaskId: Id, title: string) => void;
  onMove: (subtaskId: Id, index: number) => void;
  onDelete: (subtask: Subtask) => void;
}

function SubtaskList({ subtasks, onAdd, onToggle, onRename, onMove, onDelete }: Props) {
  const { t } = useTranslation();
  const [adding, setAdding] = useState(false);
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  function titleOf(id: UniqueIdentifier) {
    return subtasks.find((s) => s.id === id)?.title ?? "";
  }

  function positionOf(id: UniqueIdentifier | undefined) {
    const index = subtasks.findIndex((s) => s.id === id);
    return t("dnd.listPosition", { position: index + 1, total: subtasks.length });
  }

  const announcements: Announcements = {
    onDragStart: ({ active }) => t("dnd.pickedUp", { name: titleOf(active.id) }),
    onDragOver: ({ active, over }) =>
      over
        ? t("dnd.over", { name: titleOf(active.id), target: positionOf(over.id) })
        : t("dnd.notOver", { name: titleOf(active.id) }),
    onDragEnd: ({ active }) => t("dnd.dropped", { name: titleOf(active.id) }),
    onDragCancel: ({ active }) => t("dnd.cancelled", { name: titleOf(active.id) }),
  };

  function handleDragEnd({ active, over }: DragEndEvent) {
    if (!over || over.id === active.id) return;
    const index = subtasks.findIndex((s) => s.id === over.id);
    if (index !== -1) onMove(String(active.id), index);
  }

  return (
    <div>
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
        accessibility={{
          announcements,
          screenReaderInstructions: { draggable: t("dnd.instructions") },
        }}
      >
        <SortableContext items={subtasks.map((s) => s.id)} strategy={verticalListSortingStrategy}>
          <ul className={styles.list}>
            {subtasks.map((subtask) => (
              <SubtaskRow
                key={subtask.id}
                subtask={subtask}
                onToggle={onToggle}
                onRename={onRename}
                onDelete={onDelete}
              />
            ))}
          </ul>
        </SortableContext>
      </DndContext>

      {adding ? (
        <InlineInput
          label={t("subtasks.title")}
          placeholder={t("subtasks.placeholder")}
          keepOpen
          onSubmit={onAdd}
          onClose={() => setAdding(false)}
        />
      ) : (
        <button type="button" className={styles.add} onClick={() => setAdding(true)}>
          <Plus aria-hidden="true" /> {t("subtasks.add")}
        </button>
      )}
    </div>
  );
}

interface RowProps {
  subtask: Subtask;
  onToggle: (subtask: Subtask, done: boolean) => void;
  onRename: (subtaskId: Id, title: string) => void;
  onDelete: (subtask: Subtask) => void;
}

function SubtaskRow({ subtask, onToggle, onRename, onDelete }: RowProps) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const { attributes, listeners, setNodeRef, setActivatorNodeRef, transform, transition, isDragging } =
    useSortable({ id: subtask.id, disabled: editing });

  return (
    <li
      ref={setNodeRef}
      className={styles.row}
      style={{ transform: CSS.Translate.toString(transform), transition }}
      data-dragging={isDragging || undefined}
    >
      <button
        type="button"
        ref={setActivatorNodeRef}
        className={styles.handle}
        {...attributes}
        {...listeners}
        aria-label={t("subtasks.move", { title: subtask.title })}
      >
        <GripVertical aria-hidden="true" />
      </button>

      <Checkbox
        className={styles.checkbox}
        isSelected={subtask.done}
        onChange={(done) => onToggle(subtask, done)}
        aria-label={subtask.title}
      >
        <span className={styles.box} aria-hidden="true">
          <Check />
        </span>
      </Checkbox>

      {editing ? (
        <InlineInput
          label={t("subtasks.title")}
          initialValue={subtask.title}
          onSubmit={(title) => onRename(subtask.id, title)}
          onClose={() => setEditing(false)}
        />
      ) : (
        <button
          type="button"
          className={styles.title}
          data-done={subtask.done || undefined}
          aria-label={t("subtasks.rename", { title: subtask.title })}
          onClick={() => setEditing(true)}
        >
          {subtask.title}
        </button>
      )}

      <button
        type="button"
        className={styles.delete}
        aria-label={t("subtasks.delete", { title: subtask.title })}
        onClick={() => onDelete(subtask)}
      >
        <X aria-hidden="true" />
      </button>
    </li>
  );
}

export default SubtaskList;
