/**
 * SYSTÈME DE SÉCURITÉ ANTI-BOT AVANCÉ
 * Conçu pour Cloudflare Workers
 * Bloque les bots avec >70% de certitude, laisse passer les humains
 */

class BotDetector {
  constructor() {
    // Seuil de détection (70% = 0.7)
    this.BOT_THRESHOLD = 0.7;
    
    // Signatures de User-Agents suspects
    this.SUSPICIOUS_UA_PATTERNS = [
      /bot|crawler|spider|scraper|wget|curl/i,
      /headless|phantom|selenium|puppeteer/i,
      /python|java|go-http|ruby|perl/i,
      /postman|insomnia|httpie/i,
      /scanner|exploit|hack|attack/i
    ];
    
    // User-Agents légitimes connus
    this.LEGITIMATE_UA_PATTERNS = [
      /mozilla.*firefox/i,
      /mozilla.*chrome/i,
      /mozilla.*safari/i,
      /mozilla.*edge/i,
      /opera/i
    ];
    
    // IPs suspectes (ranges de datacenters connus)
    this.SUSPICIOUS_IP_RANGES = [
      '185.220.', // Tor exit nodes
      '198.98.',  // DigitalOcean
      '167.99.',  // DigitalOcean
      '138.197.', // DigitalOcean
      '159.203.', // DigitalOcean
      '52.',      // AWS (partiel)
      '54.',      // AWS (partiel)
      '18.',      // AWS (partiel)
    ];
    
    // Headers requis pour les navigateurs légitimes
    this.REQUIRED_HEADERS = [
      'accept',
      'accept-language',
      'accept-encoding'
    ];
  }

  /**
   * Analyse principale pour détecter les bots
   * Retourne un score de 0 (humain) à 1 (bot certain)
   */
  async analyzeBotScore(request, cf) {
    let score = 0;
    const analysis = {
      userAgent: 0,
      headers: 0,
      behavior: 0,
      ip: 0,
      timing: 0,
      fingerprint: 0
    };

    // 1. Analyse du User-Agent (25% du score)
    analysis.userAgent = this.analyzeUserAgent(request.headers.get('user-agent'));
    
    // 2. Analyse des headers (20% du score)
    analysis.headers = this.analyzeHeaders(request.headers);
    
    // 3. Analyse comportementale (20% du score)
    analysis.behavior = await this.analyzeBehavior(request);
    
    // 4. Analyse de l'IP (15% du score)
    analysis.ip = this.analyzeIP(cf);
    
    // 5. Analyse du timing (10% du score)
    analysis.timing = this.analyzeTiming(request);
    
    // 6. Empreinte digitale (10% du score)
    analysis.fingerprint = this.analyzeFingerprint(request);

    // Calcul du score final pondéré
    score = (
      analysis.userAgent * 0.25 +
      analysis.headers * 0.20 +
      analysis.behavior * 0.20 +
      analysis.ip * 0.15 +
      analysis.timing * 0.10 +
      analysis.fingerprint * 0.10
    );

    return { score, analysis };
  }

  analyzeUserAgent(userAgent) {
    if (!userAgent) return 0.9; // Pas de UA = très suspect
    
    // Vérifier les patterns suspects
    for (const pattern of this.SUSPICIOUS_UA_PATTERNS) {
      if (pattern.test(userAgent)) {
        return 0.95; // Très suspect
      }
    }
    
    // Vérifier les patterns légitimes
    let hasLegitimate = false;
    for (const pattern of this.LEGITIMATE_UA_PATTERNS) {
      if (pattern.test(userAgent)) {
        hasLegitimate = true;
        break;
      }
    }
    
    if (!hasLegitimate) return 0.7; // Suspect
    
    // Vérifier la structure du UA
    if (userAgent.length < 50) return 0.6; // UA trop court
    if (!userAgent.includes('Mozilla')) return 0.8; // Pas de Mozilla
    
    return 0.1; // Probablement légitime
  }

  analyzeHeaders(headers) {
    let suspiciousScore = 0;
    const headerMap = {};
    
    // Convertir en map pour analyse
    for (const [key, value] of headers.entries()) {
      headerMap[key.toLowerCase()] = value;
    }
    
    // Vérifier les headers requis
    let missingRequired = 0;
    for (const required of this.REQUIRED_HEADERS) {
      if (!headerMap[required]) {
        missingRequired++;
      }
    }
    
    if (missingRequired > 0) {
      suspiciousScore += missingRequired * 0.3;
    }
    
    // Headers suspects
    if (headerMap['x-forwarded-for'] && headerMap['x-forwarded-for'].split(',').length > 3) {
      suspiciousScore += 0.3; // Trop de proxies
    }
    
    if (!headerMap['accept-language']) {
      suspiciousScore += 0.4; // Pas de langue
    }
    
    if (headerMap['connection'] === 'close') {
      suspiciousScore += 0.2; // Connection fermée
    }
    
    // Headers de bots connus
    const botHeaders = ['x-bot', 'x-crawler', 'x-spider'];
    for (const botHeader of botHeaders) {
      if (headerMap[botHeader]) {
        suspiciousScore += 0.8;
      }
    }
    
    return Math.min(suspiciousScore, 1);
  }

