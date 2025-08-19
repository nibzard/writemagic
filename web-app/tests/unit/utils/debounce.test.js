/**
 * Unit tests for Debounce utility
 * Tests debouncing functionality for performance optimization
 */

import { debounce, debounceAsync } from '@/utils/debounce.js';

describe('Debounce Utility', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('basic debounce functionality', () => {
    test('should delay function execution', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      expect(mockFn).not.toHaveBeenCalled();
      
      jest.advanceTimersByTime(100);
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should cancel previous calls when called repeatedly', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      debouncedFn();
      debouncedFn();
      
      jest.advanceTimersByTime(100);
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should pass arguments correctly', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn('arg1', 'arg2', 'arg3');
      jest.advanceTimersByTime(100);
      
      expect(mockFn).toHaveBeenCalledWith('arg1', 'arg2', 'arg3');
    });

    test('should maintain this context', () => {
      const obj = {
        value: 42,
        method: jest.fn(function() {
          return this.value;
        })
      };
      
      obj.debouncedMethod = debounce(obj.method, 100);
      obj.debouncedMethod();
      
      jest.advanceTimersByTime(100);
      expect(obj.method).toHaveBeenCalled();
    });
  });

  describe('immediate execution option', () => {
    test('should execute immediately with immediate flag', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100, { immediate: true });
      
      debouncedFn();
      expect(mockFn).toHaveBeenCalledTimes(1);
      
      // Subsequent calls should be debounced
      debouncedFn();
      debouncedFn();
      expect(mockFn).toHaveBeenCalledTimes(1);
      
      jest.advanceTimersByTime(100);
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should allow execution again after delay with immediate flag', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100, { immediate: true });
      
      debouncedFn();
      expect(mockFn).toHaveBeenCalledTimes(1);
      
      jest.advanceTimersByTime(100);
      
      debouncedFn();
      expect(mockFn).toHaveBeenCalledTimes(2);
    });
  });

  describe('max wait option', () => {
    test('should force execution after max wait time', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100, { maxWait: 200 });
      
      debouncedFn();
      jest.advanceTimersByTime(50);
      
      debouncedFn();
      jest.advanceTimersByTime(50);
      
      debouncedFn();
      jest.advanceTimersByTime(50);
      
      debouncedFn();
      jest.advanceTimersByTime(50);
      
      // Should execute due to max wait being reached
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should reset max wait timer after execution', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100, { maxWait: 200 });
      
      debouncedFn();
      jest.advanceTimersByTime(200);
      expect(mockFn).toHaveBeenCalledTimes(1);
      
      // Should be able to execute again after max wait reset
      debouncedFn();
      jest.advanceTimersByTime(200);
      expect(mockFn).toHaveBeenCalledTimes(2);
    });
  });

  describe('cancellation', () => {
    test('should provide cancel method', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      expect(typeof debouncedFn.cancel).toBe('function');
    });

    test('should cancel pending execution', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      debouncedFn.cancel();
      
      jest.advanceTimersByTime(100);
      expect(mockFn).not.toHaveBeenCalled();
    });

    test('should allow new execution after cancellation', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      debouncedFn.cancel();
      debouncedFn();
      
      jest.advanceTimersByTime(100);
      expect(mockFn).toHaveBeenCalledTimes(1);
    });
  });

  describe('flush functionality', () => {
    test('should provide flush method', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      expect(typeof debouncedFn.flush).toBe('function');
    });

    test('should immediately execute pending function', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      debouncedFn.flush();
      
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should return function result when flushed', () => {
      const mockFn = jest.fn(() => 'result');
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      const result = debouncedFn.flush();
      
      expect(result).toBe('result');
    });

    test('should not execute if no pending call', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 100);
      
      const result = debouncedFn.flush();
      
      expect(mockFn).not.toHaveBeenCalled();
      expect(result).toBeUndefined();
    });
  });

  describe('debounceAsync function', () => {
    test('should debounce async functions', async () => {
      const mockAsyncFn = jest.fn().mockResolvedValue('async result');
      const debouncedAsyncFn = debounceAsync(mockAsyncFn, 100);
      
      const promise = debouncedAsyncFn();
      jest.advanceTimersByTime(100);
      
      const result = await promise;
      expect(result).toBe('async result');
      expect(mockAsyncFn).toHaveBeenCalledTimes(1);
    });

    test('should handle async function rejections', async () => {
      const mockAsyncFn = jest.fn().mockRejectedValue(new Error('Async error'));
      const debouncedAsyncFn = debounceAsync(mockAsyncFn, 100);
      
      const promise = debouncedAsyncFn();
      jest.advanceTimersByTime(100);
      
      await expect(promise).rejects.toThrow('Async error');
    });

    test('should cancel pending async calls', async () => {
      const mockAsyncFn = jest.fn().mockResolvedValue('result');
      const debouncedAsyncFn = debounceAsync(mockAsyncFn, 100);
      
      const promise1 = debouncedAsyncFn();
      const promise2 = debouncedAsyncFn();
      
      jest.advanceTimersByTime(100);
      
      // First promise should be cancelled, second should resolve
      await expect(promise1).rejects.toThrow('Debounced call cancelled');
      await expect(promise2).resolves.toBe('result');
      expect(mockAsyncFn).toHaveBeenCalledTimes(1);
    });
  });

  describe('real-world usage scenarios', () => {
    test('should handle search input debouncing', () => {
      const mockSearchFn = jest.fn();
      const debouncedSearch = debounce(mockSearchFn, 300);
      
      // Simulate rapid typing
      'hello world'.split('').forEach((char, i) => {
        debouncedSearch('hello world'.slice(0, i + 1));
        jest.advanceTimersByTime(50);
      });
      
      // Should only call once after the delay
      jest.advanceTimersByTime(300);
      expect(mockSearchFn).toHaveBeenCalledTimes(1);
      expect(mockSearchFn).toHaveBeenCalledWith('hello world');
    });

    test('should handle window resize debouncing', () => {
      const mockResizeHandler = jest.fn();
      const debouncedResize = debounce(mockResizeHandler, 250);
      
      // Simulate multiple resize events
      for (let i = 0; i < 10; i++) {
        debouncedResize();
        jest.advanceTimersByTime(20);
      }
      
      jest.advanceTimersByTime(250);
      expect(mockResizeHandler).toHaveBeenCalledTimes(1);
    });

    test('should handle auto-save debouncing', () => {
      const mockSaveFn = jest.fn();
      const debouncedSave = debounce(mockSaveFn, 1000, { maxWait: 5000 });
      
      // Simulate continuous typing for 6 seconds
      for (let i = 0; i < 60; i++) {
        debouncedSave('content ' + i);
        jest.advanceTimersByTime(100);
      }
      
      // Should have executed due to maxWait
      expect(mockSaveFn).toHaveBeenCalled();
    });

    test('should handle API request debouncing', async () => {
      const mockApiCall = jest.fn().mockResolvedValue({ data: 'response' });
      const debouncedApiCall = debounceAsync(mockApiCall, 500);
      
      // Simulate rapid API requests
      const promises = [];
      for (let i = 0; i < 5; i++) {
        promises.push(debouncedApiCall(`query-${i}`));
        jest.advanceTimersByTime(100);
      }
      
      jest.advanceTimersByTime(500);
      
      // Only the last promise should resolve
      const results = await Promise.allSettled(promises);
      const resolved = results.filter(r => r.status === 'fulfilled');
      const rejected = results.filter(r => r.status === 'rejected');
      
      expect(resolved).toHaveLength(1);
      expect(rejected).toHaveLength(4);
      expect(mockApiCall).toHaveBeenCalledTimes(1);
      expect(mockApiCall).toHaveBeenCalledWith('query-4');
    });
  });

  describe('edge cases and error handling', () => {
    test('should handle function that throws error', () => {
      const mockFn = jest.fn(() => {
        throw new Error('Function error');
      });
      const debouncedFn = debounce(mockFn, 100);
      
      debouncedFn();
      
      expect(() => {
        jest.advanceTimersByTime(100);
      }).toThrow('Function error');
    });

    test('should handle zero delay', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, 0);
      
      debouncedFn();
      jest.advanceTimersByTime(0);
      
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should handle negative delay', () => {
      const mockFn = jest.fn();
      const debouncedFn = debounce(mockFn, -100);
      
      debouncedFn();
      jest.advanceTimersByTime(0);
      
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    test('should handle non-function input', () => {
      expect(() => {
        debounce(null, 100);
      }).toThrow('Expected a function');
      
      expect(() => {
        debounce('not a function', 100);
      }).toThrow('Expected a function');
    });

    test('should handle invalid options', () => {
      const mockFn = jest.fn();
      
      // Should not throw with invalid options
      expect(() => {
        debounce(mockFn, 100, { invalid: 'option' });
      }).not.toThrow();
    });
  });
});