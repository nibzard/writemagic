/**
 * Debounce Utility - Performance optimization for frequent function calls
 * 
 * This utility provides debouncing functionality to limit the rate at which
 * functions can fire, essential for auto-save, search, and other real-time features.
 */

/**
 * Debounce function execution
 * 
 * @param {Function} func - Function to debounce
 * @param {number} wait - Wait time in milliseconds
 * @param {boolean} immediate - Whether to execute on leading edge
 * @returns {Function} Debounced function
 */
export function debounce(func, wait, immediate = false) {
    let timeout;
    let lastArgs;
    let lastThis;
    let result;

    const debounced = function(...args) {
        lastArgs = args;
        lastThis = this;

        const later = () => {
            timeout = null;
            if (!immediate) {
                result = func.apply(lastThis, lastArgs);
            }
        };

        const callNow = immediate && !timeout;
        
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
        
        if (callNow) {
            result = func.apply(this, args);
        }

        return result;
    };

    // Add cancel method to stop pending execution
    debounced.cancel = function() {
        clearTimeout(timeout);
        timeout = null;
    };

    // Add flush method to execute immediately
    debounced.flush = function() {
        if (timeout) {
            clearTimeout(timeout);
            timeout = null;
            result = func.apply(lastThis, lastArgs);
        }
        return result;
    };

    // Add pending method to check if execution is pending
    debounced.pending = function() {
        return timeout !== null;
    };

    return debounced;
}

/**
 * Throttle function execution (execute at most once per interval)
 * 
 * @param {Function} func - Function to throttle
 * @param {number} limit - Time limit in milliseconds
 * @param {boolean} leading - Whether to execute on leading edge
 * @param {boolean} trailing - Whether to execute on trailing edge
 * @returns {Function} Throttled function
 */
export function throttle(func, limit, leading = true, trailing = true) {
    let inThrottle;
    let lastFunc;
    let lastRan;
    let lastArgs;
    let lastThis;

    return function(...args) {
        lastArgs = args;
        lastThis = this;

        if (!inThrottle) {
            if (leading) {
                func.apply(this, args);
                lastRan = Date.now();
            }
            inThrottle = true;
        } else {
            clearTimeout(lastFunc);
            lastFunc = setTimeout(() => {
                if (Date.now() - lastRan >= limit) {
                    if (trailing) {
                        func.apply(lastThis, lastArgs);
                    }
                    lastRan = Date.now();
                    inThrottle = false;
                }
            }, limit - (Date.now() - lastRan));
        }
    };
}

/**
 * Advanced debounce with customizable options
 * 
 * @param {Function} func - Function to debounce
 * @param {number} wait - Wait time in milliseconds
 * @param {Object} options - Configuration options
 * @returns {Function} Debounced function
 */
export function advancedDebounce(func, wait, options = {}) {
    const {
        immediate = false,
        maxWait = null,
        leading = false,
        trailing = true
    } = options;

    let timeout;
    let maxTimeout;
    let lastArgs;
    let lastThis;
    let result;
    let lastCallTime;
    let lastInvokeTime = 0;

    const invokeFunc = (time) => {
        lastInvokeTime = time;
        result = func.apply(lastThis, lastArgs);
        return result;
    };

    const leadingEdge = (time) => {
        lastInvokeTime = time;
        timeout = setTimeout(timerExpired, wait);
        return leading ? invokeFunc(time) : result;
    };

    const remainingWait = (time) => {
        const timeSinceLastCall = time - lastCallTime;
        const timeSinceLastInvoke = time - lastInvokeTime;
        const timeWaiting = wait - timeSinceLastCall;

        return maxWait !== null
            ? Math.min(timeWaiting, maxWait - timeSinceLastInvoke)
            : timeWaiting;
    };

    const shouldInvoke = (time) => {
        const timeSinceLastCall = time - lastCallTime;
        const timeSinceLastInvoke = time - lastInvokeTime;

        return (
            !timeout ||
            timeSinceLastCall >= wait ||
            timeSinceLastCall < 0 ||
            (maxWait !== null && timeSinceLastInvoke >= maxWait)
        );
    };

    const timerExpired = () => {
        const time = Date.now();
        if (shouldInvoke(time)) {
            return trailingEdge(time);
        }
        timeout = setTimeout(timerExpired, remainingWait(time));
    };

    const trailingEdge = (time) => {
        timeout = null;
        if (trailing && lastArgs) {
            return invokeFunc(time);
        }
        lastArgs = lastThis = null;
        return result;
    };

    const debounced = function(...args) {
        const time = Date.now();
        const isInvoking = shouldInvoke(time);

        lastArgs = args;
        lastThis = this;
        lastCallTime = time;

        if (isInvoking) {
            if (!timeout) {
                return leadingEdge(lastCallTime);
            }
            if (maxWait !== null) {
                timeout = setTimeout(timerExpired, wait);
                return invokeFunc(lastCallTime);
            }
        }
        if (!timeout) {
            timeout = setTimeout(timerExpired, wait);
        }
        return result;
    };

    debounced.cancel = () => {
        if (timeout) {
            clearTimeout(timeout);
        }
        if (maxTimeout) {
            clearTimeout(maxTimeout);
        }
        lastInvokeTime = 0;
        lastArgs = lastCallTime = lastThis = timeout = maxTimeout = null;
    };

    debounced.flush = () => {
        return !timeout ? result : trailingEdge(Date.now());
    };

    debounced.pending = () => {
        return timeout !== null;
    };

    return debounced;
}

export default debounce;