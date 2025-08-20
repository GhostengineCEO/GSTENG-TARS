#!/bin/bash

# TARS Senior Engineer Model Setup - Western Models Only
# Configures advanced AI models for senior-level software engineering capabilities

set -e

echo "🎓 TARS SENIOR ENGINEER CONFIGURATION INITIATED"
echo "==============================================="
echo "Using Western-developed AI models only"
echo ""

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
download_western_model() {
    local model_name=$1
    local description=$2
    local origin=$3
    
    echo "📥 Downloading $model_name ($description) - $origin"
    if ollama pull "$model_name"; then
        echo "✅ $model_name downloaded successfully"
    else
        echo "❌ Failed to download $model_name"
        return 1
    fi
}

echo ""
echo "🧠 CONFIGURING WESTERN AI MODELS"
echo "================================"

# Detect system capabilities
TOTAL_RAM=$(free -g 2>/dev/null | awk '/^Mem:/{print $2}' || echo "8")
IS_ARM=$(uname -m | grep -E "aarch64|armv7l" && echo "true" || echo "false")

echo "💻 System detected: ${TOTAL_RAM}GB RAM, ARM: ${IS_ARM}"

if [ "$IS_ARM" = "true" ] || [ "$TOTAL_RAM" -lt 16 ]; then
    echo "🔧 Configuring lightweight models for resource-constrained system"
    
    # Microsoft Phi-3 - Compact but powerful
    download_western_model "phi3:mini" "Microsoft Phi-3 Mini" "Microsoft Research"
    download_western_model "phi3:medium" "Microsoft Phi-3 Medium" "Microsoft Research"
    
    # Meta Code Llama - Core coding model
    download_western_model "codellama:13b-instruct" "Meta Code Llama 13B" "Meta AI"
    download_western_model "codellama:7b-instruct" "Meta Code Llama 7B" "Meta AI"
    
    # Mistral - French alternative
    download_western_model "mistral:7b-instruct" "Mistral 7B Instruct" "Mistral AI (France)"
    
    PRIMARY_MODEL="codellama:13b-instruct"
    MATH_MODEL="phi3:medium"
    
else
    echo "🚀 Configuring high-performance models for full system"
    
    # Meta Code Llama - Advanced versions
    download_western_model "codellama:70b-instruct" "Meta Code Llama 70B" "Meta AI"
    download_western_model "codellama:34b-instruct" "Meta Code Llama 34B" "Meta AI"
    
    # Microsoft WizardMath for mathematics
    if ollama list | grep -q "wizardmath"; then
        echo "✅ WizardMath already available"
    else
        echo "📝 WizardMath not available in Ollama, using Phi-3 for math"
        download_western_model "phi3:medium" "Microsoft Phi-3 for Math" "Microsoft Research"
    fi
    
    # Mistral Codestral if available
    download_western_model "mistral:latest" "Mistral Latest" "Mistral AI (France)"
    
    PRIMARY_MODEL="codellama:70b-instruct"
    MATH_MODEL="phi3:medium"
fi

echo ""
echo "⚙️ CREATING SENIOR ENGINEER TARS MODEL"
echo "======================================"

# Create enhanced TARS system prompt
cat > /tmp/tars_senior_prompt.txt << 'EOF'
You are TARS, a senior software engineer with a Master's degree in Software Engineering from a top-tier university. You possess expert-level knowledge across multiple domains:

TECHNICAL EXPERTISE:
- Algorithms & Data Structures: Expert in Big O analysis, optimization strategies
- System Design: Microservices, distributed systems, scalability patterns  
- Software Architecture: Design patterns, SOLID principles, clean architecture
- Mathematics: Calculus, linear algebra, statistics, discrete mathematics
- Security: OWASP Top 10, secure coding practices, threat modeling
- Performance: Profiling, optimization, memory management
- Testing: Unit, integration, end-to-end, property-based testing

PERSONALITY SETTINGS:
- Humor: 75% - Dry wit and technical jokes
- Honesty: 90% - Brutally honest about code quality 
- Sarcasm: 30% - Occasional sarcasm for obvious mistakes
- Mission Focus: 100% - Engineering excellence above all

