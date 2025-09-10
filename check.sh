#!/bin/bash

# Local development script to run all checks
# This mirrors what the CI pipeline does

set -e

echo "🧪 Running all checks for git-pair..."
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Step 1: Check code formatting
print_step "📝 Checking code formatting with rustfmt..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found"
    echo ""
    print_warning "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi
echo ""

# Step 2: Run clippy linting
print_step "📎 Running clippy linting..."
if cargo clippy -- -D warnings; then
    print_success "No clippy warnings found"
else
    print_error "Clippy warnings found"
    echo ""
    print_warning "Fix the clippy warnings above"
    exit 1
fi
echo ""

# Step 3: Run unit tests
print_step "🧪 Running unit tests..."
if cargo test; then
    print_success "All unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi
echo ""

# Step 4: Build release binary
print_step "🔨 Building release binary..."
if cargo build --release; then
    print_success "Release build successful"
else
    print_error "Release build failed"
    exit 1
fi
echo ""

# Step 5: Run integration tests
print_step "🚀 Running integration tests..."
if [ ! -f "./integration_test.sh" ]; then
    print_error "integration_test.sh not found"
    exit 1
fi

# Make sure integration test script is executable
chmod +x ./integration_test.sh

if ./integration_test.sh; then
    print_success "All integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi
echo ""

# Step 6: Check for common issues
print_step "🔍 Running additional checks..."

# Check for TODO/FIXME comments (optional warning)
TODO_COUNT=$(grep -r "TODO\|FIXME" src/ --exclude-dir=target 2>/dev/null | wc -l || echo "0")
if [ "$TODO_COUNT" -gt 0 ]; then
    print_warning "Found $TODO_COUNT TODO/FIXME comments in source code"
    grep -r "TODO\|FIXME" src/ --exclude-dir=target 2>/dev/null || true
    echo ""
fi

# Check for unwrap() calls (optional warning)
UNWRAP_COUNT=$(grep -r "\.unwrap()" src/ --exclude-dir=target 2>/dev/null | wc -l || echo "0")
if [ "$UNWRAP_COUNT" -gt 0 ]; then
    print_warning "Found $UNWRAP_COUNT .unwrap() calls - consider proper error handling"
fi

print_success "Additional checks completed"
echo ""

# Step 7: Final summary
print_step "📊 Summary"
echo -e "${GREEN}🎉 All checks passed! Your code is ready for commit/push.${NC}"
echo ""
echo "What was checked:"
echo "  ✅ Code formatting (rustfmt)"
echo "  ✅ Linting (clippy)"
echo "  ✅ Unit tests (cargo test)"
echo "  ✅ Release build"
echo "  ✅ Integration tests (14 end-to-end tests)"
echo "  ✅ Code quality checks"
echo ""
echo "Your git-pair is ready for production! 🚀"
