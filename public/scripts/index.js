function addPageIdCopyButton() {
    const li = document.querySelector(".breadcrumb_section > ol > li:nth-child(2)");
    if (li === null) return;
    const pageId = li.textContent.trim();
    if (pageId === "pages" || pageId === "titles") return;
    const button = createClipboardCopyButton(pageId);
    li.appendChild(button);
}

function addTitleUrlCopyButton() {
    const div = document.querySelector(".page_title_section");
    if (div === null) return;
    const a = div.querySelector("p a");
    if (a === null) return;
    const href = a.getAttribute("href");
    if (href === null) return;
    const button = createClipboardCopyButton(href);
    div.appendChild(button);
}

// string -> HTMLButtonElement
function createClipboardCopyButton(text) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "clipboard_copy_button";
    button.textContent = "📋";
    button.addEventListener("click", async () => {
        button.disabled = true;
        try {
            await navigator.clipboard.writeText(text);
            button.textContent = "✔";
            await new Promise((resolve) => {
                setTimeout(() => {
                    button.textContent = "📋";
                    resolve();
                }, 1200);
            });
        } finally {
            button.disabled = false;
        }
    });
    return button;
}

function main() {
    addPageIdCopyButton();
    addTitleUrlCopyButton();
}

main();
