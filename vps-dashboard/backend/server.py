#!/usr/bin/env python3
"""
🔥 TABLEAU DE BORD VPS ULTRA-PERFORMANT 🔥
Backend WebSocket avec monitoring temps réel et mode debug avancé
"""

import asyncio
import json
import time
import psutil
import socket
import subprocess
import threading
from datetime import datetime, timedelta
from collections import deque, defaultdict
import websockets
import logging
from pathlib import Path
import os
import GPUtil
import netifaces
import sqlite3
from typing import Dict, List, Any
import aiofiles
import hashlib
import platform

# Configuration du logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class UltraPerformanceMonitor:
    """Moniteur ultra-performant avec cache optimisé et collecte asynchrone"""
    
    def __init__(self):
        self.debug_mode = True
        self.performance_cache = {}
        self.cache_ttl = 1.0  # 1 seconde de cache
        self.metrics_history = defaultdict(lambda: deque(maxlen=1000))
        self.alert_thresholds = {
            'cpu': 85.0,
            'memory': 90.0,
            'disk': 95.0,
            'network_errors': 10
        }
        self.connected_clients = set()
        self.db_path = "/workspace/vps-dashboard/metrics.db"
        self.init_database()
        
        # Métriques avancées
        self.process_monitor = ProcessMonitor()
        self.network_monitor = NetworkMonitor()
        self.security_monitor = SecurityMonitor()
        
    def init_database(self):
        """Initialise la base de données SQLite pour l'historique"""
        conn = sqlite3.connect(self.db_path)
        conn.execute('''
            CREATE TABLE IF NOT EXISTS metrics (
                timestamp INTEGER PRIMARY KEY,
                cpu_percent REAL,
                memory_percent REAL,
                disk_percent REAL,
                network_bytes_sent INTEGER,
                network_bytes_recv INTEGER,
                load_avg TEXT,
                processes_count INTEGER
            )
        ''')
        conn.commit()
        conn.close()
        
    async def get_cached_or_compute(self, key: str, compute_func, ttl: float = None):
        """Cache intelligent avec TTL"""
        ttl = ttl or self.cache_ttl
        current_time = time.time()
        
        if key in self.performance_cache:
            cached_data, timestamp = self.performance_cache[key]
            if current_time - timestamp < ttl:
                return cached_data
                
        # Calcul asynchrone
        data = await compute_func()
        self.performance_cache[key] = (data, current_time)
        return data
        
    async def get_system_metrics(self) -> Dict[str, Any]:
        """Collecte ultra-rapide des métriques système"""
        
        async def compute_metrics():
            # Collecte parallèle des métriques
            tasks = [
                self._get_cpu_info(),
                self._get_memory_info(),
                self._get_disk_info(),
                self._get_network_info(),
                self._get_system_info(),
                self._get_process_info(),
                self._get_gpu_info(),
            ]
            
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            metrics = {
                'timestamp': time.time(),
                'cpu': results[0] if not isinstance(results[0], Exception) else {},
                'memory': results[1] if not isinstance(results[1], Exception) else {},
                'disk': results[2] if not isinstance(results[2], Exception) else {},
                'network': results[3] if not isinstance(results[3], Exception) else {},
                'system': results[4] if not isinstance(results[4], Exception) else {},
                'processes': results[5] if not isinstance(results[5], Exception) else {},
                'gpu': results[6] if not isinstance(results[6], Exception) else {},
            }
            
            # Ajout des alertes
            metrics['alerts'] = self._generate_alerts(metrics)
            
            # Sauvegarde en base
            await self._save_metrics(metrics)
            
            return metrics
            
        return await self.get_cached_or_compute('system_metrics', compute_metrics)
        
    async def _get_cpu_info(self) -> Dict[str, Any]:
        """Informations CPU ultra-détaillées"""
        try:
            cpu_percent = psutil.cpu_percent(interval=0.1, percpu=True)
            cpu_freq = psutil.cpu_freq(percpu=True) if hasattr(psutil, 'cpu_freq') else []
            load_avg = os.getloadavg()
            cpu_count = psutil.cpu_count()
            cpu_count_logical = psutil.cpu_count(logical=True)
            
            # CPU par cœur
            cpu_cores = []
            for i, (percent, freq) in enumerate(zip(cpu_percent, cpu_freq if cpu_freq else [None] * len(cpu_percent))):
                core_info = {
                    'core': i,
                    'percent': round(percent, 1),
                    'frequency': round(freq.current, 1) if freq else None
                }
                cpu_cores.append(core_info)
            
            return {
                'total_percent': round(sum(cpu_percent) / len(cpu_percent), 1),
                'cores': cpu_cores,
                'load_avg': [round(x, 2) for x in load_avg],
                'count_physical': cpu_count,
                'count_logical': cpu_count_logical,
                'context_switches': psutil.cpu_stats().ctx_switches,
                'interrupts': psutil.cpu_stats().interrupts,
            }
        except Exception as e:
            logger.error(f"Erreur CPU: {e}")
            return {}
            
    async def _get_memory_info(self) -> Dict[str, Any]:
        """Informations mémoire détaillées"""
        try:
            memory = psutil.virtual_memory()
            swap = psutil.swap_memory()
            
            return {
                'total': memory.total,
                'available': memory.available,
                'used': memory.used,
                'free': memory.free,
                'percent': round(memory.percent, 1),
                'buffers': memory.buffers,
                'cached': memory.cached,
                'swap': {
                    'total': swap.total,
                    'used': swap.used,
                    'free': swap.free,
                    'percent': round(swap.percent, 1) if swap.total > 0 else 0
                },
                'memory_map': self._get_memory_map()
            }
        except Exception as e:
            logger.error(f"Erreur mémoire: {e}")
            return {}
            
    def _get_memory_map(self) -> List[Dict[str, Any]]:
        """Carte mémoire des processus les plus gourmands"""
        try:
            processes = []
            for proc in psutil.process_iter(['pid', 'name', 'memory_info', 'cpu_percent']):
                try:
                    processes.append({
                        'pid': proc.info['pid'],
                        'name': proc.info['name'],
                        'memory_mb': round(proc.info['memory_info'].rss / 1024 / 1024, 1),
                        'cpu_percent': round(proc.info['cpu_percent'] or 0, 1)
                    })
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
                    
            return sorted(processes, key=lambda x: x['memory_mb'], reverse=True)[:10]
        except Exception:
            return []
            
    async def _get_disk_info(self) -> Dict[str, Any]:
        """Informations disque avec I/O"""
        try:
            disk_usage = {}
            disk_io = psutil.disk_io_counters(perdisk=True)
            
            for partition in psutil.disk_partitions():
                try:
                    usage = psutil.disk_usage(partition.mountpoint)
                    disk_usage[partition.device] = {
                        'mountpoint': partition.mountpoint,
                        'fstype': partition.fstype,
                        'total': usage.total,
                        'used': usage.used,
                        'free': usage.free,
                        'percent': round(usage.used / usage.total * 100, 1) if usage.total > 0 else 0
                    }
                except PermissionError:
                    continue
                    
            # I/O Statistics
            total_io = psutil.disk_io_counters()
            io_stats = {
                'read_bytes': total_io.read_bytes,
                'write_bytes': total_io.write_bytes,
                'read_count': total_io.read_count,
                'write_count': total_io.write_count,
                'read_time': total_io.read_time,
                'write_time': total_io.write_time
            } if total_io else {}
            
            return {
                'partitions': disk_usage,
                'io_stats': io_stats,
                'io_per_disk': {k: {
                    'read_bytes': v.read_bytes,
                    'write_bytes': v.write_bytes,
                    'read_count': v.read_count,
                    'write_count': v.write_count
                } for k, v in disk_io.items()} if disk_io else {}
            }
        except Exception as e:
            logger.error(f"Erreur disque: {e}")
            return {}
            
    async def _get_network_info(self) -> Dict[str, Any]:
        """Informations réseau avancées"""
        try:
            net_io = psutil.net_io_counters(pernic=True)
            net_connections = len(psutil.net_connections())
            
            interfaces = {}
            for interface, stats in net_io.items():
                interfaces[interface] = {
                    'bytes_sent': stats.bytes_sent,
                    'bytes_recv': stats.bytes_recv,
                    'packets_sent': stats.packets_sent,
                    'packets_recv': stats.packets_recv,
                    'errin': stats.errin,
                    'errout': stats.errout,
                    'dropin': stats.dropin,
                    'dropout': stats.dropout
                }
                
            # Adresses IP
            ip_addresses = {}
            try:
                for interface in netifaces.interfaces():
                    addrs = netifaces.ifaddresses(interface)
                    if netifaces.AF_INET in addrs:
                        ip_addresses[interface] = [addr['addr'] for addr in addrs[netifaces.AF_INET]]
            except Exception:
                pass
                
            return {
                'interfaces': interfaces,
                'connections_count': net_connections,
                'ip_addresses': ip_addresses,
                'public_ip': await self._get_public_ip()
            }
        except Exception as e:
            logger.error(f"Erreur réseau: {e}")
            return {}
            
    async def _get_public_ip(self) -> str:
        """Obtient l'IP publique"""
        try:
            import aiohttp
            async with aiohttp.ClientSession() as session:
                async with session.get('https://api.ipify.org', timeout=5) as response:
                    return await response.text()
        except Exception:
            return "N/A"
            
    async def _get_system_info(self) -> Dict[str, Any]:
        """Informations système"""
        try:
            boot_time = psutil.boot_time()
            uptime = time.time() - boot_time
            
            return {
                'hostname': socket.gethostname(),
                'platform': platform.platform(),
                'architecture': platform.architecture()[0],
                'processor': platform.processor(),
                'boot_time': boot_time,
                'uptime_seconds': uptime,
                'uptime_string': str(timedelta(seconds=int(uptime))),
                'users': [{'name': user.name, 'terminal': user.terminal, 'started': user.started} 
                         for user in psutil.users()],
                'python_version': platform.python_version(),
                'kernel_version': platform.release()
            }
        except Exception as e:
            logger.error(f"Erreur système: {e}")
            return {}
            
    async def _get_process_info(self) -> Dict[str, Any]:
        """Informations processus"""
        try:
            processes = []
            total_processes = 0
            
            for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_info', 'status', 'create_time']):
                try:
                    total_processes += 1
                    if len(processes) < 20:  # Top 20 processus
                        processes.append({
                            'pid': proc.info['pid'],
                            'name': proc.info['name'],
                            'cpu_percent': round(proc.info['cpu_percent'] or 0, 1),
                            'memory_mb': round(proc.info['memory_info'].rss / 1024 / 1024, 1),
                            'status': proc.info['status'],
                            'started': proc.info['create_time']
                        })
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
                    
            processes.sort(key=lambda x: x['cpu_percent'], reverse=True)
            
            return {
                'total_count': total_processes,
                'top_processes': processes,
                'running': len([p for p in processes if p['status'] == 'running']),
                'sleeping': len([p for p in processes if p['status'] == 'sleeping'])
            }
        except Exception as e:
            logger.error(f"Erreur processus: {e}")
            return {}
            
    async def _get_gpu_info(self) -> Dict[str, Any]:
        """Informations GPU si disponible"""
        try:
            gpus = GPUtil.getGPUs()
            gpu_info = []
            
            for gpu in gpus:
                gpu_info.append({
                    'id': gpu.id,
                    'name': gpu.name,
                    'load': round(gpu.load * 100, 1),
                    'memory_used': gpu.memoryUsed,
                    'memory_total': gpu.memoryTotal,
                    'memory_percent': round((gpu.memoryUsed / gpu.memoryTotal) * 100, 1),
                    'temperature': gpu.temperature
                })
                
            return {'gpus': gpu_info}
        except Exception:
            return {'gpus': []}
            
    def _generate_alerts(self, metrics: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Génère des alertes intelligentes"""
        alerts = []
        
        # Alerte CPU
        if metrics.get('cpu', {}).get('total_percent', 0) > self.alert_thresholds['cpu']:
            alerts.append({
                'type': 'warning',
                'category': 'cpu',
                'message': f"CPU élevé: {metrics['cpu']['total_percent']}%",
                'timestamp': time.time()
            })
            
        # Alerte mémoire
        if metrics.get('memory', {}).get('percent', 0) > self.alert_thresholds['memory']:
            alerts.append({
                'type': 'critical',
                'category': 'memory',
                'message': f"Mémoire critique: {metrics['memory']['percent']}%",
                'timestamp': time.time()
            })
            
        # Alerte disque
        for device, info in metrics.get('disk', {}).get('partitions', {}).items():
            if info.get('percent', 0) > self.alert_thresholds['disk']:
                alerts.append({
                    'type': 'critical',
                    'category': 'disk',
                    'message': f"Disque {device} plein: {info['percent']}%",
                    'timestamp': time.time()
                })
                
        return alerts
        
    async def _save_metrics(self, metrics: Dict[str, Any]):
        """Sauvegarde les métriques en base"""
        try:
            conn = sqlite3.connect(self.db_path)
            conn.execute('''
                INSERT INTO metrics (
                    timestamp, cpu_percent, memory_percent, disk_percent,
                    network_bytes_sent, network_bytes_recv, load_avg, processes_count
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                int(metrics['timestamp']),
                metrics.get('cpu', {}).get('total_percent', 0),
                metrics.get('memory', {}).get('percent', 0),
                max([info.get('percent', 0) for info in metrics.get('disk', {}).get('partitions', {}).values()], default=0),
                sum([info.get('bytes_sent', 0) for info in metrics.get('network', {}).get('interfaces', {}).values()]),
                sum([info.get('bytes_recv', 0) for info in metrics.get('network', {}).get('interfaces', {}).values()]),
                json.dumps(metrics.get('cpu', {}).get('load_avg', [])),
                metrics.get('processes', {}).get('total_count', 0)
            ))
            conn.commit()
            conn.close()
        except Exception as e:
            logger.error(f"Erreur sauvegarde: {e}")
            
    async def get_debug_info(self) -> Dict[str, Any]:
        """Mode debug ultra-avancé"""
        debug_info = {
            'timestamp': time.time(),
            'cache_stats': {
                'entries': len(self.performance_cache),
                'hit_rate': 'N/A',  # TODO: implémenter
                'memory_usage': len(str(self.performance_cache))
            },
            'server_stats': {
                'connected_clients': len(self.connected_clients),
                'uptime': time.time() - getattr(self, 'start_time', time.time()),
                'python_memory': psutil.Process().memory_info().rss
            },
            'system_calls': await self._get_system_calls_stats(),
            'network_debug': await self.network_monitor.get_debug_info(),
            'security_events': await self.security_monitor.get_recent_events(),
            'performance_metrics': await self._get_performance_metrics()
        }
        
        return debug_info
        
    async def _get_system_calls_stats(self) -> Dict[str, Any]:
        """Statistiques des appels système"""
        try:
            with open('/proc/stat', 'r') as f:
                stats = f.read()
            
            return {
                'context_switches': int([line for line in stats.split('\n') if line.startswith('ctxt')][0].split()[1]),
                'processes_created': int([line for line in stats.split('\n') if line.startswith('processes')][0].split()[1]),
                'boot_time': int([line for line in stats.split('\n') if line.startswith('btime')][0].split()[1])
            }
        except Exception:
            return {}
            
    async def _get_performance_metrics(self) -> Dict[str, Any]:
        """Métriques de performance du serveur"""
        return {
            'cache_hit_ratio': 0.95,  # TODO: calculer réellement
            'avg_response_time': 0.001,  # TODO: mesurer
            'requests_per_second': 100,  # TODO: compter
            'memory_efficiency': 0.85,  # TODO: calculer
        }

class ProcessMonitor:
    """Moniteur de processus avancé"""
    
    async def get_process_tree(self) -> Dict[str, Any]:
        """Arbre des processus"""
        # TODO: Implémenter l'arbre des processus
        return {}

class NetworkMonitor:
    """Moniteur réseau avancé"""
    
    async def get_debug_info(self) -> Dict[str, Any]:
        """Informations de debug réseau"""
        return {
            'active_connections': len(psutil.net_connections()),
            'listening_ports': [conn.laddr.port for conn in psutil.net_connections() if conn.status == 'LISTEN']
        }

class SecurityMonitor:
    """Moniteur de sécurité"""
    
    async def get_recent_events(self) -> List[Dict[str, Any]]:
        """Événements de sécurité récents"""
        # TODO: Implémenter la surveillance sécurité
        return []

class WebSocketServer:
    """Serveur WebSocket ultra-performant"""
    
    def __init__(self, monitor: UltraPerformanceMonitor):
        self.monitor = monitor
        self.monitor.start_time = time.time()
        
    async def handle_client(self, websocket, path):
        """Gère les connexions client"""
        self.monitor.connected_clients.add(websocket)
        logger.info(f"Nouveau client connecté. Total: {len(self.monitor.connected_clients)}")
        
        try:
            # Envoi des données initiales
            initial_data = await self.monitor.get_system_metrics()
            await websocket.send(json.dumps({
                'type': 'initial_data',
                'data': initial_data
            }))
            
            # Boucle d'écoute des commandes client
            async for message in websocket:
                try:
                    data = json.loads(message)
                    await self.handle_command(websocket, data)
                except json.JSONDecodeError:
                    await websocket.send(json.dumps({
                        'type': 'error',
                        'message': 'Format JSON invalide'
                    }))
        except websockets.exceptions.ConnectionClosed:
            pass
        finally:
            self.monitor.connected_clients.discard(websocket)
            logger.info(f"Client déconnecté. Total: {len(self.monitor.connected_clients)}")
            
    async def handle_command(self, websocket, data):
        """Gère les commandes du client"""
        command = data.get('command')
        
        if command == 'get_debug':
            debug_info = await self.monitor.get_debug_info()
            await websocket.send(json.dumps({
                'type': 'debug_data',
                'data': debug_info
            }))
        elif command == 'toggle_debug':
            self.monitor.debug_mode = not self.monitor.debug_mode
            await websocket.send(json.dumps({
                'type': 'debug_toggled',
                'enabled': self.monitor.debug_mode
            }))
        elif command == 'get_history':
            # TODO: Récupérer l'historique depuis la base
            await websocket.send(json.dumps({
                'type': 'history_data',
                'data': []
            }))
            
    async def broadcast_updates(self):
        """Diffuse les mises à jour à tous les clients"""
        while True:
            if self.monitor.connected_clients:
                try:
                    metrics = await self.monitor.get_system_metrics()
                    message = json.dumps({
                        'type': 'metrics_update',
                        'data': metrics
                    })
                    
                    # Envoi concurrent à tous les clients
                    await asyncio.gather(*[
                        client.send(message) for client in self.monitor.connected_clients.copy()
                    ], return_exceptions=True)
                    
                except Exception as e:
                    logger.error(f"Erreur broadcast: {e}")
                    
            await asyncio.sleep(1)  # Mise à jour chaque seconde

async def main():
    """Fonction principale"""
    monitor = UltraPerformanceMonitor()
    server = WebSocketServer(monitor)
    
    # Installation des dépendances si nécessaire
    try:
        import aiohttp
    except ImportError:
        logger.info("Installation d'aiohttp...")
        subprocess.run(['pip', 'install', 'aiohttp'], check=True)
    
    logger.info("🚀 Démarrage du serveur de monitoring ultra-performant...")
    logger.info("📊 Mode debug activé")
    logger.info("🔥 Serveur WebSocket sur ws://localhost:8765")
    
    # Démarrage du serveur WebSocket et de la diffusion
    start_server = websockets.serve(server.handle_client, "0.0.0.0", 8765)
    
    await asyncio.gather(
        start_server,
        server.broadcast_updates()
    )

if __name__ == "__main__":
    asyncio.run(main())