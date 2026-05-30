// services/secureStorage.ts
//
// Secure storage abstraction for Tauri's secure-storage plugin.
// All secrets go through Tauri's secure-storage plugin via invoke().
// localStorage is NOT used as fallback for sensitive keys — if Tauri is
// unavailable (e.g. browser dev), secrets are simply not persisted.
// Non-sensitive keys (language, zoom, onboarding_done) may still use
// localStorage directly in their own modules.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SECURE_KEYS = [
  'ANTHROPIC_API_KEY',
  'auth_token',
  'bridge_api_key',
  'gateway_user',
  'CUSTOM_BASE_URL',
  'CUSTOM_API_KEY',
  'ANTHROPIC_BASE_URL',
] as const;

type SecureKey = (typeof SECURE_KEYS)[number];

// ---------------------------------------------------------------------------
// Tauri detection
// ---------------------------------------------------------------------------

/**
 * Returns true when the app is running inside a Tauri webview.
 * We check for the Tauri internals object that Tauri injects at runtime.
 */
export function isSecureAvailable(): boolean {
  try {
    return (
      typeof window !== 'undefined' &&
      '__TAURI_INTERNALS__' in window &&
      typeof (window as any).__TAURI_INTERNALS__?.invoke === 'function'
    );
  } catch {
    return false;
  }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function tauriInvoke<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T> {
  return (window as any).__TAURI_INTERNALS__.invoke(command, args ?? {});
}

/**
 * Log an error to the console without throwing.
 * Prefixes messages with [secureStorage] so they are easy to filter.
 */
function logError(context: string, error: unknown): void {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`[secureStorage] ${context}: ${message}`);
}

/**
 * Guard: return early when Tauri is not available.
 * Returns true when the operation should be skipped (no Tauri).
 */
function skipWhenUnavailable(operation: string): boolean {
  if (!isSecureAvailable()) {
    console.warn(
      `[secureStorage] ${operation} skipped — Tauri secure storage is not available in this environment.`,
    );
    return true;
  }
  return false;
}

/**
 * Validate that the caller passed one of the known secure keys.
 * This is a runtime safety net; TypeScript's `SecureKey` type already
 * prevents most mistakes at compile time.
 */
function assertSecureKey(key: string): asserts key is SecureKey {
  if (!(SECURE_KEYS as readonly string[]).includes(key)) {
    throw new Error(
      `[secureStorage] "${key}" is not a recognised secure key. ` +
        `Allowed keys: ${SECURE_KEYS.join(', ')}`,
    );
  }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Retrieve a single secret from Tauri's secure-storage plugin.
 *
 * Returns `null` when:
 *  - The key is not found in storage
 *  - Tauri is unavailable (e.g. running in a browser)
 *  - The underlying plugin call fails for any reason
 *
 * This function NEVER throws.
 */
export async function getSecure(key: SecureKey): Promise<string | null> {
  assertSecureKey(key);

  // IMPORTANT: secrets are never fetched from localStorage.
  if (skipWhenUnavailable(`getSecure("${key}")`)) {
    return null;
  }

  try {
    const result = await tauriInvoke<string | null>('plugin:secure-storage|get_item', {
      prefixed_key: key,
    });
    return result ?? null;
  } catch (error) {
    logError(`getSecure("${key}")`, error);
    return null;
  }
}

/**
 * Persist a secret to Tauri's secure-storage plugin.
 *
 * Does nothing when Tauri is unavailable — the secret is simply not persisted.
 * This function NEVER throws.
 */
export async function setSecure(key: SecureKey, value: string): Promise<void> {
  assertSecureKey(key);

  // IMPORTANT: secrets are never written to localStorage.
  if (skipWhenUnavailable(`setSecure("${key}")`)) {
    return;
  }

  try {
    await tauriInvoke('plugin:secure-storage|set_item', {
      prefixed_key: key,
      value,
    });
  } catch (error) {
    logError(`setSecure("${key}")`, error);
  }
}

/**
 * Remove a secret from Tauri's secure-storage plugin.
 *
 * Does nothing when Tauri is unavailable.
 * This function NEVER throws.
 */
export async function removeSecure(key: SecureKey): Promise<void> {
  assertSecureKey(key);

  // IMPORTANT: secrets are never removed from localStorage here — that is the
  // responsibility of whatever module originally wrote them there (during a
  // migration window).
  if (skipWhenUnavailable(`removeSecure("${key}")`)) {
    return;
  }

  try {
    await tauriInvoke('plugin:secure-storage|remove_item', {
      prefixed_key: key,
    });
  } catch (error) {
    logError(`removeSecure("${key}")`, error);
  }
}

/**
 * Fetch all known secrets in a single call.
 *
 * Keys that are unavailable or fail to retrieve are omitted from the result
 * (rather than appearing as `null`), so you can safely iterate with
 * `Object.entries()`.
 *
 * This function NEVER throws.
 */
export async function getAllSecrets(): Promise<Record<string, string>> {
  if (skipWhenUnavailable('getAllSecrets()')) {
    return {};
  }

  const result: Record<string, string> = {};

  // Fetch all keys in parallel for speed.
  const entries = await Promise.allSettled(
    SECURE_KEYS.map(async (key): Promise<[string, string]> => {
      const value = await tauriInvoke<string | null>('plugin:secure-storage|get_item', {
        prefixed_key: key,
      });
      if (value == null || value === '') {
        // Use Promise.reject so Promise.allSettled marks this as rejected and
        // we simply skip the entry.
        throw new Error('MISSING');
      }
      return [key, value];
    }),
  );

  for (const entry of entries) {
    if (entry.status === 'fulfilled') {
      const [k, v] = entry.value;
      result[k] = v;
    }
    // rejected entries are silently skipped (key not found / error)
  }

  return result;
}
