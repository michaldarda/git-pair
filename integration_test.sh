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
# Get the current branch name and check for branch-specific config file
CURRENT_BRANCH=$(git branch --show-current)
if [ ! -f ".git/git-pair/config-$CURRENT_BRANCH" ]; then
    echo "❌ Branch-specific config file not created"
    echo "Expected: .git/git-pair/config-$CURRENT_BRANCH"
    ls -la .git/git-pair/ || echo "git-pair directory doesn't exist"
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

echo "✅ Test 4: Check git hook"
if [ ! -f ".git/hooks/prepare-commit-msg" ]; then
    echo "❌ Git hook not created"
    exit 1
fi

HOOK_CONTENT=$(cat .git/hooks/prepare-commit-msg)
# With per-branch config, the hook reads co-authors dynamically from config files
# Check that the hook contains the dynamic logic instead of hard-coded names
if [[ ! "$HOOK_CONTENT" == *"CURRENT_BRANCH"* ]] || [[ ! "$HOOK_CONTENT" == *"CONFIG_FILE"* ]]; then
    echo "❌ Git hook doesn't contain per-branch logic"
    echo "Hook: $HOOK_CONTENT"
    exit 1
fi

# Check that the branch-specific config file contains the co-authors
CONFIG_CONTENT=$(cat ".git/git-pair/config-$CURRENT_BRANCH")
if [[ ! "$CONFIG_CONTENT" == *"John Doe"* ]] || [[ ! "$CONFIG_CONTENT" == *"Jane Smith"* ]]; then
    echo "❌ Branch config doesn't contain co-authors"
    echo "Config: $CONFIG_CONTENT"
    exit 1
fi

echo "✅ Test 5: Check no git commit template"
if git config commit.template 2>/dev/null; then
    echo "❌ Git commit template should not be configured in simplified version"
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

echo "✅ Test 7: Check git hook removed"
if [ -f ".git/hooks/prepare-commit-msg" ]; then
    echo "❌ Git hook not removed after clear"
    exit 1
fi

echo "✅ Test 8: Error handling - not initialized"
rm -rf .git/git-pair
ERROR_OUTPUT=$(./git-pair add Test User test@example.com 2>&1 || true)
if [[ ! "$ERROR_OUTPUT" == *"not initialized for branch"* ]]; then
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

# Go back to test directory for global roster tests
cd "$TEST_DIR"

echo "✅ Test 10: Global roster - add entries"
# Use a temporary roster file for testing to avoid polluting user's global config
TEMP_ROSTER_FILE="$TEST_DIR/test-roster"
export GIT_PAIR_ROSTER_FILE="$TEMP_ROSTER_FILE"

./git-pair add --global alice "Alice Johnson" alice@company.com
./git-pair add --global bob "Bob Wilson" bob@company.com

echo "✅ Test 11: Global roster - list entries"
ROSTER_OUTPUT=$(./git-pair list --global)
if [[ ! "$ROSTER_OUTPUT" == *"alice -> Alice Johnson"* ]] || [[ ! "$ROSTER_OUTPUT" == *"bob -> Bob Wilson"* ]]; then
    echo "❌ Global roster listing failed"
    echo "Output: $ROSTER_OUTPUT"
    exit 1
fi

echo "✅ Test 12: Global roster - duplicate alias handling"
ERROR_OUTPUT=$(./git-pair add --global alice "Alice Smith" alice.smith@company.com 2>&1 || true)
if [[ ! "$ERROR_OUTPUT" == *"already exists"* ]]; then
    echo "❌ Duplicate alias error handling failed"
    echo "Error output: $ERROR_OUTPUT"
    exit 1
fi

echo "✅ Test 13: Global roster - quick add"
# Re-initialize since we cleared earlier
./git-pair init
./git-pair add alice
./git-pair add bob

QUICK_STATUS=$(./git-pair status)
if [[ ! "$QUICK_STATUS" == *"Alice Johnson"* ]] || [[ ! "$QUICK_STATUS" == *"Bob Wilson"* ]]; then
    echo "❌ Quick add from global roster failed"
    echo "Status: $QUICK_STATUS"
    exit 1
fi

echo "✅ Test 14: Global roster - non-existent alias"
ERROR_OUTPUT=$(./git-pair add charlie 2>&1 || true)
if [[ ! "$ERROR_OUTPUT" == *"not found in global roster"* ]]; then
    echo "❌ Non-existent alias error handling failed"
    echo "Error output: $ERROR_OUTPUT"
    exit 1
fi

# Cleanup temporary HOME
rm -rf "$TEMP_ROSTER_FILE"
unset GIT_PAIR_ROSTER_FILE

# Cleanup
cd "$OLDPWD"
rm -rf "$TEST_DIR"

echo "🎉 All 14 integration tests passed!"
echo "🚀 git-pair with global roster is ready for use!"
