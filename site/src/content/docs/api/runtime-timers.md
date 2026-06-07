---
title: "runtime:timers"
description: "Timer functions for Forge applications - setTimeout, setInterval, and friends"
slug: api/runtime-timers
---

Timer functions for scheduling code execution in Forge applications. Provides the familiar `setTimeout`, `setInterval`, `clearTimeout`, and `clearInterval` APIs.

> **Implementation**: This module provides timer functionality for the Deno runtime within Forge applications. See [ext_timers](/docs/crates/ext-timers) for implementation details.

## Overview

The timers module provides standard browser-like timer APIs that work in the Deno runtime environment. These functions are also installed as globals, so you can use them without importing.

## Import

```typescript
// Optional - functions are also available as globals
import { setTimeout, setInterval, clearTimeout, clearInterval } from "runtime:timers";
```

## API Reference

<!-- forge:api -->
<!-- generated from sdk/runtime.timers.ts — edit signatures in the SDK, run `make docs-api` to refresh -->
```typescript
setTimeout(callback: (...args: unknown[]) => void, delay?: number, ...args: unknown[]): number
clearTimeout(timerId: number): void
setInterval(callback: (...args: unknown[]) => void, delay?: number, ...args: unknown[]): number
clearInterval(timerId: number): void
```
<!-- /forge:api -->

### setTimeout(callback, delay?, ...args)

Schedules a function to execute after a specified delay.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `callback` | `(...args: unknown[]) => void` | Function to execute |
| `delay` | `number` | Delay in milliseconds (default: 0) |
| `...args` | `unknown[]` | Arguments to pass to callback |

**Returns:** `number` - Timer ID for cancellation

**Example:**

```typescript
// Execute after 1 second
const timerId = setTimeout(() => {
  console.log("Hello after 1 second!");
}, 1000);

// With arguments
setTimeout((name, greeting) => {
  console.log(`${greeting}, ${name}!`);
}, 2000, "Alice", "Hello");

// Minimum delay (executes as soon as possible)
setTimeout(() => {
  console.log("Executes on next tick");
});
```

---

### setInterval(callback, delay?, ...args)

Schedules a function to execute repeatedly at specified intervals.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `callback` | `(...args: unknown[]) => void` | Function to execute |
| `delay` | `number` | Interval in milliseconds (default: 0) |
| `...args` | `unknown[]` | Arguments to pass to callback |

**Returns:** `number` - Timer ID for cancellation

**Example:**

```typescript
// Execute every second
let count = 0;
const intervalId = setInterval(() => {
  count++;
  console.log(`Tick ${count}`);

  if (count >= 5) {
    clearInterval(intervalId);
    console.log("Done!");
  }
}, 1000);

// Polling example
const pollId = setInterval(async () => {
  const status = await checkStatus();
  if (status === "complete") {
    clearInterval(pollId);
    console.log("Task completed!");
  }
}, 5000);
```

---

### clearTimeout(timerId)

Cancels a timeout previously scheduled with `setTimeout`.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `timerId` | `number` | Timer ID returned by setTimeout |

**Example:**

```typescript
const timerId = setTimeout(() => {
  console.log("This will not execute");
}, 5000);

// Cancel before it executes
clearTimeout(timerId);
```

---

### clearInterval(timerId)

Cancels an interval previously scheduled with `setInterval`.

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `timerId` | `number` | Timer ID returned by setInterval |

**Example:**

```typescript
const intervalId = setInterval(() => {
  console.log("Tick");
}, 1000);

// Stop after 5 seconds
setTimeout(() => {
  clearInterval(intervalId);
  console.log("Stopped");
}, 5000);
```

## Type Definitions

```typescript
interface TimerResult {
  id: number;
}

interface TimerCallback {
  callback: (...args: unknown[]) => void;
  args: unknown[];
  repeat: boolean;
  delay: number;
}
```

## Global Availability

Timer functions are installed on `globalThis`, so you can use them without importing:

```typescript
// These work without any import
setTimeout(() => console.log("Hello"), 1000);
setInterval(() => console.log("Tick"), 1000);
clearTimeout(timerId);
clearInterval(intervalId);
```

