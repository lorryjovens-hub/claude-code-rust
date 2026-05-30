import { describe, it, expect } from 'vitest';
import { getErrorMessage } from '../utils/errorHelpers';

describe('getErrorMessage', () => {
  it('should return error message from Error instance', () => {
    const error = new Error('Test error message');
    expect(getErrorMessage(error)).toBe('Test error message');
  });

  it('should return string when input is string', () => {
    expect(getErrorMessage('String error')).toBe('String error');
  });

  it('should return fallback message for unknown error types', () => {
    expect(getErrorMessage(null as unknown)).toBe('Unknown error');
    expect(getErrorMessage(undefined)).toBe('Unknown error');
    expect(getErrorMessage(123 as unknown)).toBe('Unknown error');
    expect(getErrorMessage({} as unknown)).toBe('Unknown error');
  });

  it('should use custom fallback value', () => {
    expect(getErrorMessage(null as unknown, 'Custom fallback')).toBe('Custom fallback');
  });
});