  async analyzeBehavior(request) {
    const url = new URL(request.url);
    let suspiciousScore = 0;
    
    // Patterns d'URL suspects
    const suspiciousPatterns = [
      /\/wp-admin/,
      /\/admin/,
      /\.php$/,
      /\/api\/.*\/.*\/.*/, // API trop profonde
      /\.(xml|json)$/,
      /\/robots\.txt/,
      /\/sitemap/
    ];
    
    for (const pattern of suspiciousPatterns) {
      if (pattern.test(url.pathname)) {
        suspiciousScore += 0.3;
      }
    }
    
    // Paramètres suspects
    const params = url.searchParams;
    if (params.has('debug') || params.has('test') || params.has('admin')) {
      suspiciousScore += 0.4;
    }
    
    // Méthodes HTTP suspectes pour certains endpoints
    if (request.method !== 'GET' && request.method !== 'POST') {
      suspiciousScore += 0.3;
    }
    
    return Math.min(suspiciousScore, 1);
  }

  analyzeIP(cf) {
    if (!cf || !cf.ip) return 0.5;
    
    const ip = cf.ip;
    let suspiciousScore = 0;
    
    // Vérifier les ranges suspects
    for (const range of this.SUSPICIOUS_IP_RANGES) {
      if (ip.startsWith(range)) {
        suspiciousScore += 0.6;
        break;
      }
    }
    
    // Analyse du pays (optionnel)
    if (cf.country) {
      const suspiciousCountries = ['CN', 'RU', 'KP']; // Exemple
      if (suspiciousCountries.includes(cf.country)) {
        suspiciousScore += 0.2;
      }
    }
    
    // Vérifier si c'est un datacenter
    if (cf.asOrganization && 
        (cf.asOrganization.toLowerCase().includes('hosting') ||
         cf.asOrganization.toLowerCase().includes('cloud') ||
         cf.asOrganization.toLowerCase().includes('server'))) {
      suspiciousScore += 0.4;
    }
    
    return Math.min(suspiciousScore, 1);
  }

  analyzeTiming(request) {
    // Cette analyse serait plus complète avec un stockage persistant
    // Pour l'instant, analyse basique basée sur les headers
    
    let suspiciousScore = 0;
    const now = Date.now();
    
    // Si pas de cache headers, c'est suspect pour un navigateur
    const cacheControl = request.headers.get('cache-control');
    if (!cacheControl) {
      suspiciousScore += 0.3;
    }
    
    return Math.min(suspiciousScore, 1);
  }

  analyzeFingerprint(request) {
    let suspiciousScore = 0;
    
    // Analyse de la cohérence des headers
    const userAgent = request.headers.get('user-agent');
    const acceptLanguage = request.headers.get('accept-language');
    const acceptEncoding = request.headers.get('accept-encoding');
    
    if (userAgent && userAgent.includes('Chrome')) {
      if (!acceptEncoding || !acceptEncoding.includes('gzip')) {
        suspiciousScore += 0.4; // Chrome supporte toujours gzip
      }
    }
    
    if (userAgent && userAgent.includes('Firefox')) {
      if (!acceptLanguage || acceptLanguage === 'en-US') {
        suspiciousScore += 0.2; // Firefox a généralement des langues multiples
      }
    }
    
    return Math.min(suspiciousScore, 1);
  }

