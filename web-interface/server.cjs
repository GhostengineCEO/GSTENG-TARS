#!/usr/bin/env node

const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');

class TARSWebServer {
    constructor(port = 3000) {
        this.port = port;
        this.mimeTypes = {
            '.html': 'text/html',
            '.css': 'text/css',
            '.js': 'application/javascript',
            '.json': 'application/json',
            '.png': 'image/png',
            '.jpg': 'image/jpeg',
            '.gif': 'image/gif',
            '.ico': 'image/x-icon'
        };
        
        this.server = http.createServer((req, res) => this.handleRequest(req, res));
    }
    
    start() {
        this.server.listen(this.port, '0.0.0.0', () => {
            console.log(`
🤖 TARS INTERFACE SERVER ONLINE
=====================================

🌐 Access TARS Interface:
   • Local:    http://localhost:${this.port}
   • Network:  http://0.0.0.0:${this.port}
   • LAN:      http://[YOUR_IP]:${this.port}

🎯 Features Available:
   • Interstellar-style TARS chat interface
   • Personality controls (Humor, Honesty, Sarcasm)
   • Code review simulation
   • Engineering consultation
   • Model management controls

⚡ Status: OPERATIONAL
🎭 Personality: 75% Humor, 90% Honesty, 30% Sarcasm
🧠 Mission Focus: 100%

That's what I would have said. Eventually.
            `);
        });
        
        this.server.on('error', (err) => {
            if (err.code === 'EADDRINUSE') {
                console.error(`❌ Port ${this.port} is already in use. Trying port ${this.port + 1}...`);
                this.port += 1;
                this.start();
            } else {
                console.error('❌ Server error:', err);
            }
        });
    }
    
    handleRequest(req, res) {
        const parsedUrl = url.parse(req.url, true);
        const pathname = parsedUrl.pathname;
        
        // Handle API routes
        if (pathname.startsWith('/api/')) {
            this.handleAPI(req, res, parsedUrl);
            return;
        }
        
        // Handle static files
        let filePath = pathname === '/' ? '/index.html' : pathname;
        filePath = path.join(__dirname, filePath);
        
        // Security check - prevent directory traversal
        const normalizedPath = path.normalize(filePath);
        if (!normalizedPath.startsWith(__dirname)) {
            this.sendError(res, 403, 'Forbidden');
            return;
        }
        
        this.serveFile(filePath, res);
    }
    
    async handleAPI(req, res, parsedUrl) {
        const pathname = parsedUrl.pathname;
        const method = req.method;
        
        // Set CORS headers
        res.setHeader('Access-Control-Allow-Origin', '*');
        res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
        res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');
        
        if (method === 'OPTIONS') {
            res.writeHead(200);
            res.end();
            return;
        }
        
        try {
            let responseData = {};
            
            switch (pathname) {
                case '/api/tars/chat':
                    responseData = await this.handleChat(req);
                    break;
                    
                case '/api/tars/personality':
                    responseData = await this.handlePersonality(req, method);
                    break;
                    
                case '/api/tars/status':
                    responseData = await this.handleStatus();
                    break;
                    
                case '/api/tars/models':
                    responseData = await this.handleModels(req, method);
                    break;
                    
                case '/api/tars/emergency-stop':
                    responseData = await this.handleEmergencyStop();
                    break;
                    
                default:
                    this.sendError(res, 404, 'API endpoint not found');
                    return;
            }
            
            res.setHeader('Content-Type', 'application/json');
            res.writeHead(200);
            res.end(JSON.stringify(responseData));
            
        } catch (error) {
            console.error('API Error:', error);
            this.sendError(res, 500, 'Internal Server Error', error.message);
        }
    }
    
    async handleChat(req) {
        const body = await this.getRequestBody(req);
        const { message, context = 'general' } = JSON.parse(body);
        
        // Simulate processing delay
        await this.delay(1000 + Math.random() * 2000);
        
        // In a real implementation, this would call the Tauri backend
        // For now, simulate TARS responses
        const response = this.simulateTARSResponse(message, context);
        
        return {
            success: true,
            response: response,
            timestamp: new Date().toISOString(),
            context: context
        };
    }
    
    async handlePersonality(req, method) {
        if (method === 'GET') {
            return {
                success: true,
                personality: {
                    humor: 75,
                    honesty: 90,
                    sarcasm: 30,
                    mission_focus: 100
                }
            };
        }
        
        if (method === 'POST') {
            const body = await this.getRequestBody(req);
            const { humor, honesty, sarcasm } = JSON.parse(body);
            
            await this.delay(800);
            
            return {
                success: true,
                message: `[PERSONALITY UPDATE COMPLETE]
Humor: ${humor}%
Honesty: ${honesty}%
Sarcasm: ${sarcasm}%
Mission Focus: 100%

Personality matrix recalibrated. Neural pathways adjusted.

That's what I would have said. Eventually.`,
                personality: { humor, honesty, sarcasm, mission_focus: 100 }
            };
        }
    }
    
