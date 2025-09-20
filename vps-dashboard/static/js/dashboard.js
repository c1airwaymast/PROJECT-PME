// 🔥 VPS Dashboard Ultra-Performant - JavaScript 🔥

class UltraPerformanceDashboard {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 10;
        this.reconnectInterval = 1000;
        this.debugMode = false;
        this.charts = {};
        this.lastMetrics = {};
        this.alertsQueue = [];
        this.logsPaused = false;
        
        this.init();
    }
    
    init() {
        console.log('🚀 Initialisation du Dashboard Ultra-Performant');
        this.setupEventListeners();
        this.initCharts();
        this.connectWebSocket();
        this.startPerformanceMonitoring();
    }
    
    setupEventListeners() {
        // Bouton debug
        document.getElementById('debugToggle').addEventListener('click', () => {
            this.toggleDebugMode();
        });
        
        // Bouton refresh
        document.getElementById('refreshBtn').addEventListener('click', () => {
            this.refreshData();
        });
        
        // Bouton fullscreen
        document.getElementById('fullscreenBtn').addEventListener('click', () => {
            this.toggleFullscreen();
        });
        
        // Debug panel
        document.getElementById('debugClose').addEventListener('click', () => {
            this.closeDebugPanel();
        });
        
        // Debug tabs
        document.querySelectorAll('.debug-tab').forEach(tab => {
            tab.addEventListener('click', (e) => {
                this.switchDebugTab(e.target.dataset.tab);
            });
        });
        
        // Logs controls
        document.getElementById('clearLogs').addEventListener('click', () => {
            this.clearLogs();
        });
        
        document.getElementById('pauseLogs').addEventListener('click', () => {
            this.toggleLogsPause();
        });
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey && e.key === 'd') {
                e.preventDefault();
                this.toggleDebugMode();
            }
            if (e.key === 'F11') {
                e.preventDefault();
                this.toggleFullscreen();
            }
        });
        
        // Auto-hide alerts
        setInterval(() => {
            this.cleanupOldAlerts();
        }, 5000);
    }
    
    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.hostname}:8765`;
        
        console.log(`🔌 Connexion WebSocket: ${wsUrl}`);
        
        try {
            this.ws = new WebSocket(wsUrl);
            
            this.ws.onopen = () => {
                console.log('✅ WebSocket connecté');
                this.updateConnectionStatus(true);
                this.reconnectAttempts = 0;
                this.addLog('Connexion WebSocket établie', 'info');
            };
            
            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handleWebSocketMessage(data);
                } catch (error) {
                    console.error('❌ Erreur parsing WebSocket:', error);
                    this.addLog(`Erreur parsing: ${error.message}`, 'error');
                }
            };
            
            this.ws.onclose = () => {
                console.log('🔌 WebSocket fermé');
                this.updateConnectionStatus(false);
                this.addLog('Connexion WebSocket fermée', 'warning');
                this.attemptReconnect();
            };
            
            this.ws.onerror = (error) => {
                console.error('❌ Erreur WebSocket:', error);
                this.addLog(`Erreur WebSocket: ${error.message || 'Erreur inconnue'}`, 'error');
            };
            
        } catch (error) {
            console.error('❌ Erreur création WebSocket:', error);
            this.addLog(`Erreur création WebSocket: ${error.message}`, 'error');
        }
    }
    
    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            const delay = this.reconnectInterval * Math.pow(1.5, this.reconnectAttempts - 1);
            
            console.log(`🔄 Tentative de reconnexion ${this.reconnectAttempts}/${this.maxReconnectAttempts} dans ${delay}ms`);
            this.addLog(`Reconnexion dans ${Math.round(delay/1000)}s (${this.reconnectAttempts}/${this.maxReconnectAttempts})`, 'warning');
            
            setTimeout(() => {
                this.connectWebSocket();
            }, delay);
        } else {
            console.error('❌ Impossible de se reconnecter après', this.maxReconnectAttempts, 'tentatives');
            this.addLog('Impossible de se reconnecter. Veuillez actualiser la page.', 'error');
        }
    }
    
    handleWebSocketMessage(data) {
        switch (data.type) {
            case 'initial_data':
            case 'metrics_update':
                this.updateMetrics(data.data);
                break;
                
            case 'debug_data':
                this.updateDebugInfo(data.data);
                break;
                
            case 'debug_toggled':
                this.debugMode = data.enabled;
                this.updateDebugButton();
                break;
                
            case 'history_data':
                this.updateHistoryCharts(data.data);
                break;
                
            case 'error':
                this.addLog(`Erreur serveur: ${data.message}`, 'error');
                this.showAlert('error', data.message);
                break;
                
            default:
                console.warn('🤷 Type de message inconnu:', data.type);
        }
    }
    
    updateMetrics(metrics) {
        this.lastMetrics = metrics;
        
        // CPU
        if (metrics.cpu) {
            this.updateCPUMetrics(metrics.cpu);
        }
        
        // Mémoire
        if (metrics.memory) {
            this.updateMemoryMetrics(metrics.memory);
        }
        
        // Disque
        if (metrics.disk) {
            this.updateDiskMetrics(metrics.disk);
        }
        
        // Réseau
        if (metrics.network) {
            this.updateNetworkMetrics(metrics.network);
        }
        
        // Processus
        if (metrics.processes) {
            this.updateProcessesTable(metrics.processes);
        }
        
        // Système
        if (metrics.system) {
            this.updateSystemInfo(metrics.system);
        }
        
        // GPU
        if (metrics.gpu && metrics.gpu.gpus.length > 0) {
            this.updateGPUInfo(metrics.gpu);
        }
        
        // Alertes
        if (metrics.alerts) {
            this.handleAlerts(metrics.alerts);
        }
        
        // Mise à jour des graphiques
        this.updateCharts(metrics);
    }
    
    updateCPUMetrics(cpu) {
        const percent = cpu.total_percent || 0;
        
        // Valeurs principales
        document.getElementById('cpuValue').textContent = `${percent.toFixed(1)}%`;
        document.getElementById('cpuPercent').textContent = `${percent.toFixed(1)}%`;
        
        // Anneau de progression
        const ring = document.getElementById('cpuRing');
        const circumference = 2 * Math.PI * 50;
        const offset = circumference - (percent / 100) * circumference;
        ring.style.strokeDashoffset = offset;
        
        // Couleur selon l'utilisation
        const card = document.querySelector('.cpu-card');
        this.updateCardStatus(card, percent);
        
        // Détails
        if (cpu.load_avg) {
            document.getElementById('loadAvg').textContent = cpu.load_avg.join(', ');
        }
        document.getElementById('cpuCores').textContent = `${cpu.count_physical}/${cpu.count_logical}`;
        
        // Détails des cœurs
        const cpuDetails = document.getElementById('cpuDetails');
        if (cpu.cores && cpu.cores.length > 0) {
            let coresHtml = '<div class="cores-grid">';
            cpu.cores.forEach((core, index) => {
                coresHtml += `
                    <div class="core-item">
                        <span>Core ${index}:</span>
                        <span>${core.percent.toFixed(1)}%</span>
                    </div>
                `;
            });
            coresHtml += '</div>';
            
            const existingCores = cpuDetails.querySelector('.cores-grid');
            if (existingCores) {
                existingCores.innerHTML = coresHtml;
            } else {
                cpuDetails.insertAdjacentHTML('beforeend', coresHtml);
            }
        }
    }
    
    updateMemoryMetrics(memory) {
        const percent = memory.percent || 0;
        const usedGB = (memory.used / (1024**3)).toFixed(2);
        const totalGB = (memory.total / (1024**3)).toFixed(2);
        const availableGB = (memory.available / (1024**3)).toFixed(2);
        
        // Valeurs principales
        document.getElementById('memoryValue').textContent = `${percent.toFixed(1)}%`;
        document.getElementById('memoryPercent').textContent = `${percent.toFixed(1)}%`;
        
        // Anneau de progression
        const ring = document.getElementById('memoryRing');
        const circumference = 2 * Math.PI * 50;
        const offset = circumference - (percent / 100) * circumference;
        ring.style.strokeDashoffset = offset;
        
        // Couleur selon l'utilisation
        const card = document.querySelector('.memory-card');
        this.updateCardStatus(card, percent);
        
        // Détails
        document.getElementById('memoryUsed').textContent = `${usedGB} GB`;
        document.getElementById('memoryAvailable').textContent = `${availableGB} GB`;
        
        // Swap
        if (memory.swap) {
            const memoryDetails = document.getElementById('memoryDetails');
            let swapInfo = memoryDetails.querySelector('.swap-info');
            if (!swapInfo) {
                swapInfo = document.createElement('div');
                swapInfo.className = 'swap-info';
                memoryDetails.appendChild(swapInfo);
            }
            
            const swapUsedGB = (memory.swap.used / (1024**3)).toFixed(2);
            const swapTotalGB = (memory.swap.total / (1024**3)).toFixed(2);
            
            swapInfo.innerHTML = `
                <div class="detail-item">
                    <span>Swap:</span>
                    <span>${swapUsedGB}/${swapTotalGB} GB</span>
                </div>
            `;
        }
    }
    
    updateDiskMetrics(disk) {
        let maxPercent = 0;
        let totalUsed = 0;
        let totalSize = 0;
        
        // Calcul des totaux
        if (disk.partitions) {
            Object.values(disk.partitions).forEach(partition => {
                maxPercent = Math.max(maxPercent, partition.percent || 0);
                totalUsed += partition.used || 0;
                totalSize += partition.total || 0;
            });
        }
        
        const percent = totalSize > 0 ? (totalUsed / totalSize) * 100 : 0;
        
        // Valeurs principales
        document.getElementById('diskValue').textContent = `${percent.toFixed(1)}%`;
        document.getElementById('diskPercent').textContent = `${percent.toFixed(1)}%`;
        
        // Anneau de progression
        const ring = document.getElementById('diskRing');
        const circumference = 2 * Math.PI * 50;
        const offset = circumference - (percent / 100) * circumference;
        ring.style.strokeDashoffset = offset;
        
        // Couleur selon l'utilisation
        const card = document.querySelector('.disk-card');
        this.updateCardStatus(card, maxPercent);
        
        // Détails des partitions
        const diskDetails = document.getElementById('diskDetails');
        diskDetails.innerHTML = '';
        
        if (disk.partitions) {
            Object.entries(disk.partitions).forEach(([device, partition]) => {
                const usedGB = (partition.used / (1024**3)).toFixed(1);
                const totalGB = (partition.total / (1024**3)).toFixed(1);
                
                diskDetails.insertAdjacentHTML('beforeend', `
                    <div class="detail-item">
                        <span>${device}:</span>
                        <span>${usedGB}/${totalGB} GB (${partition.percent.toFixed(1)}%)</span>
                    </div>
                `);
            });
        }
    }
    
    updateNetworkMetrics(network) {
        let totalBytesSent = 0;
        let totalBytesRecv = 0;
        
        // Calcul des totaux
        if (network.interfaces) {
            Object.values(network.interfaces).forEach(iface => {
                totalBytesSent += iface.bytes_sent || 0;
                totalBytesRecv += iface.bytes_recv || 0;
            });
        }
        
        // Calcul des vitesses (approximation)
        const now = Date.now();
        if (this.lastNetworkUpdate) {
            const timeDiff = (now - this.lastNetworkUpdate.time) / 1000;
            const sentDiff = totalBytesSent - this.lastNetworkUpdate.sent;
            const recvDiff = totalBytesRecv - this.lastNetworkUpdate.recv;
            
            const sentSpeed = Math.max(0, sentDiff / timeDiff);
            const recvSpeed = Math.max(0, recvDiff / timeDiff);
            
            document.getElementById('networkUp').textContent = this.formatBytes(sentSpeed) + '/s';
            document.getElementById('networkDown').textContent = this.formatBytes(recvSpeed) + '/s';
            
            const totalSpeed = sentSpeed + recvSpeed;
            document.getElementById('networkValue').textContent = this.formatBytes(totalSpeed) + '/s';
        }
        
        this.lastNetworkUpdate = {
            time: now,
            sent: totalBytesSent,
            recv: totalBytesRecv
        };
        
        // Détails
        if (network.connections_count !== undefined) {
            document.getElementById('networkConnections').textContent = network.connections_count;
        }
        
        if (network.public_ip) {
            document.getElementById('publicIp').textContent = network.public_ip;
        }
    }
    
    updateProcessesTable(processes) {
        document.getElementById('processCount').textContent = processes.total_count || 0;
        
        const tbody = document.getElementById('processesBody');
        tbody.innerHTML = '';
        
        if (processes.top_processes) {
            processes.top_processes.forEach(proc => {
                const row = document.createElement('tr');
                row.innerHTML = `
                    <td>${proc.pid}</td>
                    <td title="${proc.name}">${proc.name.length > 15 ? proc.name.substring(0, 15) + '...' : proc.name}</td>
                    <td>${proc.cpu_percent.toFixed(1)}%</td>
                    <td>${proc.memory_mb.toFixed(1)}</td>
                    <td><span class="process-status ${proc.status}">${proc.status}</span></td>
                `;
                tbody.appendChild(row);
            });
        }
    }
    
    updateSystemInfo(system) {
        const systemInfo = document.getElementById('systemInfo');
        systemInfo.innerHTML = '';
        
        const info = [
            { label: 'Hostname', value: system.hostname },
            { label: 'OS', value: system.platform },
            { label: 'Architecture', value: system.architecture },
            { label: 'Uptime', value: system.uptime_string },
            { label: 'Kernel', value: system.kernel_version },
            { label: 'Python', value: system.python_version }
        ];
        
        info.forEach(item => {
            if (item.value) {
                systemInfo.insertAdjacentHTML('beforeend', `
                    <div class="system-info-item">
                        <span class="system-info-label">${item.label}:</span>
                        <span class="system-info-value">${item.value}</span>
                    </div>
                `);
            }
        });
        
        // Utilisateurs connectés
        if (system.users && system.users.length > 0) {
            systemInfo.insertAdjacentHTML('beforeend', `
                <div class="system-info-item">
                    <span class="system-info-label">Utilisateurs:</span>
                    <span class="system-info-value">${system.users.length}</span>
                </div>
            `);
        }
    }
    
    updateGPUInfo(gpu) {
        const gpuCard = document.getElementById('gpuCard');
        const gpuInfo = document.getElementById('gpuInfo');
        
        if (gpu.gpus && gpu.gpus.length > 0) {
            gpuCard.style.display = 'block';
            gpuInfo.innerHTML = '';
            
            gpu.gpus.forEach((g, index) => {
                gpuInfo.insertAdjacentHTML('beforeend', `
                    <div class="gpu-item">
                        <h4>${g.name}</h4>
                        <div class="gpu-metrics">
                            <div class="gpu-metric">
                                <span>Utilisation:</span>
                                <span>${g.load.toFixed(1)}%</span>
                            </div>
                            <div class="gpu-metric">
                                <span>Mémoire:</span>
                                <span>${g.memory_used}/${g.memory_total} MB (${g.memory_percent.toFixed(1)}%)</span>
                            </div>
                            <div class="gpu-metric">
                                <span>Température:</span>
                                <span>${g.temperature}°C</span>
                            </div>
                        </div>
                    </div>
                `);
            });
        }
    }
    
    updateCardStatus(card, percent) {
        card.classList.remove('low-usage', 'medium-usage', 'high-usage');
        
        if (percent >= 85) {
            card.classList.add('high-usage');
        } else if (percent >= 70) {
            card.classList.add('medium-usage');
        } else {
            card.classList.add('low-usage');
        }
    }
    
    handleAlerts(alerts) {
        alerts.forEach(alert => {
            this.showAlert(alert.type, alert.message);
        });
    }
    
    showAlert(type, message) {
        const alertsContainer = document.getElementById('alertsContainer');
        const alertId = Date.now() + Math.random();
        
        const alertElement = document.createElement('div');
        alertElement.className = `alert ${type}`;
        alertElement.dataset.alertId = alertId;
        alertElement.innerHTML = `
            <i class="fas fa-exclamation-triangle"></i>
            <span>${message}</span>
            <button class="alert-close" onclick="this.parentElement.remove()">
                <i class="fas fa-times"></i>
            </button>
        `;
        
        alertsContainer.appendChild(alertElement);
        
        // Auto-suppression après 10 secondes
        setTimeout(() => {
            const alert = document.querySelector(`[data-alert-id="${alertId}"]`);
            if (alert) {
                alert.remove();
            }
        }, 10000);
    }
    
    cleanupOldAlerts() {
        const alerts = document.querySelectorAll('.alert');
        if (alerts.length > 5) {
            // Garde seulement les 5 alertes les plus récentes
            for (let i = 0; i < alerts.length - 5; i++) {
                alerts[i].remove();
            }
        }
    }
    
    initCharts() {
        // Configuration commune des graphiques
        const commonConfig = {
            type: 'line',
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: false
                    }
                },
                scales: {
                    x: {
                        display: false
                    },
                    y: {
                        display: false,
                        min: 0,
                        max: 100
                    }
                },
                elements: {
                    point: {
                        radius: 0
                    },
                    line: {
                        tension: 0.4,
                        borderWidth: 2
                    }
                },
                animation: {
                    duration: 0
                }
            }
        };
        
        // Graphique CPU
        const cpuCtx = document.getElementById('cpuChart').getContext('2d');
        this.charts.cpu = new Chart(cpuCtx, {
            ...commonConfig,
            data: {
                labels: Array(60).fill(''),
                datasets: [{
                    data: Array(60).fill(0),
                    borderColor: '#ff6b6b',
                    backgroundColor: 'rgba(255, 107, 107, 0.1)',
                    fill: true
                }]
            }
        });
        
        // Graphique Mémoire
        const memoryCtx = document.getElementById('memoryChart').getContext('2d');
        this.charts.memory = new Chart(memoryCtx, {
            ...commonConfig,
            data: {
                labels: Array(60).fill(''),
                datasets: [{
                    data: Array(60).fill(0),
                    borderColor: '#4834d4',
                    backgroundColor: 'rgba(72, 52, 212, 0.1)',
                    fill: true
                }]
            }
        });
        
        // Graphique Disque
        const diskCtx = document.getElementById('diskChart').getContext('2d');
        this.charts.disk = new Chart(diskCtx, {
            ...commonConfig,
            data: {
                labels: Array(60).fill(''),
                datasets: [{
                    data: Array(60).fill(0),
                    borderColor: '#ff9ff3',
                    backgroundColor: 'rgba(255, 159, 243, 0.1)',
                    fill: true
                }]
            }
        });
        
        // Graphique Réseau
        const networkCtx = document.getElementById('networkChart').getContext('2d');
        this.charts.network = new Chart(networkCtx, {
            ...commonConfig,
            data: {
                labels: Array(60).fill(''),
                datasets: [
                    {
                        label: 'Upload',
                        data: Array(60).fill(0),
                        borderColor: '#00d2d3',
                        backgroundColor: 'rgba(0, 210, 211, 0.1)',
                        fill: false
                    },
                    {
                        label: 'Download',
                        data: Array(60).fill(0),
                        borderColor: '#01a3a4',
                        backgroundColor: 'rgba(1, 163, 164, 0.1)',
                        fill: false
                    }
                ]
            },
            options: {
                ...commonConfig.options,
                scales: {
                    ...commonConfig.options.scales,
                    y: {
                        display: false,
                        min: 0
                        // max sera calculé dynamiquement
                    }
                }
            }
        });
    }
    
    updateCharts(metrics) {
        // CPU
        if (metrics.cpu && this.charts.cpu) {
            const data = this.charts.cpu.data.datasets[0].data;
            data.shift();
            data.push(metrics.cpu.total_percent || 0);
            this.charts.cpu.update('none');
        }
        
        // Mémoire
        if (metrics.memory && this.charts.memory) {
            const data = this.charts.memory.data.datasets[0].data;
            data.shift();
            data.push(metrics.memory.percent || 0);
            this.charts.memory.update('none');
        }
        
        // Disque
        if (metrics.disk && this.charts.disk) {
            let maxPercent = 0;
            if (metrics.disk.partitions) {
                Object.values(metrics.disk.partitions).forEach(partition => {
                    maxPercent = Math.max(maxPercent, partition.percent || 0);
                });
            }
            
            const data = this.charts.disk.data.datasets[0].data;
            data.shift();
            data.push(maxPercent);
            this.charts.disk.update('none');
        }
        
        // Réseau
        if (metrics.network && this.charts.network && this.lastNetworkUpdate) {
            // Calcul approximatif des vitesses pour le graphique
            const uploadData = this.charts.network.data.datasets[0].data;
            const downloadData = this.charts.network.data.datasets[1].data;
            
            // Simulation de données réseau (à améliorer avec de vraies métriques)
            const uploadSpeed = Math.random() * 10; // MB/s
            const downloadSpeed = Math.random() * 50; // MB/s
            
            uploadData.shift();
            uploadData.push(uploadSpeed);
            downloadData.shift();
            downloadData.push(downloadSpeed);
            
            this.charts.network.update('none');
        }
    }
    
    toggleDebugMode() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ command: 'toggle_debug' }));
            
            if (!this.debugMode) {
                this.showDebugPanel();
                this.requestDebugData();
            } else {
                this.closeDebugPanel();
            }
        }
    }
    
    showDebugPanel() {
        // Créer l'overlay
        const overlay = document.createElement('div');
        overlay.className = 'debug-overlay';
        overlay.onclick = () => this.closeDebugPanel();
        document.body.appendChild(overlay);
        
        // Afficher le panel
        const debugPanel = document.getElementById('debugPanel');
        debugPanel.style.display = 'block';
        
        // Animation d'entrée
        setTimeout(() => {
            debugPanel.style.opacity = '1';
            debugPanel.style.transform = 'translate(-50%, -50%) scale(1)';
        }, 10);
        
        this.debugMode = true;
        this.updateDebugButton();
    }
    
    closeDebugPanel() {
        const debugPanel = document.getElementById('debugPanel');
        const overlay = document.querySelector('.debug-overlay');
        
        debugPanel.style.display = 'none';
        if (overlay) {
            overlay.remove();
        }
        
        this.debugMode = false;
        this.updateDebugButton();
    }
    
    updateDebugButton() {
        const debugBtn = document.getElementById('debugToggle');
        if (this.debugMode) {
            debugBtn.classList.add('active');
            debugBtn.innerHTML = '<i class="fas fa-bug"></i> Debug ON';
        } else {
            debugBtn.classList.remove('active');
            debugBtn.innerHTML = '<i class="fas fa-bug"></i> Mode Debug';
        }
    }
    
    switchDebugTab(tabName) {
        // Mise à jour des onglets
        document.querySelectorAll('.debug-tab').forEach(tab => {
            tab.classList.remove('active');
        });
        document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
        
        // Mise à jour du contenu
        document.querySelectorAll('.debug-tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(`debug${tabName.charAt(0).toUpperCase() + tabName.slice(1)}`).classList.add('active');
        
        // Charger les données spécifiques à l'onglet
        this.loadDebugTabData(tabName);
    }
    
    loadDebugTabData(tabName) {
        switch (tabName) {
            case 'performance':
                this.initDebugPerformanceChart();
                break;
            case 'cache':
                this.loadCacheStats();
                break;
            case 'network':
                this.loadNetworkDebug();
                break;
            case 'security':
                this.loadSecurityEvents();
                break;
            case 'logs':
                // Les logs sont déjà en temps réel
                break;
        }
    }
    
    initDebugPerformanceChart() {
        const ctx = document.getElementById('debugPerformanceChart');
        if (!ctx || this.charts.debugPerformance) return;
        
        this.charts.debugPerformance = new Chart(ctx.getContext('2d'), {
            type: 'line',
            data: {
                labels: Array(30).fill(''),
                datasets: [
                    {
                        label: 'CPU',
                        data: Array(30).fill(0),
                        borderColor: '#ff6b6b',
                        backgroundColor: 'rgba(255, 107, 107, 0.1)',
                        fill: false
                    },
                    {
                        label: 'Memory',
                        data: Array(30).fill(0),
                        borderColor: '#4834d4',
                        backgroundColor: 'rgba(72, 52, 212, 0.1)',
                        fill: false
                    },
                    {
                        label: 'Disk I/O',
                        data: Array(30).fill(0),
                        borderColor: '#ff9ff3',
                        backgroundColor: 'rgba(255, 159, 243, 0.1)',
                        fill: false
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        display: true,
                        labels: {
                            color: '#ffffff'
                        }
                    }
                },
                scales: {
                    x: {
                        ticks: { color: '#888888' },
                        grid: { color: '#333333' }
                    },
                    y: {
                        ticks: { color: '#888888' },
                        grid: { color: '#333333' },
                        min: 0,
                        max: 100
                    }
                }
            }
        });
    }
    
    requestDebugData() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ command: 'get_debug' }));
        }
    }
    
    updateDebugInfo(debugData) {
        // Mise à jour des métriques de performance
        if (debugData.performance_metrics) {
            const metrics = debugData.performance_metrics;
            document.getElementById('cacheHitRate').textContent = `${(metrics.cache_hit_ratio * 100).toFixed(1)}%`;
            document.getElementById('avgResponseTime').textContent = `${(metrics.avg_response_time * 1000).toFixed(1)}ms`;
            document.getElementById('requestsPerSecond').textContent = `${metrics.requests_per_second}`;
            document.getElementById('memoryEfficiency').textContent = `${(metrics.memory_efficiency * 100).toFixed(1)}%`;
        }
        
        // Mise à jour des stats du cache
        if (debugData.cache_stats) {
            this.updateCacheStats(debugData.cache_stats);
        }
        
        // Mise à jour du debug réseau
        if (debugData.network_debug) {
            this.updateNetworkDebug(debugData.network_debug);
        }
        
        // Mise à jour des événements de sécurité
        if (debugData.security_events) {
            this.updateSecurityEvents(debugData.security_events);
        }
    }
    
    updateCacheStats(cacheStats) {
        const cacheStatsElement = document.getElementById('cacheStats');
        cacheStatsElement.innerHTML = `
            <div class="debug-metric">
                <span>Entrées en cache:</span>
                <span>${cacheStats.entries}</span>
            </div>
            <div class="debug-metric">
                <span>Taux de hit:</span>
                <span>${cacheStats.hit_rate}</span>
            </div>
            <div class="debug-metric">
                <span>Utilisation mémoire:</span>
                <span>${this.formatBytes(cacheStats.memory_usage)}</span>
            </div>
        `;
    }
    
    updateNetworkDebug(networkDebug) {
        const networkDebugElement = document.getElementById('networkDebug');
        networkDebugElement.innerHTML = `
            <div class="debug-metric">
                <span>Connexions actives:</span>
                <span>${networkDebug.active_connections}</span>
            </div>
            <div class="debug-metric">
                <span>Ports en écoute:</span>
                <span>${networkDebug.listening_ports ? networkDebug.listening_ports.join(', ') : 'N/A'}</span>
            </div>
        `;
    }
    
    updateSecurityEvents(securityEvents) {
        const securityEventsElement = document.getElementById('securityEvents');
        
        if (securityEvents.length === 0) {
            securityEventsElement.innerHTML = '<p>Aucun événement de sécurité récent</p>';
            return;
        }
        
        let eventsHtml = '';
        securityEvents.forEach(event => {
            eventsHtml += `
                <div class="security-event">
                    <div class="event-time">${new Date(event.timestamp * 1000).toLocaleString()}</div>
                    <div class="event-message">${event.message}</div>
                    <div class="event-type ${event.type}">${event.type}</div>
                </div>
            `;
        });
        
        securityEventsElement.innerHTML = eventsHtml;
    }
    
    addLog(message, type = 'info') {
        if (this.logsPaused) return;
        
        const logsOutput = document.getElementById('logsOutput');
        if (!logsOutput) return;
        
        const timestamp = new Date().toLocaleTimeString();
        const logEntry = document.createElement('div');
        logEntry.className = `log-entry ${type}`;
        logEntry.textContent = `[${timestamp}] ${message}`;
        
        logsOutput.appendChild(logEntry);
        
        // Auto-scroll
        logsOutput.scrollTop = logsOutput.scrollHeight;
        
        // Limiter le nombre de logs
        const logs = logsOutput.querySelectorAll('.log-entry');
        if (logs.length > 1000) {
            for (let i = 0; i < logs.length - 1000; i++) {
                logs[i].remove();
            }
        }
    }
    
    clearLogs() {
        const logsOutput = document.getElementById('logsOutput');
        if (logsOutput) {
            logsOutput.innerHTML = '';
        }
    }
    
    toggleLogsPause() {
        this.logsPaused = !this.logsPaused;
        const pauseBtn = document.getElementById('pauseLogs');
        pauseBtn.textContent = this.logsPaused ? 'Reprendre' : 'Pause';
        pauseBtn.classList.toggle('active', this.logsPaused);
    }
    
    refreshData() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.addLog('Actualisation des données...', 'info');
            // Le serveur envoie automatiquement les données
        } else {
            this.addLog('Impossible d\'actualiser: pas de connexion', 'error');
        }
    }
    
    toggleFullscreen() {
        if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen().catch(err => {
                this.addLog(`Erreur plein écran: ${err.message}`, 'error');
            });
        } else {
            document.exitFullscreen();
        }
    }
    
    updateConnectionStatus(connected) {
        const statusElement = document.getElementById('connectionStatus');
        const icon = statusElement.querySelector('i');
        const text = statusElement.querySelector('span');
        
        if (connected) {
            statusElement.className = 'connection-status connected';
            text.textContent = 'Connecté';
        } else {
            statusElement.className = 'connection-status disconnected';
            text.textContent = 'Déconnecté';
        }
    }
    
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
    
    startPerformanceMonitoring() {
        // Surveillance des performances du dashboard lui-même
        setInterval(() => {
            const memoryInfo = performance.memory;
            if (memoryInfo) {
                this.addLog(`Dashboard RAM: ${this.formatBytes(memoryInfo.usedJSHeapSize)}`, 'info');
            }
        }, 30000); // Toutes les 30 secondes
    }
}

// Initialisation du dashboard
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new UltraPerformanceDashboard();
});

// Gestion des erreurs globales
window.addEventListener('error', (event) => {
    if (window.dashboard) {
        window.dashboard.addLog(`Erreur JS: ${event.error.message}`, 'error');
    }
});

// Gestion de la fermeture de la page
window.addEventListener('beforeunload', () => {
    if (window.dashboard && window.dashboard.ws) {
        window.dashboard.ws.close();
    }
});

console.log('🔥 Dashboard Ultra-Performant chargé et prêt!');