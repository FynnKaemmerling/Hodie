import { useRef, useState, type MouseEvent, type ReactNode } from "react";
import { Ellipsis } from "lucide-react";
import { Button, Menu, MenuItem, MenuTrigger, Popover } from "react-aria-components";
import { isTextInput } from "../platform";
import styles from "./ActionMenu.module.css";

export interface ActionMenuItem {
  id: string;
  label: string;
  onAction: () => void;
  danger?: boolean;
  disabled?: boolean;
}

function MenuItems({ items }: { items: ActionMenuItem[] }) {
  return (
    <Menu
      className={styles.menu}
      autoFocus="first"
      disabledKeys={items.filter((item) => item.disabled).map((item) => item.id)}
      onAction={(key) => items.find((item) => item.id === key)?.onAction()}
    >
      {items.map((item) => (
        <MenuItem
          key={item.id}
          id={item.id}
          className={styles.item}
          data-danger={item.danger || undefined}
        >
          {item.label}
        </MenuItem>
      ))}
    </Menu>
  );
}

interface Props {
  label: string;
  items: ActionMenuItem[];
}

/** A "more" button that opens a menu of actions. */
function ActionMenu({ label, items }: Props) {
  return (
    <MenuTrigger>
      <Button className={styles.trigger} aria-label={label}>
        <Ellipsis aria-hidden="true" />
      </Button>
      <Popover className={styles.popover} placement="bottom end" offset={4}>
        <MenuItems items={items} />
      </Popover>
    </MenuTrigger>
  );
}

/**
 * The same actions as a right-click menu at the pointer. Spread
 * `onContextMenu` on the element and render `menu` next to it.
 */
export function useContextMenu(items: ActionMenuItem[]): {
  onContextMenu: (event: MouseEvent) => void;
  menu: ReactNode;
} {
  const [position, setPosition] = useState<{ x: number; y: number } | null>(null);
  const anchor = useRef<HTMLDivElement>(null);

  function onContextMenu(event: MouseEvent) {
    // Text fields keep the system menu for cut, copy and paste.
    if (isTextInput(event.target)) return;
    event.preventDefault();
    event.stopPropagation();
    // The context menu key and Shift+F10 report no pointer position.
    if (event.clientX === 0 && event.clientY === 0) {
      const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
      setPosition({ x: rect.left + rect.width / 2, y: rect.bottom });
    } else {
      setPosition({ x: event.clientX, y: event.clientY });
    }
  }

  const menu = position && (
    <>
      <div ref={anchor} className={styles.anchor} style={{ left: position.x, top: position.y }} />
      <Popover
        className={styles.popover}
        triggerRef={anchor}
        isOpen
        onOpenChange={(open) => !open && setPosition(null)}
        placement="bottom start"
        offset={2}
      >
        <MenuItems items={items} />
      </Popover>
    </>
  );

  return { onContextMenu, menu };
}

export default ActionMenu;
