# TARS + Llama Integration Guide

## 🤖 Overview
TARS is already fully configured to use local Llama models through Ollama. The integration system automatically routes requests between local and cloud models based on availability and complexity.

## 📋 Prerequisites
- Your downloaded Llama model
- Ollama (will be installed automatically if needed)

## 🚀 Integration Steps

### Step 1: Run the TARS Model Setup Script
```bash
# Make the script executable and run it
chmod +x scripts/setup-tars-models.sh
./scripts/setup-tars-models.sh
```

This script will:
- ✅ Install Ollama if not present
- ✅ Download optimized models for TARS (CodeLlama, Phi-3, Gemma)
- ✅ Create a custom "tars-engineering" model with TARS personality
- ✅ Configure optimal parameters for Raspberry Pi

### Step 2: Add Your Downloaded Llama Model
If you have a specific Llama model file, add it to Ollama:

```bash
# For GGUF format models
ollama create my-llama -f Modelfile

# Or import directly if it's a standard model
ollama pull llama3.1:8b-instruct
```

### Step 3: Switch TARS to Use Your Model
The TARS system can switch between models dynamically:

```bash
# List available models
ollama list

# Set your preferred model as default
# This updates the CURRENT_MODEL in local_llm.rs
```

## 🧠 How TARS AI Routing Works

### Automatic Model Selection
TARS intelligently chooses between local and cloud models:

```rust
// From ai/router.rs
pub enum LlmSource {
    Local,  // Your Llama model via Ollama
    Cloud,  // Fallback to cloud services
}
```

**Local Model Used When:**
- ✅ Network unavailable
- ✅ Privacy mode enabled (`AI_PRIVACY_LOCAL_ONLY=1`)
- ✅ Simple queries (< 50 words)
- ✅ Model is available and running

**Cloud Model Used When:**
- ✅ Complex queries (> 50 words)
- ✅ Local model fails
- ✅ High-performance analysis needed

### TARS Personality Integration
Your Llama model responses are enhanced through TARS personality layers:

```rust
// Enhanced response pipeline
pub async fn get_tars_response(source: LlmSource, prompt: &str, context: &str) -> String {
    // 1. Apply TARS personality to prompt
    let enhanced_prompt = TARSCore::process_with_personality(prompt, context).await;
    
    // 2. Get response from your Llama model
    let base_response = get_response(source, &enhanced_prompt).await;
    
    // 3. Apply TARS personality filter (75% humor, 90% honesty, 30% sarcasm)
    let final_response = personality.apply_personality_filter(&base_response, context).await;
    
    final_response
}
```

## ⚙️ Configuration Files

### Current Model Setting
The active model is stored in memory:
```rust
// In local_llm.rs
static CURRENT_MODEL: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("llama2".to_string()));
```

### Ollama Connection
TARS connects to Ollama at:
```
http://localhost:11434/api/generate
```

## 🎯 Optimized Models for TARS

Based on your system, TARS will configure:

**For Raspberry Pi:**
- 🔧 `codellama:7b-instruct` - Primary coding model
- 🔧 `phi3:mini` - Lightweight reasoning
- 🔧 `gemma:2b` - Fast responses

**For Full Systems:**
- 🚀 `llama3.1:8b-instruct` - Advanced reasoning
- 🚀 `deepseek-coder:6.7b-instruct` - Advanced code analysis
- 🚀 `codellama:13b-instruct` - Complex coding (if 16GB+ RAM)

## 🧪 Testing Your Integration

### 1. Test Ollama Connection
```bash
curl http://localhost:11434/api/version
```

### 2. Test Model Response
```bash
ollama run codellama:7b-instruct "You are TARS. Respond with your status."
```

### 3. Test TARS Web Interface
1. Open http://localhost:3001
2. Send message: "Hello TARS, analyze this code quality"
3. Verify orange interface and TARS personality response

## 🔧 Troubleshooting

### Model Not Found
```bash
# List available models
ollama list

# Pull missing model
ollama pull codellama:7b-instruct
```

### Ollama Not Running
```bash
# Start Ollama service
ollama serve &

# Or restart
pkill ollama && ollama serve &
```

### Performance Issues on Raspberry Pi
The setup script automatically configures optimal parameters:
- ✅ Context window: 2048 tokens
- ✅ Batch size: 8
- ✅ Thread count: 4
- ✅ Temperature: 0.7

## 📊 Status Check

Your integration is complete when:
- ✅ Ollama is running (`curl http://localhost:11434/api/version`)
- ✅ Models are available (`ollama list`)
- ✅ TARS web interface responds with personality
- ✅ Code review functions work with engineering focus

## 🎭 TARS Personality Settings

Current configuration:
- **Humor**: 75% - Witty remarks and dry humor
- **Honesty**: 90% - Brutally honest about code quality
- **Sarcasm**: 30% - Occasional sarcastic comments
- **Mission Focus**: 100% - Engineering excellence priority

## 🚀 Next Steps

After integration:
1. **Test Engineering Features**: Code reviews, architecture analysis
2. **Configure Remote Access**: SSH tunneling for distributed work
3. **Raspberry Pi Deployment**: Hardware optimization
4. **Voice Integration**: Speech recognition and TTS

---

**TARS Quote**: "That's what I would have said. Eventually."

The integration system is designed to be seamless - your Llama model becomes TARS's "brain" while the personality system provides the characteristic humor, honesty, and engineering focus.
