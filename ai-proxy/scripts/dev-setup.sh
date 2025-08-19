#!/bin/bash

# Development setup script for WriteMagic AI Proxy

set -e

echo "🚀 Setting up WriteMagic AI Proxy development environment..."

# Check Node.js version
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ first."
    exit 1
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js version 18+ is required. Current version: $(node -v)"
    exit 1
fi

echo "✅ Node.js version: $(node -v)"

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Create logs directory
echo "📁 Creating logs directory..."
mkdir -p logs

# Copy environment file if it doesn't exist
if [ ! -f .env ]; then
    echo "📄 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env file with your API keys and configuration"
else
    echo "✅ .env file already exists"
fi

# Check if API keys are configured
if ! grep -q "your_.*_api_key_here" .env; then
    echo "✅ API keys appear to be configured"
else
    echo "⚠️  API keys not configured yet. Please edit .env file."
fi

# Create development SSL certificates (optional)
if [ ! -d "ssl" ]; then
    echo "🔒 Creating development SSL certificates..."
    mkdir -p ssl
    
    # Generate self-signed certificate for development
    openssl req -x509 -newkey rsa:4096 -keyout ssl/key.pem -out ssl/cert.pem -days 365 -nodes -subj "/C=US/ST=Dev/L=Dev/O=WriteMagic/CN=localhost" 2>/dev/null || {
        echo "⚠️  OpenSSL not available. Skipping SSL certificate generation."
        echo "   You can still run the proxy over HTTP for development."
    }
fi

# Test configuration
echo "🧪 Testing configuration..."
node -e "
try {
  const config = require('./config');
  console.log('✅ Configuration loaded successfully');
  
  const providers = Object.keys(config.ai.providers).filter(
    name => config.ai.providers[name].apiKey && 
           config.ai.providers[name].apiKey !== 'your_' + name + '_api_key_here'
  );
  
  if (providers.length === 0) {
    console.log('⚠️  No AI providers configured with API keys');
  } else {
    console.log('✅ Configured providers: ' + providers.join(', '));
  }
} catch (error) {
  console.error('❌ Configuration error:', error.message);
  process.exit(1);
}
"

# Run tests if requested
if [ "$1" = "--with-tests" ]; then
    echo "🧪 Running tests..."
    npm test
fi

# Create PM2 ecosystem file for production deployment
cat > ecosystem.config.js << 'EOF'
module.exports = {
  apps: [{
    name: 'writemagic-ai-proxy',
    script: 'server.js',
    instances: 'max',
    exec_mode: 'cluster',
    env: {
      NODE_ENV: 'production',
      PORT: 3001
    },
    env_production: {
      NODE_ENV: 'production'
    },
    error_file: 'logs/err.log',
    out_file: 'logs/out.log',
    log_file: 'logs/combined.log',
    time: true,
    max_memory_restart: '500M',
    node_args: '--max-old-space-size=400'
  }]
};
EOF

echo "✅ Created PM2 ecosystem configuration"

# Create Docker setup files
cat > Dockerfile << 'EOF'
FROM node:18-alpine

# Create app directory
WORKDIR /app

# Install app dependencies
COPY package*.json ./
RUN npm ci --only=production

# Bundle app source
COPY . .

# Create logs directory
RUN mkdir -p logs

# Create non-root user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001

# Change ownership of app directory
RUN chown -R nodejs:nodejs /app
USER nodejs

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3001/health', (res) => process.exit(res.statusCode === 200 ? 0 : 1)).on('error', () => process.exit(1))"

# Start the server
CMD ["npm", "start"]
EOF

cat > .dockerignore << 'EOF'
node_modules
npm-debug.log
logs
.env
.env.local
.env.production
.git
.gitignore
README.md
Dockerfile
.dockerignore
coverage
.nyc_output
test
tests
*.test.js
.vscode
.idea
ssl
EOF

echo "✅ Created Docker configuration files"

# Create useful development scripts
cat > scripts/start-dev.sh << 'EOF'
#!/bin/bash
echo "🚀 Starting WriteMagic AI Proxy in development mode..."
export NODE_ENV=development
export LOG_LEVEL=debug
export ENABLE_REQUEST_LOGGING=true
nodemon server.js
EOF

cat > scripts/test-endpoints.sh << 'EOF'
#!/bin/bash

# Test script for AI proxy endpoints

BASE_URL="http://localhost:3001"

echo "🧪 Testing WriteMagic AI Proxy endpoints..."

# Test health endpoint
echo "Testing /health endpoint..."
curl -s "${BASE_URL}/health" | jq '.' || {
    echo "❌ Health endpoint test failed"
    exit 1
}
echo "✅ Health endpoint OK"

# Test providers endpoint
echo "Testing /api/ai/providers endpoint..."
curl -s "${BASE_URL}/api/ai/providers" | jq '.' || {
    echo "❌ Providers endpoint test failed"
    exit 1
}
echo "✅ Providers endpoint OK"

# Test providers health endpoint
echo "Testing /api/ai/providers/health endpoint..."
curl -s "${BASE_URL}/api/ai/providers/health" | jq '.' || {
    echo "❌ Providers health endpoint test failed"
    exit 1
}
echo "✅ Providers health endpoint OK"

# Test completion endpoint (requires API keys)
echo "Testing /api/ai/complete endpoint..."
RESPONSE=$(curl -s -X POST "${BASE_URL}/api/ai/complete" \
    -H "Content-Type: application/json" \
    -d '{
        "messages": [
            {"role": "user", "content": "Say hello"}
        ],
        "max_tokens": 10
    }')

if echo "$RESPONSE" | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
    echo "✅ Completion endpoint OK"
else
    echo "⚠️  Completion endpoint test failed (likely due to missing API keys):"
    echo "$RESPONSE" | jq '.'
fi

echo "🎉 Endpoint testing complete!"
EOF

chmod +x scripts/start-dev.sh scripts/test-endpoints.sh

echo "✅ Created development scripts"

# Final instructions
echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your API keys:"
echo "   - CLAUDE_API_KEY (get from https://console.anthropic.com/)"
echo "   - OPENAI_API_KEY (get from https://platform.openai.com/)"
echo ""
echo "2. Set CORS origins for your web application:"
echo "   - CORS_ORIGIN=http://localhost:3000"
echo "   - ALLOWED_ORIGINS=http://localhost:3000,https://yourdomain.com"
echo ""
echo "3. Start the development server:"
echo "   npm run dev"
echo "   # OR"
echo "   ./scripts/start-dev.sh"
echo ""
echo "4. Test the endpoints:"
echo "   ./scripts/test-endpoints.sh"
echo ""
echo "5. View logs in the 'logs/' directory"
echo ""
echo "📚 See README.md for full documentation and API reference"
echo ""
echo "🔧 Production deployment options:"
echo "   - PM2: pm2 start ecosystem.config.js"
echo "   - Docker: docker build -t writemagic-ai-proxy ."
echo "   - Direct: npm start"