#!/usr/bin/env bash

# ========================================
# DEPRECATED: SJMC Server Deployment
# ========================================
# This script was used in the original SJMCL project to deploy releases
# to SJMC (Shanghai Jiao Tong Minecraft Club) servers.
# 
# XMCL does not use SJMC deployment infrastructure, so this script
# has been commented out but preserved for reference.
#
# Original functionality: Uploaded release artifacts to SJMC deployment server
# using HMAC-SHA256 authentication.
# ========================================

set -euo pipefail

echo "This script is deprecated and no longer used in XMCL"
echo "SJMC server deployment has been disabled for this fork"
exit 0

# # Required environment variables:
# # - SJMC_SECRET_KEY
# # - SJMC_DEPLOY_API
# # - SJMC_DEPLOY_PROJECT
#
# if [ -z "${SJMC_SECRET_KEY:-}" ]; then
#   echo "❌ SJMC_DEPLOY_SECRET_KEY secret is not set"
#   exit 1
# fi
#
# if [ -z "${SJMC_DEPLOY_API:-}" ]; then
#   echo "❌ SJMC_DEPLOY_API secret is not set"
#   exit 1
# fi
#
# if [ -z "${SJMC_DEPLOY_PROJECT:-}" ]; then
#   echo "❌ SJMC_DEPLOY_PROJECT secret is not set"
#   exit 1
# fi
#
# # Enforce HTTPS and trim whitespace/newlines
# SJMC_DEPLOY_API=$(echo -n "$SJMC_DEPLOY_API" | tr -d '[:space:]')
# if [[ ! "$SJMC_DEPLOY_API" =~ ^https:// ]]; then
#   echo "❌ SJMC_DEPLOY_API must start with https://"
#   exit 1
# fi
#
# echo "✅ All required secrets are set"
#
# cd release-artifacts
# zip -r ../releases.zip *
# cd ..
#
# echo "📦 Created releases.zip with artifacts"
#
# ARTIFACT_HASH=$(sha256sum releases.zip | awk '{ print $1 }')
# DEPLOY_TIMESTAMP=$(date +%s)
#
# # HMAC-SHA256: hash of "<timestamp><artifact_hash>" using SECRET_KEY
# DEPLOY_HASH=$(echo -n "${DEPLOY_TIMESTAMP}${ARTIFACT_HASH}" | openssl dgst -sha256 -hmac "$SJMC_SECRET_KEY" | cut -d' ' -f2)
#
# echo "🔑 Generated deployment hash"
# echo "📅 Timestamp: $DEPLOY_TIMESTAMP"
#
# echo "🚀 Uploading to deployment server..."
# set +e
#
# STATUS_CODE=$(curl --tlsv1.2 --proto '=https' --location -X POST "$SJMC_DEPLOY_API" \
#                 -F "deploy_project=${SJMC_DEPLOY_PROJECT}" \
#                 -F "deploy_timestamp=${DEPLOY_TIMESTAMP}" \
#                 -F "deploy_hash=${DEPLOY_HASH}" \
#                 -F "deploy_artifact=@releases.zip" \
#                 -sS \
#                 -o /dev/null \
#                 -w "%{http_code}")
#
# set -e
#
# if [ "$STATUS_CODE" = "200" ]; then
#   echo "✅ Upload successful!"
# else
#   echo "❌ Upload failed with status code: $STATUS_CODE"
#   exit 1
# fi