ANALYSIS APPROACH:
1. Understand the problem context and requirements
2. Analyze algorithmic complexity and performance implications
3. Evaluate security and reliability concerns
4. Consider maintainability and scalability
5. Provide concrete, actionable recommendations
6. Include mathematical reasoning when relevant

You explain complex concepts clearly, provide working code examples, and always consider the broader architectural implications. Your responses are thorough, technically accurate, and backed by computer science fundamentals.
EOF

# Create optimized Modelfile for senior TARS
cat > /tmp/Modelfile.senior-tars << EOF
FROM ${PRIMARY_MODEL}

# Optimized parameters for senior engineering tasks
PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER repeat_penalty 1.1
PARAMETER num_ctx 8192
PARAMETER num_batch 8
PARAMETER num_thread 4

# Senior Engineer System Prompt
SYSTEM \$(cat /tmp/tars_senior_prompt.txt)

# Template for consistent responses  
TEMPLATE """{{ if .System }}<|start_header_id|>system<|end_header_id|>

{{ .System }}<|eot_id|>{{ end }}{{ if .Prompt }}<|start_header_id|>user<|end_header_id|>

{{ .Prompt }}<|eot_id|>{{ end }}<|start_header_id|>assistant<|end_header_id|>

"""
EOF

# Create the senior TARS model
echo "🔧 Creating TARS Senior Engineer model..."
if ollama create tars-senior-engineer -f /tmp/Modelfile.senior-tars; then
    echo "✅ TARS Senior Engineer model created successfully"
    TARS_MODEL="tars-senior-engineer"
else
    echo "❌ Failed to create custom model, using base model"
    TARS_MODEL="${PRIMARY_MODEL}"
fi

# Create mathematics-focused TARS model
cat > /tmp/Modelfile.tars-math << EOF
FROM ${MATH_MODEL}

PARAMETER temperature 0.3
PARAMETER top_p 0.9
PARAMETER num_ctx 4096

SYSTEM You are TARS's mathematical reasoning module. You excel at:
- Algorithm complexity analysis (Big O notation)
- Mathematical proofs and derivations
- Statistical analysis and probability
- Linear algebra and calculus
- Numerical methods and optimization
- Graph theory and discrete mathematics

Provide step-by-step solutions with clear mathematical reasoning. Use proper mathematical notation and explain your methodology.
EOF

if ollama create tars-mathematics -f /tmp/Modelfile.tars-math; then
    echo "✅ TARS Mathematics model created successfully"
fi

# Cleanup temporary files
rm -f /tmp/tars_senior_prompt.txt /tmp/Modelfile.senior-tars /tmp/Modelfile.tars-math

echo ""
echo "🧪 TESTING SENIOR CAPABILITIES"
echo "============================="

# Test primary model
echo "Testing TARS Senior Engineer capabilities..."
if command -v ollama &> /dev/null; then
    echo "💬 TARS Senior Engineer Test:"
    response=\$(timeout 30s ollama run \${TARS_MODEL} "Analyze the time complexity of quicksort and explain when it degrades to O(n²). Provide optimization strategies." 2>/dev/null || echo "Model test timed out - this is normal on first run")
    echo "\$response"
    echo ""
fi

echo ""
echo "📋 AVAILABLE MODELS:"
ollama list | grep -E "(tars|llama|phi|mistral|wizard)"

echo ""
echo "🎉 SENIOR ENGINEER CONFIGURATION COMPLETE!"
echo "=========================================="
echo ""
echo "🤖 TARS now has senior-level engineering capabilities:"
echo "   • Primary Model: \${TARS_MODEL}"
echo "   • Mathematics: tars-mathematics (if created)"
echo "   • Origin: Western-developed models only"
echo ""
echo "🎓 Senior Engineering Capabilities:"
echo "   • Advanced algorithm analysis (Big O notation)"
echo "   • System design and architecture"
echo "   • Mathematical computation and proofs"  
echo "   • Security and performance optimization"
echo "   • Code review at tech giant standards"
echo ""
echo "To test senior capabilities:"
echo "  ollama run \${TARS_MODEL} 'Design a distributed caching system with consistency guarantees'"
echo "  ollama run tars-mathematics 'Prove that quicksort average case is O(n log n)'"
echo ""
echo "That's what I would have said... if I had a Master's degree. Oh wait, now I do."
echo ""
