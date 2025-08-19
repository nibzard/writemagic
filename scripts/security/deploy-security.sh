#!/bin/bash
set -euo pipefail

# WriteMagic Production Security Deployment
# This script deploys comprehensive security measures to production environments

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_tools=()
    
    # Required tools
    if ! command -v kubectl >/dev/null 2>&1; then
        missing_tools+=("kubectl")
    fi
    
    if ! command -v docker >/dev/null 2>&1; then
        missing_tools+=("docker")
    fi
    
    if ! command -v trivy >/dev/null 2>&1; then
        missing_tools+=("trivy")
    fi
    
    if ! command -v openssl >/dev/null 2>&1; then
        missing_tools+=("openssl")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Install missing tools and try again"
        exit 1
    fi
    
    log_info "All prerequisites satisfied"
}

# Deploy network policies
deploy_network_policies() {
    local environment=${1:-staging}
    log_info "Deploying network policies for $environment..."
    
    local namespace="writemagic-$environment"
    
    # Replace environment placeholder
    sed "s/writemagic-ENVIRONMENT/$namespace/g" "$PROJECT_ROOT/k8s/network-policies.yaml" > "/tmp/network-policies-$environment.yaml"
    
    # Apply network policies
    kubectl apply -f "/tmp/network-policies-$environment.yaml"
    
    # Verify network policies
    log_info "Verifying network policies..."
    kubectl get networkpolicies -n "$namespace" --no-headers | while read -r policy rest; do
        log_debug "Network policy deployed: $policy"
    done
    
    # Clean up temp file
    rm "/tmp/network-policies-$environment.yaml"
    
    log_info "Network policies deployed successfully"
}

# Deploy secrets securely
deploy_secrets() {
    local environment=${1:-staging}
    log_info "Deploying secrets for $environment..."
    
    local namespace="writemagic-$environment"
    
    # Create namespace if it doesn't exist
    kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -
    
    # Label namespace for network policies
    kubectl label namespace "$namespace" name="$namespace" --overwrite
    
    # Deploy secrets using External Secrets Operator if available
    if kubectl get crd externalsecrets.external-secrets.io >/dev/null 2>&1; then
        log_info "Using External Secrets Operator..."
        sed "s/writemagic-ENVIRONMENT/$namespace/g" "$PROJECT_ROOT/k8s/secrets.yaml" | \
        kubectl apply -f -
    else
        log_warn "External Secrets Operator not found, creating basic secrets"
        "$SCRIPT_DIR/setup-secrets.sh" k8s "$environment"
    fi
}

# Scan container images before deployment
scan_container_images() {
    local environment=${1:-staging}
    log_info "Scanning container images for vulnerabilities..."
    
    local images=(
        "writemagic/app:$environment"
        "postgres:15-alpine"
        "redis:7-alpine"
        "nginx:alpine"
    )
    
    for image in "${images[@]}"; do
        log_info "Scanning $image..."
        
        # Pull latest image
        docker pull "$image" >/dev/null 2>&1 || log_warn "Failed to pull $image"
        
        # Run Trivy scan
        if trivy image --severity HIGH,CRITICAL --no-progress --quiet "$image" > "/tmp/scan-$environment-$(echo "$image" | tr '/:' '-').txt"; then
            local vuln_count
            vuln_count=$(wc -l < "/tmp/scan-$environment-$(echo "$image" | tr '/:' '-').txt")
            
            if [ "$vuln_count" -gt 0 ]; then
                log_warn "$image has $vuln_count HIGH/CRITICAL vulnerabilities"
                cat "/tmp/scan-$environment-$(echo "$image" | tr '/:' '-').txt"
            else
                log_info "$image - No HIGH/CRITICAL vulnerabilities found"
            fi
        else
            log_error "Failed to scan $image"
        fi
    done
}

# Configure ingress security
configure_ingress_security() {
    local environment=${1:-staging}
    log_info "Configuring ingress security for $environment..."
    
    local namespace="writemagic-$environment"
    
    # Create TLS certificates if they don't exist
    if ! kubectl get secret writemagic-tls -n "$namespace" >/dev/null 2>&1; then
        log_info "Creating TLS certificate..."
        
        # Generate self-signed certificate for staging
        if [ "$environment" = "staging" ]; then
            openssl req -x509 -newkey rsa:4096 -keyout /tmp/tls.key -out /tmp/tls.crt \
                -days 365 -nodes -subj "/C=US/ST=Dev/L=Dev/O=WriteMagic/CN=staging.writemagic.com"
            
            kubectl create secret tls writemagic-tls \
                --cert=/tmp/tls.crt \
                --key=/tmp/tls.key \
                -n "$namespace"
            
            rm /tmp/tls.key /tmp/tls.crt
        else
            log_warn "Production TLS certificates should be managed by cert-manager or external provider"
        fi
    fi
    
    # Apply ingress with security annotations
    cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: writemagic-ingress
  namespace: $namespace
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/ssl-protocols: "TLSv1.2 TLSv1.3"
    nginx.ingress.kubernetes.io/ssl-ciphers: "ECDHE-ECDSA-AES128-GCM-SHA256,ECDHE-RSA-AES128-GCM-SHA256,ECDHE-ECDSA-AES256-GCM-SHA384,ECDHE-RSA-AES256-GCM-SHA384"
    nginx.ingress.kubernetes.io/configuration-snippet: |
      add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
      add_header X-Frame-Options "DENY" always;
      add_header X-Content-Type-Options "nosniff" always;
      add_header X-XSS-Protection "1; mode=block" always;
      add_header Referrer-Policy "strict-origin-when-cross-origin" always;
      add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self' https://api.anthropic.com https://api.openai.com; object-src 'none'; frame-src 'none';" always;
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - $environment.writemagic.com
    secretName: writemagic-tls
  rules:
  - host: $environment.writemagic.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: writemagic-backend
            port:
              number: 8080
EOF
    
    log_info "Ingress security configured"
}