## Lifecycle Hooks

Timer operations support the standard extensibility hooks.

### onBefore(opName, callback)

```typescript
import { onBefore } from "runtime:timers";

onBefore("timerCreate", (args) => {
  console.log("Creating timer");
});
```

### onAfter(opName, callback)

```typescript
import { onAfter } from "runtime:timers";

onAfter("timerCreate", (result) => {
  console.log("Timer created with ID:", result);
});
```

### onError(opName, callback)

```typescript
import { onError } from "runtime:timers";

onError("timerSleep", (error) => {
  console.error("Timer error:", error.message);
});
```

**Available operation names:** `"timerCreate"`, `"timerCancel"`, `"timerSleep"`, `"timerExists"`

## Complete Example

```typescript
/**
 * Debounce function - delays execution until after a period of inactivity
 */
function debounce<T extends (...args: unknown[]) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timerId: number | undefined;

  return (...args: Parameters<T>) => {
    if (timerId !== undefined) {
      clearTimeout(timerId);
    }

    timerId = setTimeout(() => {
      fn(...args);
      timerId = undefined;
    }, delay);
  };
}

/**
 * Throttle function - limits execution to once per time period
 */
function throttle<T extends (...args: unknown[]) => void>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false;

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      setTimeout(() => {
        inThrottle = false;
      }, limit);
    }
  };
}

/**
 * Retry with exponential backoff
 */
async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> {
  let lastError: Error | undefined;

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      const delay = baseDelay * Math.pow(2, attempt);
      console.log(`Attempt ${attempt + 1} failed, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw lastError;
}

/**
 * Countdown timer
 */
function countdown(seconds: number, onTick: (remaining: number) => void): Promise<void> {
  return new Promise((resolve) => {
    let remaining = seconds;

    const intervalId = setInterval(() => {
      remaining--;
      onTick(remaining);

      if (remaining <= 0) {
        clearInterval(intervalId);
        resolve();
      }
    }, 1000);

    // Initial tick
    onTick(remaining);
  });
}

// Usage examples
async function main() {
  // Debounced search
  const search = debounce((query: string) => {
    console.log(`Searching for: ${query}`);
  }, 300);

  // Simulating rapid input
  search("h");
  search("he");
  search("hel");
  search("hell");
  search("hello"); // Only this one executes

  // Throttled scroll handler
  const handleScroll = throttle(() => {
    console.log("Scroll position updated");
  }, 100);

  // Countdown
  console.log("Starting countdown...");
  await countdown(5, (remaining) => {
    console.log(`${remaining} seconds remaining`);
  });
  console.log("Countdown complete!");

  // Retry with backoff
  try {
    const result = await retryWithBackoff(async () => {
      // Simulating an API call that might fail
      if (Math.random() < 0.5) {
        throw new Error("Network error");
      }
      return "Success!";
    });
    console.log(result);
  } catch (error) {
    console.error("All retries failed:", error);
  }
}

main();
```

## Best Practices

### Always Clear Timers

Clear timers when they're no longer needed to prevent memory leaks:

```typescript
class Component {
  private timerId?: number;

  start() {
    this.timerId = setInterval(() => {
      this.update();
    }, 1000);
  }

  stop() {
    if (this.timerId !== undefined) {
      clearInterval(this.timerId);
      this.timerId = undefined;
    }
  }
}
```

### Handle Callback Errors

Errors in timer callbacks are caught and logged but won't stop the timer:

```typescript
setInterval(() => {
  try {
    riskyOperation();
  } catch (error) {
    console.error("Operation failed:", error);
    // Timer continues running
  }
}, 1000);
```

### Use Promises for One-Time Delays

For simple delays, wrap setTimeout in a Promise:

```typescript
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  console.log("Starting...");
  await delay(2000);
  console.log("After 2 seconds");
}
```

### Be Careful with Minimum Delays

A delay of 0 doesn't mean immediate execution - it schedules for the next tick:

```typescript
console.log("1");
setTimeout(() => console.log("3"), 0);
console.log("2");
// Output: 1, 2, 3
```
