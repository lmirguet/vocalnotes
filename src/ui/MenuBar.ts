// Disclosure panels keep ordinary button/input keyboard semantics.
export function createMenuBar(nonce: string) {
  const menu = document.createElement("nav");
  menu.setAttribute("aria-label", "Menus");
  menu.className = "menu-bar";
  const groups: Array<{ trigger: HTMLButtonElement; panel: HTMLElement }> = [];
  let opened: typeof groups[number] | undefined;
  // Disclosure buttons, not ARIA menu/menuitem: panels contain an editable path
  // and help prose. Enter/Space and Tab keep their ordinary native semantics.
  function closeMenu(returnFocus = false) {
    if (!opened) return;
    const previous = opened;
    previous.panel.hidden = true;
    previous.trigger.setAttribute("aria-expanded", "false");
    opened = undefined;
    if (returnFocus) previous.trigger.focus();
  }
  function menuClick(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const group = groups.find(g => g.trigger === target.closest("button"));
    if (group) {
      const wasOpen = opened === group;
      closeMenu();
      group.trigger.focus();
      if (!wasOpen) {
        group.panel.hidden = false;
        group.trigger.setAttribute("aria-expanded", "true");
        opened = group;
      }
    } else if (opened?.panel.contains(target) && target.closest("button[data-action]:not(:disabled)")) {
      closeMenu(true);
    }
  }
  function menuKey(event: KeyboardEvent) {
    const index = groups.findIndex(g => g.trigger === event.target);
    if (index < 0) return; // arrows in the path input edit text, never move menus
    const next = event.key === "ArrowRight" ? (index + 1) % groups.length
      : event.key === "ArrowLeft" ? (index + groups.length - 1) % groups.length
        : event.key === "Home" ? 0 : event.key === "End" ? groups.length - 1 : -1;
    if (next < 0) return;
    event.preventDefault();
    closeMenu();
    groups[next]!.trigger.focus();
  }
  function outsideClick(event: MouseEvent) {
    if (event.target instanceof Node && !menu.contains(event.target)) closeMenu();
  }
  function escapeMenu(event: KeyboardEvent) {
    if (event.key === "Escape" && opened && event.target instanceof Node && menu.contains(event.target)) {
      event.preventDefault();
      closeMenu(true);
    }
  }
  function leaveMenu(event: FocusEvent) {
    if (!(event.relatedTarget instanceof Node) || !menu.contains(event.relatedTarget)) closeMenu();
  }
  menu.addEventListener("click", menuClick);
  menu.addEventListener("keydown", menuKey);
  menu.addEventListener("focusout", leaveMenu);
  document.addEventListener("click", outsideClick);
  document.addEventListener("keydown", escapeMenu);

  function group(label: string) {
    const container = document.createElement("div");
    container.className = "menu-group";
    const trigger = document.createElement("button");
    trigger.type = "button"; trigger.className = "menu-trigger"; trigger.textContent = label;
    const panel = document.createElement("section");
    panel.className = "menu-panel"; panel.id = `menu-${nonce}-${groups.length}`;
    panel.setAttribute("aria-label", label); panel.hidden = true;
    trigger.setAttribute("aria-controls", panel.id); trigger.setAttribute("aria-expanded", "false");
    groups.push({ trigger, panel }); container.append(trigger, panel); menu.append(container);
    return panel;
  }
  return { element: menu, group, destroy() {
    closeMenu();
    menu.removeEventListener("click", menuClick);
    menu.removeEventListener("keydown", menuKey);
    menu.removeEventListener("focusout", leaveMenu);
    document.removeEventListener("click", outsideClick);
    document.removeEventListener("keydown", escapeMenu);
  } };
}
