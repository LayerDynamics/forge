/**
 * Forge DevTools Panel
 *
 * This script manages the custom DevTools panel for Forge applications,
 * providing real-time monitoring, tracing, debugging, and runtime information.
 */
(function() {
    'use strict';

    // ==========================================================================
    // Configuration
    // ==========================================================================

    const CONFIG = {
        pollInterval: 1000,         // Metrics polling interval (ms)
        maxLogEntries: 500,         // Maximum entries in log views
        maxSpans: 100,              // Maximum spans to display
        connectionTimeout: 5000,    // CDP connection timeout (ms)
        retryInterval: 3000,        // Connection retry interval (ms)
    };

    // ==========================================================================
    // State
    // ==========================================================================

    const state = {
        connected: false,
        paused: false,
        currentTab: 'monitor',
        metricsInterval: null,
        eventListeners: [],
        spans: {
            active: [],
            finished: []
        },
        signals: {
            events: []
        }
    };

    // ==========================================================================
    // DOM Elements
    // ==========================================================================

    const $ = (selector) => document.querySelector(selector);
    const $$ = (selector) => document.querySelectorAll(selector);

    // ==========================================================================
    // CDP Communication
    // ==========================================================================

    /**
     * Send a CDP message and wait for response.
     * Uses the platform-specific communication channel.
     */
    async function sendCdp(method, params = {}) {
        return new Promise((resolve, reject) => {
            const id = Date.now();
            const message = { id, method, params };

            // Try different communication channels based on platform
            if (typeof chrome !== 'undefined' && chrome.devtools) {
                // Chrome DevTools Extension API
                chrome.devtools.inspectedWindow.eval(
                    `window.__forgeCdp && window.__forgeCdp(${JSON.stringify(message)})`,
                    (result, error) => {
                        if (error) reject(error);
                        else resolve(result || {});
                    }
                );
            } else if (typeof window.__forgeCdpSend === 'function') {
                // Direct injection (WebView2, WebKitGTK)
                window.__forgeCdpSend(message)
                    .then(resolve)
                    .catch(reject);
            } else if (typeof webkit !== 'undefined' && webkit.messageHandlers) {
                // WebKit message handlers (Safari)
                const handler = webkit.messageHandlers.forgeCdp;
                if (handler) {
                    // Set up response listener
                    const responseHandler = (event) => {
                        if (event.detail && event.detail.id === id) {
                            window.removeEventListener('forgeCdpResponse', responseHandler);
                            resolve(event.detail.result || {});
                        }
                    };
                    window.addEventListener('forgeCdpResponse', responseHandler);
                    handler.postMessage(message);

                    // Timeout
                    setTimeout(() => {
                        window.removeEventListener('forgeCdpResponse', responseHandler);
                        reject(new Error('CDP request timeout'));
                    }, CONFIG.connectionTimeout);
                } else {
                    reject(new Error('WebKit handler not available'));
                }
            } else {
                // Fallback: mock response for development
                console.warn('[Forge] CDP not available, using mock data');
                resolve(getMockResponse(method));
            }
        });
    }

    /**
     * Enable a CDP domain
     */
    async function enableDomain(domain) {
        try {
            await sendCdp(`${domain}.enable`);
            console.log(`[Forge] Enabled domain: ${domain}`);
            return true;
        } catch (e) {
            console.error(`[Forge] Failed to enable ${domain}:`, e);
            return false;
        }
    }

    /**
     * Subscribe to CDP events
     */
    function onCdpEvent(event, callback) {
        const listener = { event, callback };
        state.eventListeners.push(listener);

        // Register with CDP if available
        if (typeof window.__forgeCdpOn === 'function') {
            window.__forgeCdpOn(event, callback);
        }
    }

    // ==========================================================================
    // Tab Navigation
    // ==========================================================================

    function initTabs() {
        $$('.tabs button').forEach(btn => {
            btn.addEventListener('click', () => {
                const tabId = btn.dataset.tab;
                switchTab(tabId);
            });
        });
    }

    function switchTab(tabId) {
        // Update tab buttons
        $$('.tabs button').forEach(b => {
            b.classList.toggle('active', b.dataset.tab === tabId);
            b.setAttribute('aria-selected', b.dataset.tab === tabId);
        });

        // Update panels
        $$('.panel').forEach(p => {
            p.classList.toggle('active', p.id === tabId);
        });

        state.currentTab = tabId;

        // Refresh tab content
        refreshCurrentTab();
    }

    function refreshCurrentTab() {
        switch (state.currentTab) {
            case 'monitor':
                refreshMonitor();
                break;
            case 'trace':
                refreshTrace();
                break;
            case 'debugger':
                refreshDebugger();
                break;
            case 'signals':
                refreshSignals();
                break;
            case 'runtime':
                refreshRuntime();
                break;
        }
    }

    // ==========================================================================
    // Monitor Panel
    // ==========================================================================

    async function refreshMonitor() {
        try {
            const metrics = await sendCdp('Forge.Monitor.getMetrics');
            updateMonitorDisplay(metrics);
        } catch (e) {
            console.error('[Forge] Failed to get metrics:', e);
        }
    }

    function updateMonitorDisplay(metrics) {
        // CPU
        if (metrics.cpu) {
            const cpuPercent = metrics.cpu.totalPercent || 0;
            $('#cpu-value').textContent = `${cpuPercent.toFixed(1)}%`;
            $('#cpu-gauge-fill').style.width = `${Math.min(cpuPercent, 100)}%`;
            $('#cpu-cores').textContent = `${metrics.cpu.coreCount || '--'} cores`;
            $('#cpu-system').textContent = `${(metrics.cpu.systemPercent || 0).toFixed(1)}%`;
            $('#cpu-user').textContent = `${(metrics.cpu.userPercent || 0).toFixed(1)}%`;

            // Color coding
            const cpuCard = $('#cpu-card');
            cpuCard.classList.toggle('warning', cpuPercent > 70);
            cpuCard.classList.toggle('critical', cpuPercent > 90);
        }

        // Memory
        if (metrics.memory) {
            const usedMb = (metrics.memory.usedBytes || 0) / (1024 * 1024);
            const totalMb = (metrics.memory.totalBytes || 0) / (1024 * 1024);
            const memPercent = totalMb > 0 ? (usedMb / totalMb * 100) : 0;

            $('#memory-value').textContent = `${usedMb.toFixed(1)} MB`;
            $('#memory-gauge-fill').style.width = `${Math.min(memPercent, 100)}%`;
            $('#memory-total').textContent = `${(totalMb / 1024).toFixed(1)} GB total`;
            $('#memory-available').textContent = `${((metrics.memory.availableBytes || 0) / (1024 * 1024)).toFixed(1)} MB`;
            $('#memory-swap').textContent = `${((metrics.memory.swapUsedBytes || 0) / (1024 * 1024)).toFixed(1)} MB`;

            // Color coding
            const memCard = $('#memory-card');
            memCard.classList.toggle('warning', memPercent > 70);
            memCard.classList.toggle('critical', memPercent > 90);
        }

        // Event Loop
        if (metrics.eventLoop) {
            const latency = metrics.eventLoop.latencyUs || 0;
            $('#event-loop-value').innerHTML = `${latency} <small>us</small>`;
            $('#event-loop-min').textContent = `${metrics.eventLoop.minUs || '--'} us`;
            $('#event-loop-max').textContent = `${metrics.eventLoop.maxUs || '--'} us`;

            // Health indicator
            const health = $('#event-loop-health');
            if (latency < 1000) {
                health.textContent = 'Healthy';
                health.className = 'metric-badge health-good';
            } else if (latency < 10000) {
                health.textContent = 'Degraded';
                health.className = 'metric-badge health-warning';
            } else {
                health.textContent = 'Slow';
                health.className = 'metric-badge health-critical';
            }
        }

        // Network (if available)
        if (metrics.network) {
            $('#network-rx').textContent = `${((metrics.network.rxBytesPerSec || 0) / 1024).toFixed(1)} KB/s`;
            $('#network-tx').textContent = `${((metrics.network.txBytesPerSec || 0) / 1024).toFixed(1)} KB/s`;
            $('#network-interfaces').textContent = `${metrics.network.interfaceCount || '--'} interfaces`;
            $('#network-total-rx').textContent = `${((metrics.network.totalRxBytes || 0) / (1024 * 1024)).toFixed(1)} MB`;
            $('#network-total-tx').textContent = `${((metrics.network.totalTxBytes || 0) / (1024 * 1024)).toFixed(1)} MB`;
        }

        // Subscriptions
        if (metrics.subscriptions) {
            const subContainer = $('#monitor-subscriptions');
            if (metrics.subscriptions.length > 0) {
                subContainer.innerHTML = metrics.subscriptions.map(sub => `
                    <div class="subscription-item">
                        <span class="sub-id">#${sub.id}</span>
                        <span class="sub-type">${sub.type}</span>
                        <span class="sub-interval">${sub.interval}ms</span>
                    </div>
                `).join('');
            } else {
                subContainer.innerHTML = '<div class="empty-state">No active subscriptions</div>';
            }
        }

        // Update timestamp
        updateTimestamp();
    }

    // ==========================================================================
    // Trace Panel
    // ==========================================================================

    async function refreshTrace() {
        try {
            const [activeResp, finishedResp] = await Promise.all([
                sendCdp('Forge.Trace.getActiveSpans'),
                sendCdp('Forge.Trace.getSpans')
            ]);

            state.spans.active = activeResp.spans || [];
            state.spans.finished = (finishedResp.spans || []).slice(-CONFIG.maxSpans);

            updateTraceDisplay();
        } catch (e) {
            console.error('[Forge] Failed to get trace data:', e);
        }
    }

    function updateTraceDisplay() {
        // Stats
        $('#trace-active-count').textContent = state.spans.active.length;
        $('#trace-finished-count').textContent = state.spans.finished.length;

        // Active spans
        const activeContainer = $('#active-spans');
        if (state.spans.active.length > 0) {
            activeContainer.innerHTML = state.spans.active.map(span => `
                <div class="span-item active">
                    <span class="span-indicator running"></span>
                    <span class="span-name">${escapeHtml(span.name)}</span>
                    <span class="span-id">#${span.id}</span>
                    <span class="span-elapsed">${formatDuration(span.elapsedMs)}ms</span>
                </div>
            `).join('');
        } else {
            activeContainer.innerHTML = '<div class="empty-state">No active spans</div>';
        }

        // Finished spans
        const finishedContainer = $('#finished-spans');
        if (state.spans.finished.length > 0) {
            finishedContainer.innerHTML = state.spans.finished.map(span => `
                <div class="span-item finished">
                    <span class="span-indicator ${span.result ? 'success' : 'completed'}"></span>
                    <span class="span-name">${escapeHtml(span.name)}</span>
                    <span class="span-id">#${span.id}</span>
                    <span class="span-duration">${span.durationMs.toFixed(2)}ms</span>
                    ${span.attributes ? `<button class="span-expand" data-span="${span.id}">+</button>` : ''}
                </div>
                ${span.attributes ? `<div class="span-details" id="span-details-${span.id}" hidden>${JSON.stringify(span.attributes, null, 2)}</div>` : ''}
            `).join('');

            // Add expand handlers
            finishedContainer.querySelectorAll('.span-expand').forEach(btn => {
                btn.addEventListener('click', () => {
                    const details = $(`#span-details-${btn.dataset.span}`);
                    if (details) {
                        details.hidden = !details.hidden;
                        btn.textContent = details.hidden ? '+' : '-';
                    }
                });
            });
        } else {
            finishedContainer.innerHTML = '<div class="empty-state">No finished spans</div>';
        }

        updateTimestamp();
    }

    // ==========================================================================
    // Debugger Panel
    // ==========================================================================

    async function refreshDebugger() {
        try {
            const debuggerInfo = await sendCdp('Forge.Runtime.getDebuggerState');
            updateDebuggerDisplay(debuggerInfo);
        } catch (e) {
            console.error('[Forge] Failed to get debugger state:', e);
        }
    }

    function updateDebuggerDisplay(info) {
        // Connection status
        const connEl = $('#debugger-connected');
        connEl.textContent = info.connected ? 'Connected' : 'Disconnected';
        connEl.className = `status-value ${info.connected ? 'connected' : 'disconnected'}`;

        // State
        const stateEl = $('#debugger-state');
        if (info.paused) {
            stateEl.textContent = 'Paused';
            stateEl.className = 'status-value paused';
        } else if (info.enabled) {
            stateEl.textContent = 'Running';
            stateEl.className = 'status-value running';
        } else {
            stateEl.textContent = 'Disabled';
            stateEl.className = 'status-value disabled';
        }

        // Breakpoints
        $('#breakpoint-count').textContent = info.breakpointCount || 0;
        const bpContainer = $('#breakpoints');
        if (info.breakpoints && info.breakpoints.length > 0) {
            bpContainer.innerHTML = info.breakpoints.map(bp => `
                <div class="breakpoint-item">
                    <span class="bp-indicator"></span>
                    <span class="bp-location">${escapeHtml(bp.id)}</span>
                </div>
            `).join('');
        } else {
            bpContainer.innerHTML = '<div class="empty-state">No breakpoints set</div>';
        }

        // Scripts
        $('#script-count').textContent = info.scriptCount || 0;
        const scriptContainer = $('#scripts');
        if (info.scripts && info.scripts.length > 0) {
            scriptContainer.innerHTML = info.scripts.map(script => `
                <div class="script-item">
                    <span class="script-name">${escapeHtml(script.url || script.id)}</span>
                    <span class="script-length">${script.length || '--'} chars</span>
                </div>
            `).join('');
        } else {
            scriptContainer.innerHTML = '<div class="empty-state">No scripts loaded</div>';
        }

        // Call stack
        const stackContainer = $('#callstack');
        if (info.callFrames && info.callFrames.length > 0) {
            stackContainer.innerHTML = info.callFrames.map((frame, i) => `
                <div class="callframe-item">
                    <span class="frame-index">#${i}</span>
                    <span class="frame-function">${escapeHtml(frame.functionName || '(anonymous)')}</span>
                    <span class="frame-location">${escapeHtml(frame.url || '')}:${frame.lineNumber || 0}</span>
                </div>
            `).join('');
        } else {
            stackContainer.innerHTML = '<div class="empty-state">Not paused</div>';
        }

        updateTimestamp();
    }

    // ==========================================================================
    // Signals Panel
    // ==========================================================================

    async function refreshSignals() {
        try {
            const [supportedResp, subsResp] = await Promise.all([
                sendCdp('Forge.Signals.getSupported'),
                sendCdp('Forge.Signals.getSubscriptions')
            ]);

            updateSignalsDisplay(supportedResp, subsResp);
        } catch (e) {
            console.error('[Forge] Failed to get signals data:', e);
        }
    }

    function updateSignalsDisplay(supported, subscriptions) {
        // Platform info
        const platformEl = $('#signals-platform');
        platformEl.textContent = (supported.signals?.length || 0) > 0 ? 'Unix signals supported' : 'Signals not supported on this platform';

        // Supported signals
        const supportedContainer = $('#supported-signals');
        if (supported.signals && supported.signals.length > 0) {
            supportedContainer.innerHTML = supported.signals.map(sig => `
                <span class="signal-badge">${escapeHtml(sig)}</span>
            `).join('');
        } else {
            supportedContainer.innerHTML = '<div class="empty-state">No signals supported</div>';
        }

        // Subscriptions
        $('#signal-subscription-count').textContent = subscriptions.count || 0;
        const subsContainer = $('#signal-subscriptions');
        if (subscriptions.subscriptions && subscriptions.subscriptions.length > 0) {
            subsContainer.innerHTML = subscriptions.subscriptions.map(sub => `
                <div class="subscription-item">
                    <span class="sub-id">#${sub.id}</span>
                    <span class="sub-signals">${(sub.signals || []).join(', ')}</span>
                </div>
            `).join('');
        } else {
            subsContainer.innerHTML = '<div class="empty-state">No active subscriptions</div>';
        }

        updateTimestamp();
    }

    function addSignalEvent(event) {
        state.signals.events.unshift({
            timestamp: Date.now(),
            ...event
        });

        // Trim to max entries
        if (state.signals.events.length > CONFIG.maxLogEntries) {
            state.signals.events = state.signals.events.slice(0, CONFIG.maxLogEntries);
        }

        updateSignalEventsDisplay();
    }

    function updateSignalEventsDisplay() {
        const container = $('#signal-events');
        if (state.signals.events.length > 0) {
            container.innerHTML = state.signals.events.slice(0, 50).map(evt => `
                <div class="log-entry">
                    <span class="log-time">${formatTime(evt.timestamp)}</span>
                    <span class="log-signal">${escapeHtml(evt.signal)}</span>
                </div>
            `).join('');
        } else {
            container.innerHTML = '<div class="empty-state">Waiting for signals...</div>';
        }
    }

    // ==========================================================================
    // Runtime Panel
    // ==========================================================================

    async function refreshRuntime() {
        try {
            const [runtimeInfo, extensionsResp] = await Promise.all([
                sendCdp('Forge.Runtime.getInfo'),
                sendCdp('Forge.Runtime.getExtensions')
            ]);

            updateRuntimeDisplay(runtimeInfo, extensionsResp);
        } catch (e) {
            console.error('[Forge] Failed to get runtime info:', e);
        }
    }

    function updateRuntimeDisplay(info, extensions) {
        // App info
        $('#app-name').textContent = info.appName || '--';
        $('#app-version').textContent = info.appVersion || '--';
        $('#app-pid').textContent = info.pid || '--';

        // Platform info
        $('#platform-os').textContent = info.os || '--';
        $('#platform-arch').textContent = info.arch || '--';
        $('#platform-webview').textContent = info.webviewVersion || '--';

        // Windows
        const windowContainer = $('#window-list');
        if (info.windows && info.windows.length > 0) {
            windowContainer.innerHTML = info.windows.map(win => `
                <div class="window-item">
                    <span class="window-id">${escapeHtml(win.id)}</span>
                    <span class="window-title">${escapeHtml(win.title || 'Untitled')}</span>
                    <span class="window-size">${win.width}x${win.height}</span>
                </div>
            `).join('');
        } else {
            windowContainer.innerHTML = '<div class="empty-state">No windows</div>';
        }

        // Extensions
        $('#extension-count').textContent = extensions.extensions?.length || 0;
        const extContainer = $('#extensions');
        if (extensions.extensions && extensions.extensions.length > 0) {
            extContainer.innerHTML = extensions.extensions.map(ext => `
                <div class="extension-item ${ext.loaded ? 'loaded' : 'not-loaded'}">
                    <span class="ext-name">${escapeHtml(ext.name)}</span>
                    <span class="ext-status ${ext.loaded ? 'active' : 'inactive'}">${ext.loaded ? 'Active' : 'Inactive'}</span>
                </div>
            `).join('');
        } else {
            extContainer.innerHTML = '<div class="empty-state">No extensions loaded</div>';
        }

        updateTimestamp();
    }

    // ==========================================================================
    // Toolbar Actions
    // ==========================================================================

    function initToolbarActions() {
        // Refresh button
        $('#refresh-btn')?.addEventListener('click', () => {
            refreshCurrentTab();
        });

        // Pause button
        $('#pause-btn')?.addEventListener('click', () => {
            state.paused = !state.paused;
            const btn = $('#pause-btn');
            btn.textContent = state.paused ? '\u25B6' : '\u23F8';
            btn.title = state.paused ? 'Resume updates' : 'Pause updates';

            if (!state.paused) {
                startMetricsPolling();
            } else {
                stopMetricsPolling();
            }
        });

        // Clear trace button
        $('#trace-clear-btn')?.addEventListener('click', () => {
            state.spans.finished = [];
            updateTraceDisplay();
        });
    }

    // ==========================================================================
    // Polling
    // ==========================================================================

    function startMetricsPolling() {
        if (state.metricsInterval) {
            clearInterval(state.metricsInterval);
        }

        state.metricsInterval = setInterval(() => {
            if (!state.paused && state.currentTab === 'monitor') {
                refreshMonitor();
            }
        }, CONFIG.pollInterval);
    }

    function stopMetricsPolling() {
        if (state.metricsInterval) {
            clearInterval(state.metricsInterval);
            state.metricsInterval = null;
        }
    }

    // ==========================================================================
    // Utility Functions
    // ==========================================================================

    function escapeHtml(str) {
        if (typeof str !== 'string') return '';
        const div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    }

    function formatDuration(ms) {
        if (ms < 1) return ms.toFixed(3);
        if (ms < 10) return ms.toFixed(2);
        if (ms < 100) return ms.toFixed(1);
        return Math.round(ms);
    }

    function formatTime(timestamp) {
        const d = new Date(timestamp);
        return d.toLocaleTimeString(undefined, {
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            fractionalSecondDigits: 3
        });
    }

    function updateTimestamp() {
        const el = $('#update-time');
        if (el) {
            el.textContent = `Last update: ${formatTime(Date.now())}`;
        }
    }

    function updateConnectionStatus(connected) {
        state.connected = connected;
        const indicator = $('#connection-status');
        if (indicator) {
            indicator.classList.toggle('connected', connected);
            indicator.classList.toggle('disconnected', !connected);
            indicator.title = connected ? 'Connected to runtime' : 'Disconnected';
        }
    }

    // ==========================================================================
    // Mock Data (for development)
    // ==========================================================================

    function getMockResponse(method) {
        const mocks = {
            'Forge.Monitor.getMetrics': {
                cpu: { totalPercent: 15.5, coreCount: 8, systemPercent: 5.2, userPercent: 10.3 },
                memory: { usedBytes: 256 * 1024 * 1024, totalBytes: 16 * 1024 * 1024 * 1024, availableBytes: 12 * 1024 * 1024 * 1024 },
                eventLoop: { latencyUs: 250, minUs: 100, maxUs: 1500 }
            },
            'Forge.Trace.getSpans': {
                spans: [
                    { id: 1, name: 'fetch_data', durationMs: 125.5 },
                    { id: 2, name: 'render_view', durationMs: 32.1 }
                ]
            },
            'Forge.Trace.getActiveSpans': { spans: [] },
            'Forge.Runtime.getDebuggerState': {
                connected: false,
                enabled: false,
                paused: false,
                breakpointCount: 0,
                scriptCount: 0
            },
            'Forge.Signals.getSupported': {
                signals: ['SIGINT', 'SIGTERM', 'SIGHUP', 'SIGUSR1', 'SIGUSR2']
            },
            'Forge.Signals.getSubscriptions': {
                count: 0,
                subscriptions: []
            },
            'Forge.Runtime.getInfo': {
                appName: 'Forge App',
                appVersion: '1.0.0',
                pid: 12345,
                os: 'darwin',
                arch: 'arm64'
            },
            'Forge.Runtime.getExtensions': {
                extensions: [
                    { name: 'ext_fs', loaded: true },
                    { name: 'ext_window', loaded: true },
                    { name: 'ext_monitor', loaded: true }
                ]
            }
        };

        return mocks[method] || {};
    }

    // ==========================================================================
    // Initialization
    // ==========================================================================

    async function init() {
        console.log('[Forge] Initializing DevTools panel');

        // Initialize UI
        initTabs();
        initToolbarActions();

        // Enable CDP domains
        await Promise.all([
            enableDomain('Forge.Monitor'),
            enableDomain('Forge.Trace'),
            enableDomain('Forge.Signals'),
            enableDomain('Forge.Runtime')
        ]);

        // Subscribe to events
        onCdpEvent('Forge.Monitor.metricsUpdate', (params) => {
            if (state.currentTab === 'monitor') {
                updateMonitorDisplay(params);
            }
        });

        onCdpEvent('Forge.Trace.spanStarted', (params) => {
            state.spans.active.push(params.span);
            if (state.currentTab === 'trace') {
                updateTraceDisplay();
            }
        });

        onCdpEvent('Forge.Trace.spanFinished', (params) => {
            state.spans.active = state.spans.active.filter(s => s.id !== params.span.id);
            state.spans.finished.push(params.span);
            if (state.currentTab === 'trace') {
                updateTraceDisplay();
            }
        });

        onCdpEvent('Forge.Signals.signalReceived', (params) => {
            addSignalEvent(params);
        });

        // Initial data load
        updateConnectionStatus(true);
        await refreshCurrentTab();

        // Start polling
        startMetricsPolling();

        console.log('[Forge] DevTools panel initialized');
    }

    // Export for external initialization
    window.__forgeInit = init;

    // Auto-init when ready
    if (document.readyState === 'complete' || document.readyState === 'interactive') {
        init();
    } else {
        document.addEventListener('DOMContentLoaded', init);
    }

})();
