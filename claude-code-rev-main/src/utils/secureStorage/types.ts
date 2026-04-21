export type SecureStorageData = Record<string, unknown>

export interface SecureStorage {
  get(key: string): Promise<SecureStorageData | null>
  set(key: string, value: SecureStorageData): Promise<void>
  delete(key: string): Promise<void>
}
