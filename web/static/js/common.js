let reflectorData = null;
let ws;

function connect() {
    const wsUrl = `wss://${location.hostname}/ws`;
    ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
        reflectorData = JSON.parse(event.data);
        document.dispatchEvent(new CustomEvent("reflectorUpdate", { detail: reflectorData }));
    };

    ws.onclose = () => setTimeout(connect, 1000);
}

function updateTable(id, arr, mapFn) {
    const tbody = document.querySelector(`#${id} tbody`);
    if (!tbody) return;

    tbody.innerHTML = '';

    if (!arr || arr.length === 0) {
        const row = document.createElement('tr');
        const cell = document.createElement('td');
        cell.colSpan = tbody.parentElement.querySelectorAll('th').length;
        cell.classList.add('has-text-centered');
        cell.textContent = 'No data available';
        row.appendChild(cell);
        tbody.appendChild(row);
        return;
    }

    arr.forEach(item => {
        const row = document.createElement('tr');
        mapFn(item).forEach(val => {
            const cell = document.createElement('td');
            cell.innerHTML = val != null ? val : '';
            row.appendChild(cell);
        });
        tbody.appendChild(row);
    });
}

function formatTime(ts) {
    if (!ts) return '';
    const d = new Date(ts.secs_since_epoch * 1000);
    return d.toLocaleString();
}

function formatDuration(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h}h ${m}m ${s}s`;
}

function initThemeDropdown() {
    const saved = document.cookie.match(/(?:^|; )theme=([^;]*)/);
    const currentTheme = saved ? saved[1] : 'auto';

    document.documentElement.setAttribute('data-theme', currentTheme);

    markActiveTheme(currentTheme);

    document.querySelectorAll('.theme-option').forEach(option => {
        option.addEventListener('click', () => {
            const theme = option.dataset.theme;
            document.documentElement.setAttribute('data-theme', theme);
            document.cookie = `theme=${theme}; path=/; max-age=${60 * 60 * 24 * 365}`;
            markActiveTheme(theme);
        });
    });
}

function markActiveTheme(theme) {
    document.querySelectorAll('.theme-option').forEach(opt => {
        opt.classList.remove('is-active');
    });

    const activeItem = document.querySelector(`.theme-option[data-theme="${theme}"]`);
    if (activeItem) activeItem.classList.add('is-active');
}

document.addEventListener('DOMContentLoaded', () => {
    initThemeDropdown();
    connect();
});
