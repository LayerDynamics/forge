// Forge Weather App - Main Deno entry point
// Demonstrates: runtime:net HTTP fetch, runtime:sys notifications, runtime:window tray icons

import { createWindow, tray, menu } from "runtime:window";
import { windowEvents, sendToWindow } from "runtime:ipc";
import { fetchJson } from "runtime:net";
import { notify, clipboard } from "runtime:sys";

// Types for Open-Meteo API
interface GeocodingResult {
  results?: Array<{
    id: number;
    name: string;
    latitude: number;
    longitude: number;
    country: string;
    admin1?: string;
  }>;
}

interface WeatherData {
  current: {
    temperature_2m: number;
    relative_humidity_2m: number;
    apparent_temperature: number;
    weather_code: number;
    wind_speed_10m: number;
    wind_direction_10m: number;
  };
  daily: {
    time: string[];
    weather_code: number[];
    temperature_2m_max: number[];
    temperature_2m_min: number[];
  };
}

interface Location {
  name: string;
  country: string;
  lat: number;
  lon: number;
}

interface WeatherState {
  location: Location | null;
  weather: WeatherData | null;
  loading: boolean;
  error: string | null;
}

// Weather code to description mapping
const WEATHER_CODES: Record<number, { description: string; icon: string }> = {
  0: { description: "Clear sky", icon: "sun" },
  1: { description: "Mainly clear", icon: "sun" },
  2: { description: "Partly cloudy", icon: "cloud-sun" },
  3: { description: "Overcast", icon: "cloud" },
  45: { description: "Fog", icon: "smog" },
  48: { description: "Depositing rime fog", icon: "smog" },
  51: { description: "Light drizzle", icon: "cloud-rain" },
  53: { description: "Moderate drizzle", icon: "cloud-rain" },
  55: { description: "Dense drizzle", icon: "cloud-rain" },
  61: { description: "Slight rain", icon: "cloud-rain" },
  63: { description: "Moderate rain", icon: "cloud-showers-heavy" },
  65: { description: "Heavy rain", icon: "cloud-showers-heavy" },
  71: { description: "Slight snow", icon: "snowflake" },
  73: { description: "Moderate snow", icon: "snowflake" },
  75: { description: "Heavy snow", icon: "snowflake" },
  77: { description: "Snow grains", icon: "snowflake" },
  80: { description: "Slight rain showers", icon: "cloud-sun-rain" },
  81: { description: "Moderate rain showers", icon: "cloud-sun-rain" },
  82: { description: "Violent rain showers", icon: "cloud-showers-heavy" },
  95: { description: "Thunderstorm", icon: "bolt" },
  96: { description: "Thunderstorm with hail", icon: "bolt" },
  99: { description: "Thunderstorm with heavy hail", icon: "bolt" },
};

function getWeatherInfo(code: number) {
  return WEATHER_CODES[code] || { description: "Unknown", icon: "question" };
}

// Geocode a location name
async function geocodeLocation(query: string): Promise<Location[]> {
  const url = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(query)}&count=5&language=en&format=json`;
  const data = await fetchJson<GeocodingResult>(url);

  if (!data.results || data.results.length === 0) {
    return [];
  }

  return data.results.map(r => ({
    name: r.name,
    country: r.country,
    lat: r.latitude,
    lon: r.longitude,
  }));
}

// Fetch weather for coordinates
async function fetchWeather(lat: number, lon: number): Promise<WeatherData> {
  const url = `https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m,wind_direction_10m&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto`;
  return await fetchJson<WeatherData>(url);
}

// Main application
async function main() {
  console.log("Forge Weather App starting...");

  let state: WeatherState = {
    location: null,
    weather: null,
    loading: false,
    error: null,
  };

  // Create tray icon
  const trayIcon = await tray.create({
    tooltip: "Forge Weather",
    menu: [
      { id: "refresh", label: "Refresh" },
      { id: "separator", label: "-", type: "separator" },
      { id: "quit", label: "Quit" },
    ],
  });

  // Open main window
  const win = await createWindow({
    url: "app://index.html",
    width: 400,
    height: 500,
    title: "Forge Weather",
  });

  console.log(`Window opened with ID: ${win.id}`);

  // Send state to renderer
  function sendState() {
    const weatherInfo = state.weather?.current
      ? getWeatherInfo(state.weather.current.weather_code)
      : null;

    sendToWindow(win.id, "state", {
      ...state,
      weatherDescription: weatherInfo?.description,
      weatherIcon: weatherInfo?.icon,
    });
  }

  // Update tray with current temperature
  function updateTray() {
    if (state.weather?.current) {
      const temp = Math.round(state.weather.current.temperature_2m);
      trayIcon.update({
        tooltip: `${state.location?.name}: ${temp}°C`,
        menu: [
          { id: "temp", label: `${temp}°C - ${getWeatherInfo(state.weather.current.weather_code).description}`, enabled: false },
          { id: "separator", label: "-", type: "separator" },
          { id: "refresh", label: "Refresh" },
          { id: "separator2", label: "-", type: "separator" },
          { id: "quit", label: "Quit" },
        ],
      });
    }
  }

  // Fetch weather for current location
  async function refreshWeather() {
    if (!state.location) return;

    state.loading = true;
    state.error = null;
    sendState();

    try {
      state.weather = await fetchWeather(state.location.lat, state.location.lon);
      updateTray();

      // Check for severe weather and notify
      const code = state.weather.current.weather_code;
      if (code >= 95) {
        await notify(
          "Severe Weather Alert",
          `${getWeatherInfo(code).description} in ${state.location.name}`
        );
      }
    } catch (e) {
      state.error = `Failed to fetch weather: ${e}`;
      console.error(state.error);
    } finally {
      state.loading = false;
      sendState();
    }
  }

  // Handle menu events
  menu.onMenu(async (event) => {
    console.log("Menu event:", event);

    switch (event.itemId) {
      case "refresh":
        await refreshWeather();
        break;
      case "quit":
        trayIcon.destroy();
        win.close();
        Deno.exit(0);
        break;
    }
  });

  // Handle window events
  for await (const event of windowEvents()) {
    console.log("Window event:", event.channel, event.payload);

    switch (event.channel) {
      case "ready":
        sendState();
        break;

      case "search": {
        const query = event.payload as string;
        if (!query.trim()) break;

        state.loading = true;
        state.error = null;
        sendState();

        try {
          const locations = await geocodeLocation(query);
          sendToWindow(win.id, "search-results", locations);
        } catch (e) {
          state.error = `Search failed: ${e}`;
        } finally {
          state.loading = false;
          sendState();
        }
        break;
      }

      case "select-location": {
        const location = event.payload as Location;
        state.location = location;
        sendState();
        await refreshWeather();
        break;
      }

      case "refresh":
        await refreshWeather();
        break;

      case "copy-weather": {
        if (state.weather?.current && state.location) {
          const temp = state.weather.current.temperature_2m;
          const desc = getWeatherInfo(state.weather.current.weather_code).description;
          const text = `Weather in ${state.location.name}: ${temp}°C, ${desc}`;
          await clipboard.write(text);
          await notify("Copied", "Weather info copied to clipboard");
        }
        break;
      }
    }
  }
}

main().catch(console.error);
