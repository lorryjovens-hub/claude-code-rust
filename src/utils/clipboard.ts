/**
 * Copies text to clipboard, compatible with HTTP and HTTPS environments.
 * Uses navigator.clipboard if available, otherwise falls back to document.execCommand.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
    if (!text) return false;

    try {
        // Try Modern Async API first (needs Secure Context)
        // We check if navigator.clipboard is available and writeText is a function
        if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
            await navigator.clipboard.writeText(text);
            return true;
        }
    } catch (err) {
        console.warn('Navigator clipboard failed, trying fallback', err);
    }

    // Fallback for HTTP / older browsers
    try {
        const textArea = document.createElement('textarea');
        textArea.value = text;

        // Ensure the textarea is not visible but part of the DOM
        textArea.style.position = 'fixed'; // Avoid scrolling to bottom
        textArea.style.left = '-9999px';
        textArea.style.top = '0';
        document.body.appendChild(textArea);

        textArea.focus();
        textArea.select();

        const successful = document.execCommand('copy');
        document.body.removeChild(textArea);
        return successful;
    } catch (err) {
        console.error('Fallback clipboard copy failed', err);
        return false;
    }
}
