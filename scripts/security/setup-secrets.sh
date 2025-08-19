#!/bin/bash
set -euo pipefail

# WriteMagic Security Setup - External Secret Management
# This script sets up secure secret management for development and production environments

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Generate secure random password
generate_password() {
    local length=${1:-32}
    openssl rand -base64 "$length" | tr -d "=+/" | cut -c1-"$length"
}

# Create Docker secrets for development
setup_docker_secrets() {
    log_info "Setting up Docker secrets for development environment..."
    
    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    
    # Initialize Docker swarm if not already done
    if ! docker info --format '{{.Swarm.LocalNodeState}}' | grep -q active; then
        log_info "Initializing Docker swarm for secrets management..."
        docker swarm init --advertise-addr 127.0.0.1
    fi
    
    # Generate and create secrets
    local secrets=(
        "writemagic_postgres_password"
        "writemagic_minio_password"
        "writemagic_grafana_admin_password"
        "writemagic_grafana_secret_key"
    )
    
    for secret_name in "${secrets[@]}"; do
        if docker secret ls --format '{{.Name}}' | grep -q "^$secret_name$"; then
            log_warn "Secret $secret_name already exists, skipping..."
        else
            local password
            password=$(generate_password)
            echo "$password" | docker secret create "$secret_name" -
            log_info "Created secret: $secret_name"
        fi
    done
}

# Setup Kubernetes secrets (for production)
setup_k8s_secrets() {
    local environment=${1:-staging}
    log_info "Setting up Kubernetes secrets for $environment environment..."
    
    # Check if kubectl is available
    if ! command -v kubectl >/dev/null 2>&1; then
        log_error "kubectl is not installed. Please install kubectl and try again."
        exit 1
    fi
    
    # Create namespace if it doesn't exist
    local namespace="writemagic-$environment"
    kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -
    
    # Generate secrets
    local db_password minio_password jwt_secret encryption_key session_secret cookie_secret
    db_password=$(generate_password 32)
    minio_password=$(generate_password 32)
    jwt_secret=$(generate_password 64)
    encryption_key=$(generate_password 64)
    session_secret=$(generate_password 64)
    cookie_secret=$(generate_password 64)
    
    # Create main secrets
    kubectl create secret generic writemagic-secrets \
        --namespace="$namespace" \
        --from-literal=database-password="$db_password" \
        --from-literal=jwt-secret="$jwt_secret" \
        --from-literal=encryption-key="$encryption_key" \
        --from-literal=session-secret="$session_secret" \
        --from-literal=cookie-secret="$cookie_secret" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create MinIO secrets
    kubectl create secret generic writemagic-storage-secrets \
        --namespace="$namespace" \
        --from-literal=minio-password="$minio_password" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    log_info "Kubernetes secrets created in namespace: $namespace"
    log_warn "IMPORTANT: Store these credentials in a secure password manager:"
    log_warn "Database password: $db_password"
    log_warn "MinIO password: $minio_password"
}