  /**
   * Génère une page de challenge pour vérification humaine
   */
  generateChallengeResponse(botScore, analysis) {
    const challengeHtml = `
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>Vérification de sécurité</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                margin: 0;
                padding: 20px;
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            .container {
                background: white;
                border-radius: 10px;
                padding: 40px;
                box-shadow: 0 10px 30px rgba(0,0,0,0.3);
                max-width: 500px;
                width: 100%;
                text-align: center;
            }
            .shield {
                font-size: 64px;
                margin-bottom: 20px;
            }
            h1 {
                color: #333;
                margin-bottom: 10px;
            }
            .score {
                background: #f8f9fa;
                border-radius: 5px;
                padding: 15px;
                margin: 20px 0;
                font-family: monospace;
            }
            .challenge {
                margin: 30px 0;
                padding: 20px;
                background: #e3f2fd;
                border-radius: 5px;
            }
            button {
                background: #4CAF50;
                color: white;
                border: none;
                padding: 15px 30px;
                border-radius: 5px;
                cursor: pointer;
                font-size: 16px;
                transition: all 0.3s;
            }
            button:hover {
                background: #45a049;
                transform: translateY(-2px);
            }
            .math-challenge {
                font-size: 18px;
                margin: 15px 0;
            }
            input {
                padding: 10px;
                border: 2px solid #ddd;
                border-radius: 5px;
                font-size: 16px;
                margin: 0 10px;
                width: 80px;
                text-align: center;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="shield">🛡️</div>
            <h1>Vérification de sécurité</h1>
            <p>Notre système a détecté une activité suspecte. Veuillez confirmer que vous êtes humain.</p>
            
            <div class="score">
                Score de détection: ${Math.round(botScore * 100)}%
            </div>
            
            <div class="challenge">
                <div class="math-challenge">
                    Résolvez cette équation simple:
                    <br><br>
                    <span id="num1"></span> + <span id="num2"></span> = 
                    <input type="number" id="answer" maxlength="3">
                </div>
                <button onclick="verifyHuman()">Vérifier</button>
            </div>
            
            <small>Cette vérification protège le site contre les attaques automatisées.</small>
        </div>

        <script>
            // Génération du challenge mathématique
            const num1 = Math.floor(Math.random() * 20) + 1;
            const num2 = Math.floor(Math.random() * 20) + 1;
            const correctAnswer = num1 + num2;
            
            document.getElementById('num1').textContent = num1;
            document.getElementById('num2').textContent = num2;
            
            function verifyHuman() {
                const userAnswer = parseInt(document.getElementById('answer').value);
                
                if (userAnswer === correctAnswer) {
                    // Créer un cookie de vérification
                    const token = btoa(Date.now() + ':verified:' + Math.random());
                    document.cookie = 'human_verified=' + token + '; path=/; max-age=3600; secure; samesite=strict';
                    
                    // Rediriger vers la page originale
                    window.location.href = window.location.href.replace('?challenge=1', '');
                } else {
                    alert('Réponse incorrecte. Veuillez réessayer.');
                    document.getElementById('answer').value = '';
                    document.getElementById('answer').focus();
                }
            }
            
            // Focus automatique sur le champ de réponse
            document.getElementById('answer').focus();
            
            // Validation au clavier
            document.getElementById('answer').addEventListener('keypress', function(e) {
                if (e.key === 'Enter') {
                    verifyHuman();
                }
            });
        </script>
    </body>
    </html>`;
    
    return new Response(challengeHtml, {
      headers: {
        'Content-Type': 'text/html; charset=UTF-8',
        'Cache-Control': 'no-cache, no-store, must-revalidate',
        'X-Bot-Score': botScore.toString()
      }
    });
  }

  /**
   * Vérifie si l'utilisateur a déjà passé la vérification
   */
  isHumanVerified(request) {
    const cookies = request.headers.get('cookie');
    if (!cookies) return false;
    
    const humanCookie = cookies.split(';')
      .find(c => c.trim().startsWith('human_verified='));
    
    if (!humanCookie) return false;
    
    try {
      const token = humanCookie.split('=')[1];
      const decoded = atob(token);
      const [timestamp, status] = decoded.split(':');
      
      // Vérifier que le token n'est pas trop ancien (1 heure)
      const tokenTime = parseInt(timestamp);
      const now = Date.now();
      const oneHour = 60 * 60 * 1000;
      
      return status === 'verified' && (now - tokenTime) < oneHour;
    } catch (e) {
      return false;
    }
  }
}

/**
 * POINT D'ENTRÉE PRINCIPAL DU WORKER
 */
export default {
  async fetch(request, env, ctx) {
    try {
      const detector = new BotDetector();
      const url = new URL(request.url);
      
      // Vérifier si c'est une demande de challenge
      const isChallenge = url.searchParams.has('challenge');
      
      // Si l'utilisateur est déjà vérifié, laisser passer
      if (detector.isHumanVerified(request)) {
        return await handleLegitimateRequest(request, env);
      }
      
      // Analyser le score de bot
      const { score, analysis } = await detector.analyzeBotScore(request, request.cf);
      
      // Log pour debugging (à retirer en production)
      console.log(`Bot Score: ${Math.round(score * 100)}% for ${request.cf?.ip || 'unknown'}`);
      
      // Si le score dépasse le seuil, bloquer ou challenger
      if (score >= detector.BOT_THRESHOLD) {
        // Pour les scores très élevés (>90%), bloquer directement
        if (score >= 0.9) {
          return new Response('Access Denied - Automated Access Detected', {
            status: 403,
            headers: {
              'Content-Type': 'text/plain',
              'X-Bot-Score': score.toString(),
              'X-Block-Reason': 'High bot probability'
            }
          });
        }
        
        // Pour les scores modérés, proposer un challenge
        return detector.generateChallengeResponse(score, analysis);
      }
      
      // Score acceptable, laisser passer
      return await handleLegitimateRequest(request, env);
      
    } catch (error) {
      console.error('Security Worker Error:', error);
      // En cas d'erreur, laisser passer pour éviter de bloquer les vrais utilisateurs
      return await handleLegitimateRequest(request, env);
    }
  }
};

/**
 * Gère les requêtes légitimes (proxie vers votre vrai site)
 */
async function handleLegitimateRequest(request, env) {
  return Response.redirect('http://airwaymast.org/', 302);
}