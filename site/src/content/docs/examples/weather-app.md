---
title: "weather-app"
description: Weather app demonstrating runtime:net and system notifications
slug: examples/weather-app
---

A weather application demonstrating network requests and system integration.

## Overview

This example shows:
- HTTP requests with `runtime:net`
- Capability-scoped network access
- System notifications with `runtime:sys`
- Clipboard integration

## Features

- Current weather display
- Location-based weather (via geocoding)
- Desktop notifications for weather alerts
- Copy weather to clipboard

## Running

```bash
forge dev examples/weather-app
```

## Capabilities

```toml
[capabilities.net]
fetch = ["https://api.open-meteo.com/*", "https://geocoding-api.open-meteo.com/*"]

[capabilities.sys]
notifications = true
clipboard = true

[capabilities.channels]
allowed = ["*"]
```

## Key Patterns

### Scoped Network Access

Only the specified API domains are accessible:

```typescript
import { fetch } from "runtime:net";

// Allowed - matches fetch pattern
const weather = await fetch("https://api.open-meteo.com/v1/forecast?...");

// Would fail - not in allowed list
// const other = await fetch("https://other-api.com/data");
```

### Geocoding + Weather

```typescript
// Get coordinates from city name
const geoUrl = `https://geocoding-api.open-meteo.com/v1/search?name=${city}`;
const geoData = await (await fetch(geoUrl)).json();
const { latitude, longitude } = geoData.results[0];

// Get weather for coordinates
const weatherUrl = `https://api.open-meteo.com/v1/forecast?latitude=${latitude}&longitude=${longitude}&current_weather=true`;
const weather = await (await fetch(weatherUrl)).json();
```

### Notifications

```typescript
import { notify } from "runtime:sys";

await notify({
  title: "Weather Alert",
  body: `Temperature dropping to ${temp}°C`,
  icon: "./assets/weather-cold.png"
});
```

### Clipboard

```typescript
import { writeClipboard } from "runtime:sys";

await writeClipboard(`Current weather: ${temp}°C, ${description}`);
```

## API Used

This example uses the free [Open-Meteo API](https://open-meteo.com/):
- No API key required
- Global weather data
- Geocoding service included
