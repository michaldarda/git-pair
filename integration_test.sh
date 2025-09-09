#!/bin/bash

# Integration test script for git-pair
set -e

echo "🧪 Running git-pair integration tests..."

# Build the project
echo "📦 Building git-pair..."
cargo build --release

# Create a temporary directory for testing
TEST_DIR=$(mktemp -d)
cd "$TEST_DIR"

echo "📁 Created test directory: $TEST_DIR"

# Initialize a git repo
git init
git config user.name "Test User"
git config user.email "test@example.com"

# Copy the git-pair binary
cp "$OLDPWD/target/release/git-pair" .

echo "✅ Test 1: Initialize git-pair"
./git-pair init
if [ ! -f ".git/git-pair/config" ]; then
    echo "❌ Config file not created"
    exit 1
fi

echo "✅ Test 2: Add co-authors"
./git-pair add John Doe john.doe@company.com
./git-pair add Jane Smith jane.smith@company.com

echo "✅ Test 3: Check status"
STATUS_OUTPUT=$(./git-pair status)
if [[ ! "$STATUS_OUTPUT" == *"John Doe"* ]] || [[ ! "$STATUS_OUTPUT" == *"Jane Smith"* ]]; then
    echo "❌ Status doesn't show co-authors correctly"
    echo "Output: $STATUS_OUTPUT"
    exit 1
fi

echo "✅ Test 4: Check commit template"
if [ ! -f ".git/git-pair/commit-template" ]; then
    echo "❌ Commit template not created"
    exit 1
fi

TEMPLATE_CONTENT=$(cat .git/git-pair/commit-template)
if [[ ! "$TEMPLATE_CONTENT" == *"John Doe"* ]] || [[ ! "$TEMPLATE_CONTENT" == *"Jane Smith"* ]]; then
    echo "❌ Commit template doesn't contain co-authors"
    echo "Template: $TEMPLATE_CONTENT"
    exit 1
fi

echo "✅ Test 5: Check git config"
GIT_TEMPLATE=$(git config commit.template)
if [[ ! "$GIT_TEMPLATE" == *".git/git-pair/commit-template"* ]]; then
    echo "❌ Git commit template not configured"
    echo "Git config: $GIT_TEMPLATE"
    exit 1
fi

echo "✅ Test 6: Clear co-authors"
./git-pair clear
CLEAR_STATUS=$(./git-pair status)
if [[ ! "$CLEAR_STATUS" == *"No co-authors configured"* ]]; then
    echo "❌ Co-authors not cleared properly"
    echo "Status after clear: $CLEAR_STATUS"
    exit 1
fi

echo "✅ Test 7: Check git config unset"
if git config commit.template 2>/dev/null; then
    echo "❌ Git commit template not unset after clear"
    exit 1
fi

echo "✅ Test 8: Error handling - not initialized"
rm -rf .git/git-pair
ERROR_OUTPUT=$(./git-pair add Test User test@example.com 2>&1 || true)
if [[ ! "$ERROR_OUTPUT" == *"not initialized"* ]]; then
    echo "❌ Error handling for uninitialized state failed"
    echo "Error output: $ERROR_OUTPUT"
    exit 1
fi

echo "✅ Test 9: Error handling - not in git repo"
cd /tmp
ERROR_OUTPUT=$("$TEST_DIR/git-pair" init 2>&1 || true)
if [[ ! "$ERROR_OUTPUT" == *"Not in a git repository"* ]]; then
    echo "❌ Error handling for non-git directory failed"
    echo "Error output: $ERROR_OUTPUT"
    exit 1
fi

# Cleanup
cd "$OLDPWD"
rm -rf "$TEST_DIR"

echo "🎉 All integration tests passed!"
echo "🚀 git-pair is ready for use!"
