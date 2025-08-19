/**
 * Event Emitter - Simple event system for JavaScript modules
 * 
 * This utility provides a lightweight event emitter implementation
 * for component communication and state management.
 */

/**
 * EventEmitter - Simple event emitter implementation
 * 
 * Features:
 * - Standard event emitter API (on, off, emit)
 * - Once listeners for one-time events
 * - Wildcard event support
 * - Event listener management
 * - Memory leak prevention
 */
export class EventEmitter {
    constructor() {
        this.events = new Map();
        this.maxListeners = 100; // Prevent memory leaks
    }

    /**
     * Add event listener
     */
    on(event, listener) {
        if (typeof listener !== 'function') {
            throw new Error('Listener must be a function');
        }

        if (!this.events.has(event)) {
            this.events.set(event, []);
        }

        const listeners = this.events.get(event);
        
        // Check max listeners
        if (listeners.length >= this.maxListeners) {
            console.warn(`MaxListenersExceededWarning: ${listeners.length + 1} listeners added for event '${event}'. Consider checking for memory leaks.`);
        }

        listeners.push({ listener, once: false });
        
        return this; // For chaining
    }

    /**
     * Add one-time event listener
     */
    once(event, listener) {
        if (typeof listener !== 'function') {
            throw new Error('Listener must be a function');
        }

        if (!this.events.has(event)) {
            this.events.set(event, []);
        }

        const listeners = this.events.get(event);
        listeners.push({ listener, once: true });
        
        return this;
    }

    /**
     * Remove event listener
     */
    off(event, listener) {
        if (!this.events.has(event)) {
            return this;
        }

        if (!listener) {
            // Remove all listeners for event
            this.events.delete(event);
            return this;
        }

        const listeners = this.events.get(event);
        const filtered = listeners.filter(item => item.listener !== listener);
        
        if (filtered.length === 0) {
            this.events.delete(event);
        } else {
            this.events.set(event, filtered);
        }
        
        return this;
    }

    /**
     * Emit event to all listeners
     */
    emit(event, ...args) {
        if (!this.events.has(event)) {
            return false; // No listeners
        }

        const listeners = this.events.get(event);
        const toRemove = [];

        // Call all listeners
        for (let i = 0; i < listeners.length; i++) {
            const { listener, once } = listeners[i];
            
            try {
                listener.apply(this, args);
            } catch (error) {
                console.error(`Error in event listener for '${event}':`, error);
            }

            // Mark once listeners for removal
            if (once) {
                toRemove.push(i);
            }
        }

        // Remove once listeners (in reverse order to maintain indices)
        for (let i = toRemove.length - 1; i >= 0; i--) {
            listeners.splice(toRemove[i], 1);
        }

        // Clean up empty event arrays
        if (listeners.length === 0) {
            this.events.delete(event);
        }

        return true; // Had listeners
    }

    /**
     * Get listener count for event
     */
    listenerCount(event) {
        if (!this.events.has(event)) {
            return 0;
        }
        return this.events.get(event).length;
    }

    /**
     * Get all event names
     */
    eventNames() {
        return Array.from(this.events.keys());
    }

    /**
     * Get all listeners for event
     */
    listeners(event) {
        if (!this.events.has(event)) {
            return [];
        }
        return this.events.get(event).map(item => item.listener);
    }

    /**
     * Remove all listeners
     */
    removeAllListeners(event) {
        if (event) {
            this.events.delete(event);
        } else {
            this.events.clear();
        }
        return this;
    }

    /**
     * Set maximum number of listeners per event
     */
    setMaxListeners(max) {
        if (typeof max !== 'number' || max < 0) {
            throw new Error('Max listeners must be a non-negative number');
        }
        this.maxListeners = max;
        return this;
    }

    /**
     * Get maximum listeners setting
     */
    getMaxListeners() {
        return this.maxListeners;
    }

    /**
     * Add listener to beginning of listeners array
     */
    prependListener(event, listener) {
        if (typeof listener !== 'function') {
            throw new Error('Listener must be a function');
        }

        if (!this.events.has(event)) {
            this.events.set(event, []);
        }

        const listeners = this.events.get(event);
        listeners.unshift({ listener, once: false });
        
        return this;
    }

    /**
     * Add one-time listener to beginning of listeners array
     */
    prependOnceListener(event, listener) {
        if (typeof listener !== 'function') {
            throw new Error('Listener must be a function');
        }

        if (!this.events.has(event)) {
            this.events.set(event, []);
        }

        const listeners = this.events.get(event);
        listeners.unshift({ listener, once: true });
        
        return this;
    }
}

export default EventEmitter;