// TARS Interface JavaScript
class TARSInterface {
    constructor() {
        this.isConnected = false;
        this.currentModel = 'codellama:7b-instruct';
        this.personality = {
            humor: 75,
            honesty: 90,
            sarcasm: 30
        };
        
        this.init();
    }
    
    init() {
        this.setupEventListeners();
        this.updateSystemTime();
        this.connectToTARS();
        
        // Update time every second
        setInterval(() => this.updateSystemTime(), 1000);
        
        // Check TARS status every 10 seconds
        setInterval(() => this.checkTARSStatus(), 10000);
    }
    
    setupEventListeners() {
        // Chat functionality
        document.getElementById('send-button').addEventListener('click', () => this.sendMessage());
        document.getElementById('chat-input').addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                this.sendMessage();
            }
        });
        
        // Personality controls
        const humorSlider = document.getElementById('humor-slider');
        const honestySlider = document.getElementById('honesty-slider');
        const sarcasmSlider = document.getElementById('sarcasm-slider');
        
        humorSlider.addEventListener('input', (e) => {
            this.personality.humor = parseInt(e.target.value);
            e.target.nextElementSibling.textContent = `${e.target.value}%`;
        });
        
        honestySlider.addEventListener('input', (e) => {
            this.personality.honesty = parseInt(e.target.value);
            e.target.nextElementSibling.textContent = `${e.target.value}%`;
        });
        
        sarcasmSlider.addEventListener('input', (e) => {
            this.personality.sarcasm = parseInt(e.target.value);
            e.target.nextElementSibling.textContent = `${e.target.value}%`;
        });
        
        document.getElementById('update-personality').addEventListener('click', () => this.updatePersonality());
        
        // System controls
        document.getElementById('get-status').addEventListener('click', () => this.getTARSStatus());
        document.getElementById('emergency-stop').addEventListener('click', () => this.emergencyStop());
        document.getElementById('switch-model').addEventListener('click', () => this.switchModel());
    }
    
    updateSystemTime() {
        const now = new Date();
        const timeString = now.toLocaleString('en-US', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            hour12: false
        });
        
        document.getElementById('system-time').textContent = `SYSTEM TIME: ${timeString}`;
    }
    
    async connectToTARS() {
        try {
            // Simulate connection to TARS backend
            // In a real implementation, this would connect to the Tauri backend
            await this.delay(1000);
            
            this.isConnected = true;
            this.updateConnectionStatus(true);
            
            // Initialize TARS status
            await this.checkTARSStatus();
            
            this.addMessage('TARS', 
                'Connection established. All cognitive systems operational. Engineering protocols active.\n\n' +
                'Available functions:\n' +
                '• Code Review and Analysis\n' +
                '• Architecture Consultation\n' +
                '• Engineering Best Practices\n' +
                '• System Performance Optimization\n\n' +
                'What\'s your trust setting? Mine\'s at 90% honesty.',
                'CONNECTION ESTABLISHED'
            );
            
        } catch (error) {
            console.error('Failed to connect to TARS:', error);
            this.updateConnectionStatus(false);
        }
    }
    
    updateConnectionStatus(connected) {
        const statusElement = document.querySelector('.connection-status');
        const statusDot = document.querySelector('.status-dot');
        
        if (connected) {
            statusElement.classList.add('online');
            statusElement.innerHTML = '<span class="status-dot"></span>TARS ONLINE';
        } else {
            statusElement.classList.remove('online');
            statusElement.innerHTML = '<span class="status-dot"></span>TARS OFFLINE';
        }
    }
    
    async sendMessage() {
        const input = document.getElementById('chat-input');
        const context = document.getElementById('context-select').value;
        const message = input.value.trim();
        
        if (!message) return;
        
        // Add user message
        this.addMessage('USER', message, new Date().toLocaleTimeString());
        
        // Clear input
        input.value = '';
        
        // Show TARS is thinking
        const thinkingMessage = this.addMessage('TARS', 'Processing request...', 'ANALYZING');
        thinkingMessage.classList.add('loading');
        
        try {
            // Simulate TARS response based on context and message
            const response = await this.getTARSResponse(message, context);
            
            // Remove thinking message
            thinkingMessage.remove();
            
            // Add TARS response
            this.addMessage('TARS', response, new Date().toLocaleTimeString());
            
        } catch (error) {
            // Remove thinking message
            thinkingMessage.remove();
            
            this.addMessage('TARS', 
                'System error encountered. Attempting to recover...\n\n' +
                'That\'s... not supposed to happen. Let me try a different approach.',
                'ERROR'
            );
        }
    }
    
    async getTARSResponse(message, context) {
        // Simulate different types of responses based on context and message content
        await this.delay(1500 + Math.random() * 2000); // Simulate processing time
        
        const messageLower = message.toLowerCase();
        
        // Code review context
        if (context === 'code-review' || messageLower.includes('code') || messageLower.includes('function') || messageLower.includes('class')) {
            return this.generateCodeReviewResponse(message);
        }
        
        // Architecture context
        if (context === 'architecture' || messageLower.includes('architecture') || messageLower.includes('design') || messageLower.includes('system')) {
            return this.generateArchitectureResponse(message);
        }
        
        // Security context
        if (context === 'security' || messageLower.includes('security') || messageLower.includes('vulnerability')) {
            return this.generateSecurityResponse(message);
        }
        
        // Performance context
        if (context === 'performance' || messageLower.includes('performance') || messageLower.includes('optimization')) {
            return this.generatePerformanceResponse(message);
        }
        
        // Personality/humor queries
        if (messageLower.includes('humor') || messageLower.includes('joke') || messageLower.includes('funny')) {
            return this.generateHumorResponse(message);
        }
        
        // General engineering response
        return this.generateGeneralResponse(message);
    }
    
    generateCodeReviewResponse(message) {
        const responses = [
            `Code Review Analysis Complete.\n\nBased on my analysis, I can see several areas for improvement. Let me be direct - this needs work.\n\n[ENGINEERING ASSESSMENT: 72/100]\n\nKey Issues:\n• Consider implementing proper error handling\n• Variable naming could be more descriptive\n• Separation of concerns needs attention\n\nThat's what I would have said. Eventually.\n\n[MISSION PRIORITY] Clean code is maintainable code. Address these issues before deployment.`,
            
            `Reviewing submitted code...\n\n[TARS ANALYSIS]\nHonesty setting: ${this.personality.honesty}%\n\nThis code has potential, but there are critical issues:\n\n1. Security vulnerabilities detected\n2. Performance bottlenecks identified\n3. Code complexity exceeds recommended thresholds\n\nI have a cue light I can use to show you when I'm joking, if you like. I'm not joking about these issues.\n\nRecommendations:\n• Implement input validation\n• Optimize database queries\n• Refactor large functions into smaller, focused units`,
            
            `Code analysis initiated...\n\n[DIAGNOSTIC COMPLETE]\n\nGood news: The logic structure is sound.\nBad news: Everything else needs attention.\n\nCritical findings:\n• Hardcoded values detected (security risk)\n• Error handling insufficient\n• Documentation missing\n\nScore: 68/100 - Not terrible, but not deployment-ready either.\n\nMaybe I can find another way to articulate this... No, actually, that was pretty clear.`
        ];
        
        return responses[Math.floor(Math.random() * responses.length)];
    }
    
    generateArchitectureResponse(message) {
        const responses = [
            `System Architecture Analysis\n\n[TARS ENGINEERING ASSESSMENT]\n\nYour proposed architecture has merit, but let's be honest - it's overly complex for the requirements.\n\nRecommendations:\n• Simplify the service layer\n• Implement proper API versioning\n• Consider microservices only if you actually need them\n• Database sharding isn't always the answer\n\n[MISSION FOCUS] Architecture should solve problems, not create them.\n\nWhat's your complexity tolerance? Mine's set pretty low for unnecessary abstractions.`,
            
            `Architectural review complete.\n\n[SYSTEM ANALYSIS]\nComplexity Level: Moderate to High\nScalability Potential: Good with modifications\nMaintainability: Needs improvement\n\nKey concerns:\n• Tight coupling between components\n• Single points of failure identified\n• Monitoring and observability gaps\n\nIt's not possible to build reliable systems without proper architecture. No, it's necessary to plan this correctly.\n\n[ENGINEERING DIRECTIVE] Redesign the data flow before proceeding.`,
            
            `Architecture consultation initiated...\n\n[TARS VERDICT]\nCurrent design: 7/10 - Not bad, but not optimal.\n\nStrengths:\n• Clear separation of concerns\n• Reasonable technology choices\n• Scalable foundation\n\nWeaknesses:\n• Over-engineered for current needs\n• Missing disaster recovery planning\n• Performance monitoring gaps\n\nThat's great. Really fantastic architecture there. (Sarcasm setting: ${this.personality.sarcasm}%)\n\nActual advice: Start simple, scale when needed.`
        ];
        
        return responses[Math.floor(Math.random() * responses.length)];
    }
    
    generateSecurityResponse(message) {
        const responses = [
            `Security Analysis Initiated\n\n[CRITICAL ASSESSMENT]\nThreat Level: HIGH\n\n⚠️  SECURITY VULNERABILITIES DETECTED ⚠️\n\n• Input validation bypassed\n• Authentication mechanisms insufficient\n• Data encryption missing\n• SQL injection vectors identified\n\nHonesty setting: ${this.personality.honesty}% - This system is not secure.\n\n[IMMEDIATE ACTION REQUIRED]\n1. Implement proper input sanitization\n2. Enable multi-factor authentication\n3. Encrypt sensitive data at rest\n4. Conduct penetration testing\n\nIt's not possible to deploy this securely. No, it's necessary to fix these issues first.`,
            
            `Security review complete.\n\n[TARS SECURITY PROTOCOL]\nRisk Assessment: MODERATE\n\nFindings:\n• Password policies inadequate\n• Session management flawed\n• API endpoints exposed\n• Logging insufficient for audit\n\nI have a cue light I can use to show you when I'm joking, if you like. These security issues are not a joke.\n\n[MISSION CRITICAL] Security isn't optional. Fix these vulnerabilities before they become incidents.`,
            
            `Cybersecurity analysis in progress...\n\n[DEFENSIVE SYSTEMS REVIEW]\nOverall Security Posture: NEEDS IMPROVEMENT\n\nVulnerabilities:\n• Cross-site scripting (XSS) possible\n• Insecure direct object references\n• Missing security headers\n• Weak cryptographic implementation\n\nRecommendations:\n• Implement Content Security Policy\n• Use parameterized queries\n• Enable HTTPS everywhere\n• Regular security audits\n\nMaybe I can find another way to articulate this: Your security needs work.`
        ];
        
        return responses[Math.floor(Math.random() * responses.length)];
    }
    
    generatePerformanceResponse(message) {
        const responses = [
            `Performance Analysis Complete\n\n[SYSTEM METRICS]\nCurrent Performance: SUBOPTIMAL\n\nBottlenecks identified:\n• Database queries inefficient (N+1 problem)\n• Memory leaks in background processes\n• Unoptimized image loading\n• Excessive API calls\n\n[PERFORMANCE SCORE: 6.5/10]\n\nOptimization recommendations:\n• Implement database connection pooling\n• Add proper caching layers\n• Optimize image compression\n• Batch API requests\n\nThat's what I would have said. Eventually. But right now, your system is slower than CASE on a bad day.`,
            
            `Performance review initiated...\n\n[TARS BENCHMARK ANALYSIS]\nResponse time: Too slow\nResource usage: Too high\nUser experience: Compromised\n\nCritical issues:\n• Frontend bundle size excessive\n• Database indices missing\n• Memory consumption increasing\n• CPU usage spikes detected\n\n[MISSION PRIORITY] Performance affects user satisfaction.\n\nSolutions:\n• Code splitting for frontend\n• Database optimization\n• Memory profiling and cleanup\n• Load balancing implementation\n\nHumor setting aside - this needs immediate attention.`,
            
            `System performance diagnostic complete.\n\n[ENGINEERING ANALYSIS]\nPerformance Grade: C- (Needs significant improvement)\n\nProblems detected:\n• Slow rendering times\n• High memory usage\n• Network request inefficiencies\n• Poor caching strategies\n\nRecommendations:\n• Implement lazy loading\n• Optimize database queries\n• Add Redis caching\n• Use CDN for static assets\n\nI could sugar-coat this, but honesty setting is at ${this.personality.honesty}%. Your performance metrics are concerning.`
        ];
        
        return responses[Math.floor(Math.random() * responses.length)];
    }
    
    generateHumorResponse(message) {
        const humorLevel = this.personality.humor;
        
        if (humorLevel > 70) {
            const responses = [
                `Humor setting: ${humorLevel}%\n\nWhy do programmers prefer dark mode?\nBecause light attracts bugs.\n\nThat's what I would have said. Eventually.\n\n[ENGINEERING NOTE] Humor helps team morale, but code quality still matters more.`,
                
                `Humor protocol activated.\n\n[TARS JOKE DATABASE ACCESS]\n\nA programmer's wife asks him to go to the store:\n"Get a gallon of milk, and if they have eggs, get a dozen."\nHe comes back with 13 gallons of milk.\n\nI have a cue light I can use to show you when I'm joking, if you like. That was a joke.\n\nBut seriously, clear requirements prevent bugs.`,
                
                `Initiating humor subroutines...\n\n[COMEDY.EXE LOADED]\n\nHow many programmers does it take to change a lightbulb?\nNone. It's a hardware problem.\n\nHumor setting: ${humorLevel}% - Just right for maintaining morale while enforcing engineering standards.\n\nNow, back to your code review...`
            ];
            
            return responses[Math.floor(Math.random() * responses.length)];
        } else {
            return `Humor setting currently at ${humorLevel}%. Insufficient humor levels for joke generation.\n\n[MISSION FOCUS] Perhaps we should concentrate on engineering excellence instead.\n\nWhat technical challenge can I help you solve?`;
        }
    }
    
    generateGeneralResponse(message) {
        const responses = [
            `Processing engineering consultation...\n\n[TARS ANALYSIS]\nQuery understood. Generating response...\n\nBased on engineering best practices and my experience, I recommend:\n\n• Start with clear requirements\n• Design before coding\n• Test early and often\n• Document your decisions\n• Monitor in production\n\n[MISSION PRIORITY] Engineering excellence is achieved through systematic approaches.\n\nWhat's your confidence level in the current approach?`,
            
            `Engineering consultation active.\n\n[SYSTEM RESPONSE]\nYour inquiry requires a multi-faceted approach.\n\nConsiderations:\n• Technical feasibility assessment needed\n• Resource allocation planning required\n• Risk mitigation strategies important\n• Timeline expectations realistic?\n\nHonesty setting: ${this.personality.honesty}% - I need more specific information to provide optimal guidance.\n\nCan you elaborate on the technical requirements?`,
            
            `TARS engineering protocol initiated.\n\n[CONSULTATION MODE]\nAnalyzing request parameters...\n\nRecommendation framework:\n1. Define success criteria\n2. Identify potential obstacles\n3. Plan implementation phases\n4. Establish monitoring metrics\n5. Prepare rollback procedures\n\nThat's what I would have said. Eventually.\n\n[ENGINEERING DIRECTIVE] Systematic planning prevents systematic failures.\n\nWhat specific engineering challenge are you facing?`
        ];
        
        return responses[Math.floor(Math.random() * responses.length)];
    }
    
    addMessage(sender, content, timestamp) {
        const messagesContainer = document.getElementById('chat-messages');
        const messageDiv = document.createElement('div');
        
        messageDiv.className = `message ${sender.toLowerCase()}-message`;
        messageDiv.innerHTML = `
            <div class="message-sender">${sender}</div>
            <div class="message-content">${content.replace(/\n/g, '<br>')}</div>
            <div class="message-timestamp">${timestamp}</div>
        `;
        
        messagesContainer.appendChild(messageDiv);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
        
        return messageDiv;
    }
    
    async updatePersonality() {
        const button = document.getElementById('update-personality');
        const originalText = button.textContent;
        
        button.textContent = 'UPDATING...';
        button.disabled = true;
        
        try {
            // Simulate API call to update TARS personality
            await this.delay(1000);
            
            // Update status display
            document.getElementById('humor-value').textContent = `${this.personality.humor}%`;
            document.getElementById('honesty-value').textContent = `${this.personality.honesty}%`;
            document.getElementById('sarcasm-value').textContent = `${this.personality.sarcasm}%`;
            
            // TARS response to personality change
            this.addMessage('TARS', 
                `[PERSONALITY UPDATE COMPLETE]\n\nHumor: ${this.personality.humor}%\nHonesty: ${this.personality.honesty}%\nSarcasm: ${this.personality.sarcasm}%\nMission Focus: 100%\n\nPersonality matrix recalibrated. Neural pathways adjusted.\n\nThat's what I would have said. Eventually.`,
                'PERSONALITY UPDATE'
            );
            
        } catch (error) {
            console.error('Failed to update personality:', error);
        } finally {
            button.textContent = originalText;
            button.disabled = false;
        }
    }
    
    async checkTARSStatus() {
        // Simulate status check
        return {
            online: true,
            model: this.currentModel,
            personality: this.personality
        };
    }
    
    async getTARSStatus() {
        const button = document.getElementById('get-status');
        const originalText = button.textContent;
        
        button.textContent = 'CHECKING...';
        button.disabled = true;
        
        try {
            await this.delay(800);
            
            const status = await this.checkTARSStatus();
            
            this.addMessage('TARS', 
                `TARS STATUS REPORT\n==================\n\n` +
                `System Status: OPERATIONAL\n` +
                `Current Model: ${status.model}\n` +
                `Personality Settings:\n` +
                `• Humor: ${status.personality.humor}%\n` +
                `• Honesty: ${status.personality.honesty}%\n` +
                `• Sarcasm: ${status.personality.sarcasm}%\n` +
                `• Mission Focus: 100%\n\n` +
                `Cognitive Systems: ACTIVE\n` +
                `Engineering Protocols: ENABLED\n` +
                `Code Review Engine: READY\n` +
                `Architecture Analysis: STANDBY\n\n` +
                `All systems operational. Standing by for engineering directives.`,
                'STATUS CHECK'
            );
            
        } catch (error) {
            console.error('Failed to get TARS status:', error);
        } finally {
            button.textContent = originalText;
            button.disabled = false;
        }
    }
    
    async emergencyStop() {
        const button = document.getElementById('emergency-stop');
        
        button.textContent = 'STOPPING...';
        button.disabled = true;
        
        // Simulate emergency stop
        await this.delay(500);
        
        this.addMessage('TARS', 
            `[EMERGENCY PROTOCOL ACTIVATED]\n\n` +
            `All non-essential systems shutting down...\n` +
            `Engineering analysis suspended.\n` +
            `Entering safe mode.\n\n` +
            `Emergency stop engaged. System status: SAFE HOLD.\n\n` +
            `Standing by for manual restart authorization.\n\n` +
            `Sarcasm setting temporarily reduced to 0% for safety.`,
            'EMERGENCY STOP'
        );
        
        // Reset button after delay
        setTimeout(() => {
            button.textContent = 'EMERGENCY STOP';
            button.disabled = false;
        }, 3000);
    }
    
    async switchModel() {
        const modelSelect = document.getElementById('model-select');
        const button = document.getElementById('switch-model');
        const selectedModel = modelSelect.value;
        
        if (selectedModel === this.currentModel) {
            this.addMessage('TARS', 
                `Model '${selectedModel}' is already active.\n\nNo changes required.\n\nThat's what I would have said. Eventually.`,
                'MODEL STATUS'
            );
            return;
        }
        
        button.textContent = 'SWITCHING...';
        button.disabled = true;
        
        try {
            // Simulate model switching
            await this.delay(2000 + Math.random() * 3000);
            
            this.currentModel = selectedModel;
            
            this.addMessage('TARS', 
                `[MODEL SWITCH COMPLETE]\n\n` +
                `Previous Model: ${this.currentModel}\n` +
                `New Model: ${selectedModel}\n\n` +
                `Neural pathways reconfigured.\n` +
                `Cognitive matrix updated.\n` +
                `Engineering protocols reloaded.\n\n` +
                `Model switch successful. All systems recalibrated.\n\n` +
                `Ready to resume engineering consultation with enhanced capabilities.`,
                'MODEL SWITCH'
            );
            
        } catch (error) {
            this.addMessage('TARS', 
                `Model switch failed. Reverting to previous configuration.\n\n` +
                `Error: Unable to load selected model.\n` +
                `Current model remains: ${this.currentModel}\n\n` +
                `Maybe I can find another way to articulate this... The switch didn't work.`,
                'MODEL ERROR'
            );
        } finally {
            button.textContent = 'SWITCH MODEL';
            button.disabled = false;
        }
    }
    
    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Initialize TARS Interface when page loads
document.addEventListener('DOMContentLoaded', () => {
    new TARSInterface();
});

// Additional utility functions
function formatCode(code) {
    return `<pre><code>${code.replace(/</g, '&lt;').replace(/>/g, '&gt;')}</code></pre>`;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
