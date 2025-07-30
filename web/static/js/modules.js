let lastPacketsMap = new Map();
window.hasInitialUpdate = false;

document.addEventListener("reflectorUpdate", (e) => {
    const data = e.detail;
    const container = document.getElementById("modulesContainer");
    container.innerHTML = "";

    document.getElementById('reflectorTitle').textContent = `${data.reflector_name} Dashboard`;
    document.title = `${data.reflector_name} Dashboard`;

    const sortedModules = [...data.modules].sort((a, b) =>
        a.module.localeCompare(b.module)
    );

    const activePeerMap = new Map();
    data.active_streams.forEach(stream => {
        activePeerMap.set(stream.peer.trim(), stream.source.trim());
    });

    sortedModules.forEach(mod => {
        let moduleStatusTag;
        if (!window.hasInitialUpdate) {
            moduleStatusTag = `<span class="tag is-warning">Loading...</span>`;
        } else {
            moduleStatusTag = mod.active_streams > 0
                ? `<span class="tag is-success">Active</span>`
                : `<span class="tag is-dark">Idle</span>`;
        }

        const col = document.createElement("div");
        col.className = "column is-one-third";

        const moduleBox = document.createElement("div");
        moduleBox.className = "box";

        moduleBox.innerHTML = `
      <div class="module-header">
        <h2 class="subtitle">Module ${mod.module} (Peers: ${mod.clients})</h2>
        ${moduleStatusTag}
      </div>
    `;

        const peers = data.clients
            .filter(c => c.module === mod.module)
            .sort((a, b) => a.callsign.localeCompare(b.callsign));

        if (peers.length > 0) {
            const peerTable = document.createElement("table");
            peerTable.className = "table is-fullwidth is-striped peer-table";
            peerTable.innerHTML = `
        <thead>
          <tr>
            <th>Peer</th>
            <th>Packets</th>
            <th>Bytes</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          ${peers.map(p => {
                const lastPackets = lastPacketsMap.get(p.callsign) || 0;
                const isPacketActive = p.packets_in > lastPackets;
                lastPacketsMap.set(p.callsign, p.packets_in);

                const activeSource = activePeerMap.get(p.callsign.trim());
                let statusHTML;

                if (!window.hasInitialUpdate) {
                    statusHTML = `<span class="tag is-warning">Loading...</span>`;
                } else if (activeSource) {
                    statusHTML = `<span class="tag is-success">Active: ${activeSource}</span>`;
                } else if (isPacketActive) {
                    statusHTML = `<span class="tag is-success">Active</span>`;
                } else {
                    statusHTML = `<span class="tag is-dark">Connected: ${formatElapsed(p.connected_since)}</span>`;
                }

                return `
              <tr>
                <td>${p.callsign}</td>
                <td>${p.packets_in}</td>
                <td>${formatBytes(p.bytes_in)}</td>
                <td>${statusHTML}</td>
              </tr>
            `;
            }).join('')}
        </tbody>
      `;
            moduleBox.appendChild(peerTable);
        }

        col.appendChild(moduleBox);
        container.appendChild(col);
    });

    if (!window.hasInitialUpdate) {
        window.hasInitialUpdate = true;
    }
});

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const value = bytes / Math.pow(1024, i);
    return `${value.toFixed(1)} ${sizes[i]}`;
}

function formatElapsed(ts) {
    if (!ts) return '-';
    const now = Math.floor(Date.now() / 1000);
    const connected = ts.secs_since_epoch;
    let seconds = now - connected;

    const hours = Math.floor(seconds / 3600);
    seconds %= 3600;
    const minutes = Math.floor(seconds / 60);
    seconds %= 60;

    if (hours > 0) {
        return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
        return `${minutes}m ${seconds}s`;
    } else {
        return `${seconds}s`;
    }
}
