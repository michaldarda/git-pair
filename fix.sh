#!/bin/bash

# Quick fix script - applies automatic fixes
set -e

echo "🔧 Applying automatic fixes..."
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Step 1: Apply rustfmt formatting
print_step "📝 Applying rustfmt formatting..."
cargo fmt
print_success "Code formatting applied"
echo ""

# Step 2: Apply clippy suggestions (where possible)
print_step "📎 Checking for clippy suggestions..."
if cargo clippy --fix --allow-dirty --allow-staged; then
    print_success "Clippy fixes applied (if any)"
else
    echo "Note: Some clippy issues may require manual fixes"
fi
echo ""

print_step "🎯 Summary"
echo -e "${GREEN}🔧 Automatic fixes applied!${NC}"
echo ""
echo "What was fixed:"
echo "  ✅ Code formatting (rustfmt)"
echo "  ✅ Auto-fixable clippy suggestions"
echo ""
echo "Run './check.sh' to verify all checks pass!"
