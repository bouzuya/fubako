function addClipboardCopyButton() {
    const li = document.querySelector(".breadcrumb_section > ol > li:nth-child(2)");
    if (li !== null) {
        const pageId = li.textContent.trim();
        const button = document.createElement("button");
        button.type = "button";
        button.className = "clipboard_copy_button";
        button.textContent = "📋";
        button.addEventListener("click", async () => {
            button.disabled = true;
            try {
                await navigator.clipboard.writeText(pageId);
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
        li.appendChild(button);
    }

}

function main() {
    addClipboardCopyButton();
}

main();
