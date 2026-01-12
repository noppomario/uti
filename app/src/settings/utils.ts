/**
 * Utility functions for settings
 */

/** Keys that could cause prototype pollution */
const DANGEROUS_KEYS = ['__proto__', 'constructor', 'prototype'];

/**
 * Validate that a path does not contain dangerous keys
 *
 * @param path - The path to validate
 * @throws Error if path contains dangerous keys
 */
function validatePath(path: string): void {
  const keys = path.split('.');
  for (const key of keys) {
    if (DANGEROUS_KEYS.includes(key)) {
      throw new Error(`Invalid path: "${key}" is not allowed`);
    }
  }
}

/**
 * Get a nested value from an object using dot notation
 *
 * @param obj - The object to get the value from
 * @param path - The path to the value (e.g., 'theme.color')
 * @returns The value at the path, or undefined if not found
 */
export function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
  validatePath(path);
  const keys = path.split('.');
  let current: unknown = obj;

  for (const key of keys) {
    if (current === null || current === undefined) {
      return undefined;
    }
    current = (current as Record<string, unknown>)[key];
  }

  return current;
}

/**
 * Set a nested value in an object using dot notation
 *
 * @param obj - The object to set the value in
 * @param path - The path to the value (e.g., 'theme.color')
 * @param value - The value to set
 * @returns A new object with the value set
 */
export function setNestedValue<T extends Record<string, unknown>>(
  obj: T,
  path: string,
  value: unknown
): T {
  validatePath(path);
  const keys = path.split('.');
  const result = { ...obj } as Record<string, unknown>;

  let current: Record<string, unknown> = result;
  for (let i = 0; i < keys.length - 1; i++) {
    const key = keys[i];
    if (current[key] === undefined || current[key] === null) {
      current[key] = {};
    } else {
      current[key] = { ...(current[key] as Record<string, unknown>) };
    }
    current = current[key] as Record<string, unknown>;
  }

  current[keys[keys.length - 1]] = value;
  return result as T;
}