# Setup AWS Secrets Manager (for production)
setup_aws_secrets() {
    local environment=${1:-staging}
    log_info "Setting up AWS Secrets Manager for $environment environment..."
    
    # Check if AWS CLI is available
    if ! command -v aws >/dev/null 2>&1; then
        log_error "AWS CLI is not installed. Please install AWS CLI and try again."
        exit 1
    fi
    
    # Check AWS credentials
    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        log_error "AWS credentials not configured. Please run 'aws configure' first."
        exit 1
    fi
    
    local secret_prefix="writemagic/$environment"
    
    # Database secrets
    local db_password jwt_secret encryption_key
    db_password=$(generate_password 32)
    jwt_secret=$(generate_password 64)
    encryption_key=$(generate_password 64)
    
    # Create database secret
    aws secretsmanager create-secret \
        --name "$secret_prefix/database" \
        --description "WriteMagic $environment database credentials" \
        --secret-string "{\"username\":\"writemagic_user\",\"password\":\"$db_password\",\"host\":\"writemagic-$environment-db.cluster-xyz.us-west-2.rds.amazonaws.com\",\"port\":5432,\"dbname\":\"writemagic\"}" \
        >/dev/null 2>&1 || log_warn "Database secret may already exist"
    
    # Create application secrets
    aws secretsmanager create-secret \
        --name "$secret_prefix/application" \
        --description "WriteMagic $environment application secrets" \
        --secret-string "{\"jwt_secret\":\"$jwt_secret\",\"encryption_key\":\"$encryption_key\"}" \
        >/dev/null 2>&1 || log_warn "Application secret may already exist"
    
    # Create AI provider secrets (placeholder)
    aws secretsmanager create-secret \
        --name "$secret_prefix/ai-providers" \
        --description "WriteMagic $environment AI provider keys" \
        --secret-string "{\"claude_api_key\":\"PLACEHOLDER\",\"openai_api_key\":\"PLACEHOLDER\"}" \
        >/dev/null 2>&1 || log_warn "AI provider secret may already exist"
    
    log_info "AWS Secrets Manager secrets created for $environment"
    log_warn "IMPORTANT: Update AI provider keys manually in AWS Console"
}

# Rotate secrets
rotate_secrets() {
    local environment=${1:-development}
    log_info "Rotating secrets for $environment environment..."
    
    case "$environment" in
        "development")
            log_info "Rotating Docker secrets..."
            # This would require removing and recreating secrets
            log_warn "Manual rotation required for Docker secrets in development"
            ;;
        "staging"|"production")
            log_info "Rotating AWS secrets..."
            # This would update the secret values in AWS Secrets Manager
            log_warn "Use AWS Console or CLI to rotate production secrets"
            ;;
        *)
            log_error "Unknown environment: $environment"
            exit 1
            ;;
    esac
}

# Audit secrets
audit_secrets() {
    log_info "Auditing secret configuration..."
    
    # Check for hardcoded secrets in code
    log_info "Checking for hardcoded secrets in codebase..."
    if grep -r -i "password\|secret\|key\|token" --include="*.rs" --include="*.kt" --include="*.swift" "$PROJECT_ROOT" | grep -v "test" | grep -v "example" | head -5; then
        log_warn "Found potential hardcoded secrets. Review manually."
    else
        log_info "No obvious hardcoded secrets found in source code."
    fi
    
    # Check Docker secrets
    if docker info --format '{{.Swarm.LocalNodeState}}' | grep -q active; then
        log_info "Docker secrets:"
        docker secret ls
    fi
    
    # Check environment variables
    log_info "Checking environment variables for secrets..."
    env | grep -i "password\|secret\|key\|token" || log_info "No secret environment variables found."
}

# Main function
main() {
    local command=${1:-help}
    local environment=${2:-development}
    
    case "$command" in
        "docker")
            setup_docker_secrets
            ;;
        "k8s"|"kubernetes")
            setup_k8s_secrets "$environment"
            ;;
        "aws")
            setup_aws_secrets "$environment"
            ;;
        "rotate")
            rotate_secrets "$environment"
            ;;
        "audit")
            audit_secrets
            ;;
        "help"|*)
            echo "WriteMagic Security Setup"
            echo ""
            echo "Usage: $0 <command> [environment]"
            echo ""
            echo "Commands:"
            echo "  docker                    Setup Docker secrets for development"
            echo "  k8s [env]                Setup Kubernetes secrets"
            echo "  aws [env]                Setup AWS Secrets Manager"
            echo "  rotate [env]             Rotate secrets for environment"
            echo "  audit                    Audit current secret configuration"
            echo "  help                     Show this help message"
            echo ""
            echo "Environments:"
            echo "  development              Local development (default)"
            echo "  staging                  Staging environment"
            echo "  production               Production environment"
            echo ""
            echo "Examples:"
            echo "  $0 docker                # Setup development secrets"
            echo "  $0 k8s staging          # Setup staging Kubernetes secrets"
            echo "  $0 aws production       # Setup production AWS secrets"
            echo "  $0 audit                # Audit all secret configurations"
            ;;
    esac
}

main "$@"