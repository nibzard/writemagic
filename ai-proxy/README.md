# WriteMagic AI Proxy Service

A secure, production-ready AI API proxy service that handles client-side AI requests with provider fallback, rate limiting, and comprehensive security features.

## Features

### Security
- **API Key Protection**: All AI provider keys are stored server-side only
- **CORS Protection**: Configurable origin validation with security headers
- **Rate Limiting**: Per-IP rate limiting to prevent abuse
- **Input Validation**: Comprehensive request validation and sanitization
- **Content Filtering**: PII detection and blocked pattern filtering
- **Request Sanitization**: XSS and injection attack prevention

### AI Integration
- **Provider Fallback**: Automatic failover between Claude, GPT-4, and local models
- **Response Caching**: Intelligent caching with TTL and memory management
- **Usage Monitoring**: Token counting, cost tracking, and performance metrics
- **Health Checks**: Real-time provider availability monitoring

### Performance
- **Async Processing**: Non-blocking request handling
- **Connection Pooling**: Optimized HTTP client connections
- **Response Streaming**: Support for real-time AI responses
- **Memory Management**: Automatic cache cleanup and memory monitoring

## Quick Start

### 1. Installation

```bash
cd ai-proxy
npm install
```

### 2. Configuration

Copy the environment template and configure your API keys:

```bash
cp .env.example .env
# Edit .env with your configuration
```

Required environment variables:
```env
# At least one AI provider API key is required
CLAUDE_API_KEY=your_claude_api_key_here
OPENAI_API_KEY=your_openai_api_key_here

# CORS configuration for your web app
CORS_ORIGIN=http://localhost:3000
ALLOWED_ORIGINS=http://localhost:3000,https://yourdomain.com
```

### 3. Start the Server

**Development:**
```bash
npm run dev
```

**Production:**
```bash
npm start
```

The server will start on `http://localhost:3001` by default.

## API Endpoints

### Core AI Endpoints

#### `POST /api/ai/complete`
Text completion with provider fallback.

**Request:**
```json
{
  "messages": [
    {"role": "user", "content": "Write a short story about AI"}
  ],
  "model": "claude-3-sonnet-20240229",
  "max_tokens": 1000,
  "temperature": 0.7
}
```

**Response:**
```json
{
  "id": "completion_id",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Generated story content..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 150,
    "total_tokens": 162
  },
  "metadata": {
    "requestId": "req_123",
    "processingTime": 1250,
    "cached": false,
    "provider": "claude"
  }
}
```

#### `POST /api/ai/chat`
Conversational AI with context management.

**Request:**
```json
{
  "message": "How can I improve my writing?",
  "conversation_id": "conv_123",
  "context": [
    {"role": "user", "content": "I'm working on a novel"},
    {"role": "assistant", "content": "That's exciting! What genre?"}
  ]
}
```

**Response:**
```json
{
  "message": "Here are some tips to improve your writing...",
  "conversationId": "conv_123",
  "messageId": "msg_456",
  "provider": "claude",
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 120,
    "total_tokens": 145
  },
  "metadata": {
    "requestId": "req_789",
    "processingTime": 800,
    "cached": false
  }
}
```

### Monitoring Endpoints

#### `GET /api/ai/providers/health`
Check the health status of all AI providers.

#### `GET /api/ai/providers`
List available providers and their configurations.

#### `GET /api/ai/cache/stats`
Get response cache statistics and performance metrics.

#### `POST /api/ai/cache/clear`
Clear the response cache (useful for testing).

#### `GET /health`
Service health check endpoint.

## Security Features

### API Key Management
- All API keys are stored server-side in environment variables
- Keys are never exposed to client applications
- Automatic key validation on startup

### Content Security
- **PII Detection**: Automatic detection of email addresses, phone numbers, SSNs
- **Content Filtering**: Configurable blocked patterns and content limits
- **Input Sanitization**: XSS and injection attack prevention
- **Request Size Limits**: Configurable maximum request sizes

### Access Control
- **CORS Validation**: Strict origin validation with allowlist
- **Rate Limiting**: Per-IP request limits with sliding window
- **Request Validation**: Comprehensive input validation using Joi schemas
- **Security Headers**: Helmet.js security headers with CSP

### Monitoring & Logging
- **Request Logging**: Detailed request/response logging with sensitive data filtering
- **Security Events**: Automatic logging of security violations and anomalies
- **Performance Metrics**: Response times, token usage, and provider statistics
- **Error Tracking**: Comprehensive error logging with context

## Provider Configuration

### Claude (Anthropic)
```env
CLAUDE_API_KEY=your_api_key
CLAUDE_BASE_URL=https://api.anthropic.com  # Optional
CLAUDE_MAX_TOKENS=100000
CLAUDE_TEMPERATURE=0.7
```

### OpenAI GPT
```env
OPENAI_API_KEY=your_api_key
OPENAI_BASE_URL=https://api.openai.com  # Optional
OPENAI_MAX_TOKENS=4096
OPENAI_TEMPERATURE=0.7
```

### Fallback Strategy
The service automatically falls back between providers:
1. **Primary**: Claude (default)
2. **Secondary**: OpenAI GPT-4
3. **Tertiary**: Local models (when configured)

## Rate Limiting

Default rate limits (configurable):
- **Window**: 15 minutes
- **Max Requests**: 100 per IP
- **Burst Protection**: Sliding window algorithm

```env
RATE_LIMIT_WINDOW_MS=900000    # 15 minutes
RATE_LIMIT_MAX_REQUESTS=100    # Per IP per window
```

