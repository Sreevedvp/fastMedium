// Runs after mounting through Dioxus document::eval. Values arrive as data,
// never interpolated executable source. Native dialog handles modal semantics.
const id = await dioxus.recv();
const dialog = document.getElementById(id);
if (!dialog || dialog._fuiDialog) return;
let trigger = null;
let programmaticClose = false;
let pointerStartedOutside = false;
const outside = event => {
    const rect = dialog.getBoundingClientRect();
    return event.target === dialog && (event.clientX < rect.left || event.clientX > rect.right ||
        event.clientY < rect.top || event.clientY > rect.bottom);
};
const requestDismiss = () => {
    if (dialog.dataset.dismissible === "true") dioxus.send("dismiss");
};
const cancel = event => { event.preventDefault(); requestDismiss(); };
const close = () => {
    if (programmaticClose) { programmaticClose = false; return; }
    dioxus.send("dismiss");
};
const pointerdown = event => { pointerStartedOutside = outside(event); };
const click = event => {
    if (pointerStartedOutside && outside(event) && dialog.dataset.backdropDismiss === "true") requestDismiss();
    pointerStartedOutside = false;
};
// Native inertness prevents background focus. Explicitly wrap the endpoints
// too, so Tab never falls through to browser chrome. Nested dialogs own Tab.
const keydown = event => {
    if (event.key !== "Tab" || event.target.closest("dialog") !== dialog) return;
    const tabbable = [...dialog.querySelectorAll('button, input, select, textarea, a[href], area[href], summary, [tabindex], [contenteditable="true"]')]
        .filter(el => el.tabIndex >= 0 && !el.matches(":disabled") && !el.closest("[inert]") &&
            el.getClientRects().length && getComputedStyle(el).visibility !== "hidden" &&
            el.closest("dialog") === dialog);
    const first = tabbable[0], last = tabbable.at(-1);
    if (!first) { event.preventDefault(); return; }
    if (event.shiftKey && document.activeElement === first) {
        event.preventDefault(); last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault(); first.focus();
    }
};
dialog.addEventListener("cancel", cancel);
dialog.addEventListener("close", close);
dialog.addEventListener("pointerdown", pointerdown);
dialog.addEventListener("click", click);
dialog.addEventListener("keydown", keydown);
dialog._fuiDialog = {
    setOpen(open) {
        if (open && !dialog.open) {
            trigger = document.activeElement;
            dialog.showModal();
            dialog.querySelector("[data-fui-autofocus]")?.focus();
        } else if (!open && dialog.open) {
            programmaticClose = true;
            dialog.close();
        }
    }
};
// Removing an open dialog should also restore focus and release listeners.
const observer = new MutationObserver(() => {
    if (dialog.isConnected) return;
    observer.disconnect();
    dialog.removeEventListener("cancel", cancel);
    dialog.removeEventListener("close", close);
    dialog.removeEventListener("pointerdown", pointerdown);
    dialog.removeEventListener("click", click);
    dialog.removeEventListener("keydown", keydown);
    if (dialog.open && trigger?.isConnected) trigger.focus();
    delete dialog._fuiDialog;
});
observer.observe(document.documentElement, { childList: true, subtree: true });
dioxus.send("ready");
