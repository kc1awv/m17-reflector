document.addEventListener("reflectorUpdate", (e) => {
    const s = e.detail;

    document.getElementById('reflectorTitle').textContent = `${s.reflector_name} Dashboard`;
    document.title = `${s.reflector_name} Dashboard`;
    document.getElementById('uptime').textContent = formatDuration(s.uptime_seconds);
    document.getElementById('totalClients').textContent = s.total_clients;
    document.getElementById('totalStreams').textContent = s.total_streams;

    updateTable('modulesTable', s.modules.sort((a, b) => a.module.localeCompare(b.module)), m => [
        m.module,
        m.clients,
        m.active_streams > 0
            ? `<span class="tag is-success">Active</span>`
            : `<span class="tag is-dark">Inactive</span>`
    ]);

    updateTable('activeStreamsTable', s.active_streams, a => [
        a.source,
        a.peer || '-',
        a.destination,
        a.module,
        a.stream_id,
        formatDuration(Math.floor(Date.now() / 1000) - a.started_at.secs_since_epoch)
    ]);

    updateTable(
        'recentStreamsTable',
        s.recent_streams.sort((a, b) => b.ended_at.secs_since_epoch - a.ended_at.secs_since_epoch),
        r => [
            r.source,
            r.peer || '-',
            r.destination,
            r.module,
            formatTime(r.ended_at)
        ]
    );
});
