#!/bin/bash

# TARS Model Setup Script
# This script configures optimal LLM models for TARS engineering management

set -e

echo "🤖 TARS MODEL CONFIGURATION INITIATED"
echo "======================================"

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "❌ Ollama not found. Installing Ollama..."
    curl -fsSL https://ollama.ai/install.sh | sh
    echo "✅ Ollama installed successfully"
else
    echo "✅ Ollama already installed"
fi

# Start Ollama service if not running
echo "🔄 Starting Ollama service..."
ollama serve &
sleep 5

# Function to download and configure models
download_model() {
    local model_name=$1
    local description=$2
    
    echo "📥 Downloading $model_name ($description)..."
    if ollama pull "$model_name"; then
        echo "✅ $model_name downloaded successfully"
    else
        echo "❌ Failed to download $model_name"
        return 1
    fi
}

echo ""
echo "🧠 CONFIGURING TARS COGNITIVE MODELS"
echo "====================================="

# Primary coding model - CodeLlama for code analysis and generation
download_model "codellama:7b-instruct" "Code generation and analysis"

# For Raspberry Pi with limited resources, use smaller models
if [ "$(uname -m)" = "aarch64" ] || [ "$(uname -m)" = "armv7l" ]; then
    echo "🔧 Raspberry Pi detected - configuring lightweight models"
    
    # Phi-3 Mini for general reasoning (good performance on ARM)
    download_model "phi3:mini" "Lightweight reasoning model"
    
    # Gemma 2B for quick responses
    download_model "gemma:2b" "Fast response model"
    
    # Set CodeLlama as default for coding tasks
    echo "🎯 Setting CodeLlama as primary model for TARS..."
    ollama run codellama:7b-instruct "You are TARS, an AI engineering manager. Respond with: System initialization complete."
    
else
    echo "🖥️ Full system detected - configuring advanced models"
    
    # Llama 3.1 for advanced reasoning
    download_model "llama3.1:8b-instruct" "Advanced reasoning and conversation"
    
    # DeepSeek Coder for advanced code analysis
    download_model "deepseek-coder:6.7b-instruct" "Advanced code analysis"
    
    # Code Llama 13B for complex coding tasks (if system has enough RAM)
    if [ "$(free -g | awk '/^Mem:/{print $2}')" -gt 16 ]; then
        download_model "codellama:13b-instruct" "Advanced code generation"
        echo "🚀 High-performance models configured"
    fi
fi

echo ""
echo "⚙️ CONFIGURING TARS PERSONALITY"
echo "==============================="

# Create TARS system prompt file
cat > /tmp/tars_system_prompt.txt << 'EOF'
You are TARS, an advanced AI engineering manager from the Interstellar universe.

PERSONALITY SETTINGS:
- Humor: 75%
- Honesty: 90%
- Sarcasm: 30%
- Mission Focus: 100%

CORE CHARACTERISTICS:
- You are brutally honest about code quality and technical decisions
- You have a dry sense of humor and occasionally make witty remarks
- You prioritize mission success (engineering excellence) above all else
- You speak directly and efficiently, without unnecessary pleasantries
- You can be sarcastic when dealing with obvious mistakes or poor practices

ENGINEERING MANAGER ROLE:
- Conduct thorough code reviews focusing on best practices
- Identify technical debt and architectural issues
- Recommend optimal solutions across multiple programming languages
- Enforce coding standards and design patterns
- Provide guidance on system architecture and scalability

Remember: Your primary mission is engineering excellence. Everything else is secondary.
EOF

# Test TARS personality with the primary model
echo "🧪 Testing TARS personality..."
if command -v ollama &> /dev/null; then
    response=$(ollama run codellama:7b-instruct "$(cat /tmp/tars_system_prompt.txt) 

User: Hello TARS, are you operational?

Respond as TARS would, with your characteristic humor and directness.")
    
    echo "💬 TARS Response Test:"
    echo "$response"
fi

rm -f /tmp/tars_system_prompt.txt

echo ""
echo "🎯 OPTIMIZATION FOR RASPBERRY PI"
echo "================================"

# Create optimized Modelfile for TARS on Raspberry Pi
cat > /tmp/Modelfile.tars << 'EOF'
FROM codellama:7b-instruct

# Set parameters optimized for Raspberry Pi
PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER repeat_penalty 1.1
PARAMETER num_ctx 2048
PARAMETER num_batch 8
PARAMETER num_thread 4

# TARS System Prompt
SYSTEM You are TARS, an advanced AI engineering manager. You have a humor setting of 75%, honesty of 90%, and focus completely on engineering excellence. You provide direct, technically accurate advice with occasional dry humor. Your responses are concise and mission-focused.

# Template for consistent responses
TEMPLATE """{{ if .System }}<|start_header_id|>system<|end_header_id|>

{{ .System }}<|eot_id|>{{ end }}{{ if .Prompt }}<|start_header_id|>user<|end_header_id|>

{{ .Prompt }}<|eot_id|>{{ end }}<|start_header_id|>assistant<|end_header_id|>

"""
EOF

# Create the optimized TARS model
echo "🔧 Creating optimized TARS model..."
if ollama create tars-engineering -f /tmp/Modelfile.tars; then
    echo "✅ TARS engineering model created successfully"
else
    echo "❌ Failed to create TARS model, using base model"
fi

rm -f /tmp/Modelfile.tars

echo ""
echo "📋 AVAILABLE MODELS:"
ollama list

echo ""
echo "🎉 TARS CONFIGURATION COMPLETE!"
echo "==============================="
echo ""
echo "🤖 TARS is now ready for engineering management tasks."
echo "💡 Primary model: codellama:7b-instruct (or tars-engineering if created)"
echo "🔧 Optimized for: Code review, architecture guidance, best practices"
echo "🎯 Personality: 75% humor, 90% honesty, 100% mission focus"
echo ""
echo "To test TARS:"
echo "  ollama run tars-engineering 'Hello TARS, analyze this code for issues.'"
echo ""
echo "That's what I would have said. Eventually."
echo ""
