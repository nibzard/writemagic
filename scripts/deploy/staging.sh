#!/bin/bash
set -euo pipefail

# WriteMagic Staging Deployment Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[STAGING]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
STAGING_ENV="${STAGING_ENV:-staging}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-ghcr.io/writemagic}"
VERSION="${VERSION:-latest}"
NAMESPACE="${NAMESPACE:-writemagic-staging}"

print_status "Starting WriteMagic staging deployment..."
print_status "Environment: $STAGING_ENV"
print_status "Version: $VERSION"
print_status "Namespace: $NAMESPACE"

# Pre-deployment checks
print_status "Running pre-deployment checks..."

# Check if kubectl is available
if ! command -v kubectl >/dev/null 2>&1; then
    print_error "kubectl not found. Please install kubectl."
    exit 1
fi

# Check if docker is available
if ! command -v docker >/dev/null 2>&1; then
    print_error "docker not found. Please install Docker."
    exit 1
fi

# Check cluster connectivity
if ! kubectl cluster-info >/dev/null 2>&1; then
    print_error "Cannot connect to Kubernetes cluster"
    exit 1
fi

print_success "Pre-deployment checks passed"

# Create namespace if it doesn't exist
print_status "Ensuring namespace exists..."
kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
print_success "Namespace $NAMESPACE ready"

# Deploy PostgreSQL (staging)
print_status "Deploying PostgreSQL for staging..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: $NAMESPACE
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        env:
        - name: POSTGRES_DB
          value: writemagic_staging
        - name: POSTGRES_USER
          value: writemagic
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: $NAMESPACE
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: $NAMESPACE
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
EOF

# Deploy Redis (staging)
print_status "Deploying Redis for staging..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: $NAMESPACE
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        command: ["redis-server", "--appendonly", "yes"]
        ports:
        - containerPort: 6379
        volumeMounts:
        - name: redis-storage
          mountPath: /data
      volumes:
      - name: redis-storage
        persistentVolumeClaim:
          claimName: redis-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: $NAMESPACE
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: redis-pvc
  namespace: $NAMESPACE
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 5Gi
EOF

# Deploy WriteMagic application
print_status "Deploying WriteMagic application..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: writemagic-app
  namespace: $NAMESPACE
  labels:
    app: writemagic-app
    version: $VERSION
spec:
  replicas: 2
  selector:
    matchLabels:
      app: writemagic-app
  template:
    metadata:
      labels:
        app: writemagic-app
        version: $VERSION
    spec:
      containers:
      - name: writemagic
        image: $DOCKER_REGISTRY/writemagic:$VERSION
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: info
        - name: WRITEMAGIC_ENV
          value: staging
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: app-secrets
              key: database-url
        - name: REDIS_URL
          value: redis://redis:6379
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: 256Mi
            cpu: 250m
          limits:
            memory: 512Mi
            cpu: 500m
---
apiVersion: v1
kind: Service
metadata:
  name: writemagic-app
  namespace: $NAMESPACE
spec:
  selector:
    app: writemagic-app
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
EOF

# Deploy ingress
print_status "Deploying ingress..."
cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: writemagic-ingress
  namespace: $NAMESPACE
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-staging
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - staging.writemagic.dev
    secretName: writemagic-staging-tls
  rules:
  - host: staging.writemagic.dev
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: writemagic-app
            port:
              number: 80
EOF

# Deploy monitoring (Prometheus)
print_status "Deploying monitoring stack..."
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prometheus
  namespace: $NAMESPACE
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prometheus
  template:
    metadata:
      labels:
        app: prometheus
    spec:
      containers:
      - name: prometheus
        image: prom/prometheus:latest
        ports:
        - containerPort: 9090
        volumeMounts:
        - name: prometheus-config
          mountPath: /etc/prometheus
        - name: prometheus-storage
          mountPath: /prometheus
      volumes:
      - name: prometheus-config
        configMap:
          name: prometheus-config
      - name: prometheus-storage
        persistentVolumeClaim:
          claimName: prometheus-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: prometheus
  namespace: $NAMESPACE
spec:
  selector:
    app: prometheus
  ports:
  - port: 9090
    targetPort: 9090
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: prometheus-pvc
  namespace: $NAMESPACE
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
EOF

# Wait for deployments to be ready
print_status "Waiting for deployments to be ready..."

deployments=("postgres" "redis" "writemagic-app" "prometheus")
for deployment in "${deployments[@]}"; do
    print_status "Waiting for $deployment..."
    kubectl wait --for=condition=available --timeout=300s deployment/$deployment -n $NAMESPACE
    print_success "$deployment is ready"
done

# Run post-deployment tests
print_status "Running post-deployment tests..."

# Check if services are accessible
print_status "Testing service connectivity..."

# Test database connectivity
kubectl run test-postgres --rm -i --restart=Never --image=postgres:15-alpine -n $NAMESPACE -- psql postgresql://writemagic:$(kubectl get secret postgres-secret -n $NAMESPACE -o jsonpath='{.data.password}' | base64 -d)@postgres:5432/writemagic_staging -c "SELECT version();"

# Test Redis connectivity  
kubectl run test-redis --rm -i --restart=Never --image=redis:7-alpine -n $NAMESPACE -- redis-cli -h redis ping

# Test application health
print_status "Testing application health..."
APP_POD=$(kubectl get pods -l app=writemagic-app -n $NAMESPACE -o jsonpath='{.items[0].metadata.name}')
kubectl exec $APP_POD -n $NAMESPACE -- curl -f http://localhost:8080/health

print_success "Post-deployment tests passed"

# Display deployment information
print_status "Deployment summary:"
echo "----------------------------------------"
echo "Environment: $STAGING_ENV"
echo "Version: $VERSION"
echo "Namespace: $NAMESPACE"
echo "External URL: https://staging.writemagic.dev"
echo "Prometheus: http://prometheus.$NAMESPACE.svc.cluster.local:9090"
echo "----------------------------------------"

kubectl get pods -n $NAMESPACE
kubectl get services -n $NAMESPACE
kubectl get ingress -n $NAMESPACE

print_success "WriteMagic staging deployment completed successfully!"
print_status "Application should be available at: https://staging.writemagic.dev"