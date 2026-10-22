import { Ellipsis } from "lucide-react";
import { Button, Menu, MenuItem, MenuTrigger, Popover } from "react-aria-components";
import styles from "./ActionMenu.module.css";

export interface ActionMenuItem {
  id: string;
  label: string;
  onAction: () => void;
  danger?: boolean;
  disabled?: boolean;
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
        <Menu
          className={styles.menu}
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
      </Popover>
    </MenuTrigger>
  );
}

export default ActionMenu;