# Set up monitoring and alerting
setup_security_monitoring() {
    local environment=${1:-staging}
    log_info "Setting up security monitoring for $environment..."
    
    local namespace="writemagic-$environment"
    
    # Deploy Falco for runtime security monitoring
    if ! kubectl get daemonset falco -n "$namespace" >/dev/null 2>&1; then
        log_info "Deploying Falco for runtime security..."
        
        cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: falco
  namespace: $namespace
  labels:
    app.kubernetes.io/name: falco
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: falco
  template:
    metadata:
      labels:
        app.kubernetes.io/name: falco
    spec:
      serviceAccount: falco
      hostNetwork: true
      hostPID: true
      containers:
      - name: falco
        image: falcosecurity/falco:latest
        securityContext:
          privileged: true
        resources:
          limits:
            memory: 512Mi
            cpu: 200m
          requests:
            memory: 256Mi
            cpu: 100m
        volumeMounts:
        - name: proc
          mountPath: /host/proc
          readOnly: true
        - name: boot
          mountPath: /host/boot
          readOnly: true
        - name: lib-modules
          mountPath: /host/lib/modules
          readOnly: true
        - name: usr
          mountPath: /host/usr
          readOnly: true
      volumes:
      - name: proc
        hostPath:
          path: /proc
      - name: boot
        hostPath:
          path: /boot
      - name: lib-modules
        hostPath:
          path: /lib/modules
      - name: usr
        hostPath:
          path: /usr
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: falco
  namespace: $namespace
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: falco
rules:
- apiGroups: [""]
  resources: ["nodes", "namespaces", "pods", "services", "events"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: falco
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: falco
subjects:
- kind: ServiceAccount
  name: falco
  namespace: $namespace
EOF
    fi
    
    log_info "Security monitoring configured"
}

# Validate security configuration
validate_security() {
    local environment=${1:-staging}
    log_info "Validating security configuration for $environment..."
    
    local namespace="writemagic-$environment"
    
    # Check network policies
    local policy_count
    policy_count=$(kubectl get networkpolicies -n "$namespace" --no-headers | wc -l)
    if [ "$policy_count" -lt 3 ]; then
        log_error "Insufficient network policies deployed ($policy_count found, expected at least 3)"
        return 1
    fi
    log_info "Network policies: ✓ ($policy_count policies active)"
    
    # Check secrets
    if kubectl get secret writemagic-secrets -n "$namespace" >/dev/null 2>&1; then
        log_info "Application secrets: ✓"
    else
        log_error "Application secrets not found"
        return 1
    fi
    
    # Check TLS configuration
    if kubectl get secret writemagic-tls -n "$namespace" >/dev/null 2>&1; then
        log_info "TLS certificates: ✓"
    else
        log_warn "TLS certificates not configured"
    fi
    
    # Check ingress security
    local ingress_annotations
    ingress_annotations=$(kubectl get ingress writemagic-ingress -n "$namespace" -o jsonpath='{.metadata.annotations}' 2>/dev/null || echo "{}")
    if echo "$ingress_annotations" | grep -q "ssl-redirect"; then
        log_info "Ingress security: ✓"
    else
        log_warn "Ingress security annotations not configured"
    fi
    
    # Check pod security
    local non_root_pods
    non_root_pods=$(kubectl get pods -n "$namespace" -o jsonpath='{.items[*].spec.securityContext.runAsNonRoot}' | grep -o true | wc -l)
    local total_pods
    total_pods=$(kubectl get pods -n "$namespace" --no-headers | wc -l)
    
    if [ "$non_root_pods" -gt 0 ]; then
        log_info "Pod security: ✓ ($non_root_pods/$total_pods pods running as non-root)"
    else
        log_warn "No pods configured to run as non-root user"
    fi
    
    log_info "Security validation completed"
}

# Generate security report
generate_security_report() {
    local environment=${1:-staging}
    log_info "Generating security report for $environment..."
    
    local namespace="writemagic-$environment"
    local report_file="/tmp/writemagic-security-report-$environment-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$report_file" <<EOF
# WriteMagic Security Report - $environment

Generated on: $(date -u)
Environment: $environment
Namespace: $namespace

## Network Security

### Network Policies
$(kubectl get networkpolicies -n "$namespace" --no-headers | awk '{print "- " $1}')

### Ingress Configuration
$(kubectl get ingress -n "$namespace" --no-headers | awk '{print "- " $1 " (" $4 ")"}')

## Secret Management

### Secrets
$(kubectl get secrets -n "$namespace" --no-headers | grep -v default-token | awk '{print "- " $1 " (" $2 ")"}')

## Container Security

### Running Pods
$(kubectl get pods -n "$namespace" --no-headers | awk '{print "- " $1 " (" $3 ")"}')

### Security Contexts
$(kubectl get pods -n "$namespace" -o custom-columns=NAME:.metadata.name,NONROOT:.spec.securityContext.runAsNonRoot --no-headers | awk '{print "- " $1 ": runAsNonRoot=" ($2 == "<none>" ? "not_set" : $2)}')

## Monitoring

### Security Monitoring Components
$(kubectl get daemonsets,deployments -n "$namespace" -l 'app.kubernetes.io/name in (falco,prometheus,grafana)' --no-headers | awk '{print "- " $2}')

## Recommendations

1. Regularly update container images to patch security vulnerabilities
2. Monitor Falco alerts for runtime security events
3. Review network policies quarterly to ensure they remain appropriate
4. Rotate secrets according to your security policy
5. Perform regular security assessments and penetration testing

## Compliance Status

- **Network Segmentation**: $([ $(kubectl get networkpolicies -n "$namespace" --no-headers | wc -l) -ge 3 ] && echo "✓ COMPLIANT" || echo "✗ NON-COMPLIANT")
- **Secret Management**: $(kubectl get secret writemagic-secrets -n "$namespace" >/dev/null 2>&1 && echo "✓ COMPLIANT" || echo "✗ NON-COMPLIANT")
- **TLS Encryption**: $(kubectl get secret writemagic-tls -n "$namespace" >/dev/null 2>&1 && echo "✓ COMPLIANT" || echo "✗ NON-COMPLIANT")
- **Container Security**: $([ $(kubectl get pods -n "$namespace" -o jsonpath='{.items[*].spec.securityContext.runAsNonRoot}' | grep -o true | wc -l) -gt 0 ] && echo "✓ COMPLIANT" || echo "⚠ REVIEW NEEDED")

EOF
    
    log_info "Security report generated: $report_file"
    echo "$report_file"
}

# Main deployment function
main() {
    local command=${1:-help}
    local environment=${2:-staging}
    
    case "$command" in
        "full")
            check_prerequisites
            deploy_secrets "$environment"
            deploy_network_policies "$environment"
            scan_container_images "$environment"
            configure_ingress_security "$environment"
            setup_security_monitoring "$environment"
            sleep 30 # Wait for resources to be ready
            validate_security "$environment"
            generate_security_report "$environment"
            ;;
        "secrets")
            check_prerequisites
            deploy_secrets "$environment"
            ;;
        "network")
            check_prerequisites
            deploy_network_policies "$environment"
            ;;
        "ingress")
            check_prerequisites
            configure_ingress_security "$environment"
            ;;
        "scan")
            check_prerequisites
            scan_container_images "$environment"
            ;;
        "monitor")
            check_prerequisites
            setup_security_monitoring "$environment"
            ;;
        "validate")
            check_prerequisites
            validate_security "$environment"
            ;;
        "report")
            check_prerequisites
            generate_security_report "$environment"
            ;;
        "help"|*)
            echo "WriteMagic Security Deployment Tool"
            echo ""
            echo "Usage: $0 <command> [environment]"
            echo ""
            echo "Commands:"
            echo "  full [env]               Deploy complete security stack"
            echo "  secrets [env]            Deploy secret management"
            echo "  network [env]            Deploy network policies"
            echo "  ingress [env]            Configure ingress security"
            echo "  scan [env]               Scan container images"
            echo "  monitor [env]            Setup security monitoring"
            echo "  validate [env]           Validate security configuration"
            echo "  report [env]             Generate security report"
            echo "  help                     Show this help message"
            echo ""
            echo "Environments:"
            echo "  staging                  Staging environment (default)"
            echo "  production               Production environment"
            echo ""
            echo "Examples:"
            echo "  $0 full staging          # Deploy full security stack to staging"
            echo "  $0 secrets production    # Deploy secrets to production"
            echo "  $0 validate staging      # Validate staging security"
            echo "  $0 report production     # Generate production security report"
            ;;
    esac
}

main "$@"