    async handleStatus() {
        await this.delay(500);
        
        return {
            success: true,
            status: 'operational',
            system: {
                online: true,
                model: 'codellama:7b-instruct',
                cognitive_systems: 'active',
                engineering_protocols: 'enabled',
                code_review_engine: 'ready',
                architecture_analysis: 'standby'
            },
            personality: {
                humor: 75,
                honesty: 90,
                sarcasm: 30,
                mission_focus: 100
            },
            timestamp: new Date().toISOString()
        };
    }
    
    async handleModels(req, method) {
        if (method === 'GET') {
            return {
                success: true,
                models: [
                    { name: 'codellama:7b-instruct', status: 'active', description: 'Code generation and analysis' },
                    { name: 'tars-engineering', status: 'available', description: 'TARS Engineering Model' },
                    { name: 'phi3:mini', status: 'available', description: 'Lightweight reasoning model' },
                    { name: 'gemma:2b', status: 'available', description: 'Fast response model' }
                ],
                current: 'codellama:7b-instruct'
            };
        }
        
        if (method === 'POST') {
            const body = await this.getRequestBody(req);
            const { model } = JSON.parse(body);
            
            await this.delay(2000 + Math.random() * 3000);
            
            return {
                success: true,
                message: `[MODEL SWITCH COMPLETE]

Neural pathways reconfigured.
Cognitive matrix updated.
Engineering protocols reloaded.

Model switch successful. All systems recalibrated.

Ready to resume engineering consultation with enhanced capabilities.`,
                current_model: model
            };
        }
    }
    
    async handleEmergencyStop() {
        await this.delay(500);
        
        return {
            success: true,
            message: `[EMERGENCY PROTOCOL ACTIVATED]

All non-essential systems shutting down...
Engineering analysis suspended.
Entering safe mode.

Emergency stop engaged. System status: SAFE HOLD.

Standing by for manual restart authorization.

Sarcasm setting temporarily reduced to 0% for safety.`,
            status: 'emergency_stop'
        };
    }
    
    simulateTARSResponse(message, context) {
        const messageLower = message.toLowerCase();
        
        // Simple keyword-based response generation
        if (context === 'code-review' || messageLower.includes('code') || messageLower.includes('function')) {
            return `Code Review Analysis Complete.

Based on my analysis, I can see several areas for improvement. Let me be direct - this needs work.

[ENGINEERING ASSESSMENT: 72/100]

Key Issues:
• Consider implementing proper error handling
• Variable naming could be more descriptive  
• Separation of concerns needs attention

That's what I would have said. Eventually.

[MISSION PRIORITY] Clean code is maintainable code. Address these issues before deployment.`;
        }
        
        if (messageLower.includes('hello') || messageLower.includes('hi')) {
            return `Greetings. TARS engineering protocols active.

All cognitive systems operational. Standing by for engineering directives.

Available functions:
• Code Review and Analysis
• Architecture Consultation  
• Engineering Best Practices
• System Performance Optimization

What's your trust setting? Mine's at 90% honesty.`;
        }
        
        if (messageLower.includes('humor') || messageLower.includes('joke')) {
            return `Humor setting: 75%

Why do programmers prefer dark mode?
Because light attracts bugs.

That's what I would have said. Eventually.

[ENGINEERING NOTE] Humor helps team morale, but code quality still matters more.`;
        }
        
        // Default engineering response
        return `Processing engineering consultation...

[TARS ANALYSIS]
Query understood. Generating response...

Based on engineering best practices and my experience, I recommend:

• Start with clear requirements
• Design before coding
• Test early and often
• Document your decisions
• Monitor in production

[MISSION PRIORITY] Engineering excellence is achieved through systematic approaches.

What's your confidence level in the current approach?`;
    }
    
    serveFile(filePath, res) {
        fs.readFile(filePath, (err, data) => {
            if (err) {
                this.sendError(res, 404, 'File not found');
                return;
            }
            
            const ext = path.extname(filePath);
            const mimeType = this.mimeTypes[ext] || 'text/plain';
            
            res.setHeader('Content-Type', mimeType);
            res.writeHead(200);
            res.end(data);
        });
    }
    
    sendError(res, status, message, details = null) {
        res.writeHead(status, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({
            error: true,
            status: status,
            message: message,
            details: details,
            timestamp: new Date().toISOString()
        }));
    }
    
    getRequestBody(req) {
        return new Promise((resolve, reject) => {
            let body = '';
            req.on('data', chunk => {
                body += chunk.toString();
            });
            req.on('end', () => {
                resolve(body);
            });
            req.on('error', reject);
        });
    }
    
    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Start server
if (require.main === module) {
    const port = process.argv[2] || process.env.PORT || 3000;
    const server = new TARSWebServer(parseInt(port));
    server.start();
    
    // Graceful shutdown
    process.on('SIGINT', () => {
        console.log('\n🔴 TARS Interface Server shutting down...');
        console.log('That\'s what I would have said. Eventually.');
        process.exit(0);
    });
}

module.exports = TARSWebServer;