## Caching

Response caching with intelligent key generation:
- **TTL**: 5 minutes (configurable)
- **Max Items**: 1000 cached responses
- **Cache Keys**: Generated from request content hash
- **Memory Management**: Automatic cleanup of expired entries

```env
RESPONSE_CACHE_TTL_SECONDS=300   # 5 minutes
RESPONSE_CACHE_MAX_ITEMS=1000
```

## Integration with Web App

### JavaScript Client Example

```javascript
class WriteMagicAI {
  constructor(proxyUrl = 'http://localhost:3001') {
    this.baseUrl = proxyUrl;
  }

  async complete(messages, options = {}) {
    const response = await fetch(`${this.baseUrl}/api/ai/complete`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Request-ID': this.generateRequestId(),
      },
      credentials: 'include',
      body: JSON.stringify({
        messages,
        ...options,
      }),
    });

    if (!response.ok) {
      throw new Error(`AI request failed: ${response.statusText}`);
    }

    return response.json();
  }

  async chat(message, conversationId = null, context = []) {
    const response = await fetch(`${this.baseUrl}/api/ai/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({
        message,
        conversation_id: conversationId,
        context,
      }),
    });

    return response.json();
  }

  generateRequestId() {
    return `client_${Date.now()}_${Math.random().toString(36).substring(2)}`;
  }
}

// Usage
const ai = new WriteMagicAI();

// Text completion
const result = await ai.complete([
  { role: 'user', content: 'Help me write better' }
], {
  max_tokens: 500,
  temperature: 0.7
});

console.log(result.choices[0].message.content);
```

### Error Handling

```javascript
try {
  const result = await ai.complete(messages);
  // Handle success
} catch (error) {
  if (error.status === 429) {
    // Rate limited - wait and retry
    await new Promise(resolve => setTimeout(resolve, 5000));
    // Retry logic here
  } else if (error.status === 502) {
    // Provider authentication error
    console.error('AI service configuration error');
  } else {
    // Other errors
    console.error('AI request failed:', error.message);
  }
}
```

## Deployment

### Environment Setup

1. **Production Environment Variables**:
```env
NODE_ENV=production
PORT=3001
HOST=0.0.0.0

# Security
CORS_ORIGIN=https://yourdomain.com
ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com

# Monitoring
ENABLE_REQUEST_LOGGING=true
LOG_LEVEL=info
```

2. **SSL/TLS**: Use a reverse proxy (nginx, Cloudflare) for HTTPS termination

3. **Process Management**: Use PM2 or similar for process management:
```bash
npm install -g pm2
pm2 start server.js --name "writemagic-ai-proxy"
```

### Docker Deployment

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .

EXPOSE 3001

USER node

CMD ["npm", "start"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: writemagic-ai-proxy
spec:
  replicas: 3
  selector:
    matchLabels:
      app: writemagic-ai-proxy
  template:
    metadata:
      labels:
        app: writemagic-ai-proxy
    spec:
      containers:
      - name: ai-proxy
        image: writemagic/ai-proxy:latest
        ports:
        - containerPort: 3001
        env:
        - name: NODE_ENV
          value: "production"
        - name: CLAUDE_API_KEY
          valueFrom:
            secretKeyRef:
              name: ai-secrets
              key: claude-api-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## Performance Tuning

### Memory Management
- Monitor cache memory usage
- Adjust `RESPONSE_CACHE_MAX_ITEMS` based on available memory
- Use `--max-old-space-size` for Node.js heap limit

### Connection Pooling
- Default HTTP agent pools connections automatically
- Increase `maxSockets` for high-traffic scenarios

### Rate Limiting Optimization
- Adjust rate limits based on your AI provider quotas
- Consider provider-specific rate limiting

## Security Best Practices

1. **API Keys**: Use environment variables, never commit keys to version control
2. **CORS**: Whitelist only your application domains
3. **Rate Limiting**: Set appropriate limits for your use case
4. **Monitoring**: Enable request logging and monitor for anomalies
5. **Updates**: Keep dependencies updated for security patches
6. **Network**: Use HTTPS in production, consider VPC/private networks

## Monitoring & Alerting

### Health Checks
The service provides comprehensive health endpoints for monitoring:

- `/health` - Basic service health
- `/api/ai/providers/health` - AI provider availability
- `/api/ai/cache/stats` - Performance metrics

### Logging
Structured JSON logs are generated for:
- All HTTP requests/responses
- AI provider requests
- Security events
- Performance metrics
- Error conditions

### Metrics Integration
Compatible with standard monitoring tools:
- Prometheus metrics endpoint (can be added)
- Winston logging compatible with ELK stack
- Health endpoints for Kubernetes probes

## Troubleshooting

### Common Issues

1. **CORS Errors**:
   - Check `ALLOWED_ORIGINS` configuration
   - Verify origin spelling and protocol (http/https)

2. **Rate Limiting**:
   - Adjust `RATE_LIMIT_MAX_REQUESTS` if needed
   - Check if legitimate traffic is being blocked

3. **Provider Errors**:
   - Verify API keys are correct and have sufficient quota
   - Check provider status pages for outages

4. **Cache Issues**:
   - Clear cache using `/api/ai/cache/clear`
   - Adjust TTL if responses are too stale

### Debug Mode

Enable detailed logging:
```env
NODE_ENV=development
LOG_LEVEL=debug
ENABLE_REQUEST_LOGGING=true
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.