output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "cluster_security_group_id" {
  description = "Security group ID attached to the EKS cluster"
  value       = module.eks.cluster_security_group_id
}

output "cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data required to communicate with the cluster"
  value       = module.eks.cluster_certificate_authority_data
  sensitive   = true
}

output "cluster_oidc_issuer_url" {
  description = "The URL on the EKS cluster for the OpenID Connect identity provider"
  value       = module.eks.cluster_oidc_issuer_url
}

output "cluster_primary_security_group_id" {
  description = "The cluster primary security group ID created by EKS"
  value       = module.eks.cluster_primary_security_group_id
}

output "vpc_id" {
  description = "ID of the VPC where the cluster is deployed"
  value       = module.vpc.vpc_id
}

output "vpc_cidr_block" {
  description = "The CIDR block of the VPC"
  value       = module.vpc.vpc_cidr_block
}

output "private_subnets" {
  description = "List of IDs of private subnets"
  value       = module.vpc.private_subnets
}

output "public_subnets" {
  description = "List of IDs of public subnets"
  value       = module.vpc.public_subnets
}

output "database_endpoint" {
  description = "RDS instance endpoint"
  value       = aws_db_instance.writemagic.endpoint
  sensitive   = true
}

output "database_port" {
  description = "RDS instance port"
  value       = aws_db_instance.writemagic.port
}

output "database_name" {
  description = "Name of the database"
  value       = aws_db_instance.writemagic.db_name
}

output "database_username" {
  description = "RDS instance root username"
  value       = aws_db_instance.writemagic.username
  sensitive   = true
}

output "redis_primary_endpoint" {
  description = "Redis primary endpoint"
  value       = aws_elasticache_replication_group.writemagic.primary_endpoint_address
  sensitive   = true
}

output "redis_reader_endpoint" {
  description = "Redis reader endpoint"
  value       = aws_elasticache_replication_group.writemagic.reader_endpoint_address
  sensitive   = true
}

output "redis_port" {
  description = "Redis port"
  value       = aws_elasticache_replication_group.writemagic.port
}

output "s3_bucket_name" {
  description = "Name of the S3 bucket for assets"
  value       = aws_s3_bucket.writemagic_assets.bucket
}

output "s3_bucket_arn" {
  description = "ARN of the S3 bucket for assets"
  value       = aws_s3_bucket.writemagic_assets.arn
}

output "cloudwatch_log_group_name" {
  description = "Name of the CloudWatch log group"
  value       = aws_cloudwatch_log_group.writemagic.name
}

# KMS Key outputs for application use
output "eks_kms_key_id" {
  description = "KMS key ID for EKS encryption"
  value       = aws_kms_key.eks.key_id
}

output "rds_kms_key_id" {
  description = "KMS key ID for RDS encryption"
  value       = aws_kms_key.rds.key_id
}

output "s3_kms_key_id" {
  description = "KMS key ID for S3 encryption"
  value       = aws_kms_key.s3.key_id
}

# Security group outputs for application configuration
output "rds_security_group_id" {
  description = "Security group ID for RDS"
  value       = aws_security_group.rds.id
}

output "elasticache_security_group_id" {
  description = "Security group ID for ElastiCache"
  value       = aws_security_group.elasticache.id
}

# Environment and resource identification
output "environment" {
  description = "Environment name"
  value       = var.environment
}

output "aws_region" {
  description = "AWS region"
  value       = var.aws_region
}

output "common_tags" {
  description = "Common tags applied to all resources"
  value       = local.common_tags
}

# Configuration values for Kubernetes deployments
output "kubernetes_config" {
  description = "Configuration values for Kubernetes deployments"
  value = {
    cluster_name      = module.eks.cluster_name
    cluster_endpoint  = module.eks.cluster_endpoint
    vpc_id           = module.vpc.vpc_id
    private_subnets  = module.vpc.private_subnets
    environment      = var.environment
    aws_region       = var.aws_region
  }
  sensitive = true
}

# Application configuration
output "application_config" {
  description = "Configuration values for the WriteMagic application"
  value = {
    database = {
      host     = aws_db_instance.writemagic.endpoint
      port     = aws_db_instance.writemagic.port
      name     = aws_db_instance.writemagic.db_name
      username = aws_db_instance.writemagic.username
    }
    redis = {
      primary_endpoint = aws_elasticache_replication_group.writemagic.primary_endpoint_address
      reader_endpoint  = aws_elasticache_replication_group.writemagic.reader_endpoint_address
      port            = aws_elasticache_replication_group.writemagic.port
    }
    storage = {
      bucket_name = aws_s3_bucket.writemagic_assets.bucket
      bucket_arn  = aws_s3_bucket.writemagic_assets.arn
    }
    logging = {
      log_group = aws_cloudwatch_log_group.writemagic.name
    }
    encryption = {
      eks_key = aws_kms_key.eks.key_id
      rds_key = aws_kms_key.rds.key_id
      s3_key  = aws_kms_key.s3.key_id
    }
  }
  sensitive = true
}