#!/bin/bash
# git-pair uninstaller script
# Usage: curl -sSf https://raw.githubusercontent.com/michaldarda/git-pair/master/uninstall.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="git-pair"
INSTALL_DIR="$HOME/.local/bin"

# Check if git-pair is installed
check_installation() {
    if [[ -f "$INSTALL_DIR/$BINARY_NAME" ]]; then
        local version=$(git-pair --version 2>/dev/null | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")
        echo -e "${BLUE}🔍 Found git-pair $version at $INSTALL_DIR/$BINARY_NAME${NC}"
        return 0
    elif command -v git-pair >/dev/null 2>&1; then
        local install_location=$(which git-pair)
        local version=$(git-pair --version 2>/dev/null | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")
        echo -e "${YELLOW}⚠️  Found git-pair $version at $install_location${NC}"
        echo -e "${YELLOW}   This appears to be installed outside of $INSTALL_DIR${NC}"
        echo -e "${YELLOW}   This script can only remove installations from $INSTALL_DIR${NC}"
        return 1
    else
        echo -e "${YELLOW}⚠️  git-pair is not installed or not found in PATH${NC}"
        return 1
    fi
}

# Remove the binary
remove_binary() {
    echo -e "${BLUE}🗑️  Removing git-pair from $INSTALL_DIR...${NC}"
    
    if [[ -f "$INSTALL_DIR/$BINARY_NAME" ]]; then
        rm "$INSTALL_DIR/$BINARY_NAME"
        echo -e "${GREEN}✅ Removed $INSTALL_DIR/$BINARY_NAME${NC}"
    else
        echo -e "${YELLOW}⚠️  Binary not found at $INSTALL_DIR/$BINARY_NAME${NC}"
    fi
}

# Show PATH cleanup instructions
show_cleanup_instructions() {
    local shell_profile=""
    local current_shell=$(basename "$SHELL")
    
    case "$current_shell" in
        bash)
            if [[ -f "$HOME/.bashrc" ]]; then
                shell_profile="$HOME/.bashrc"
            elif [[ -f "$HOME/.bash_profile" ]]; then
                shell_profile="$HOME/.bash_profile"
            fi
            ;;
        zsh)
            shell_profile="$HOME/.zshrc"
            ;;
        fish)
            shell_profile="$HOME/.config/fish/config.fish"
            ;;
        *)
            shell_profile="your shell profile"
            ;;
    esac
    
    echo
    echo -e "${BLUE}📋 Optional cleanup:${NC}"
    echo -e "${BLUE}   If you added $INSTALL_DIR to your PATH specifically for git-pair,${NC}"
    echo -e "${BLUE}   you may want to remove it from $shell_profile${NC}"
    echo
    echo -e "${BLUE}   Look for lines like:${NC}"
    if [[ "$current_shell" == "fish" ]]; then
        echo -e "${YELLOW}   set -gx PATH $INSTALL_DIR \$PATH${NC}"
    else
        echo -e "${YELLOW}   export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
    fi
    echo
    echo -e "${BLUE}🧹 Clean up git-pair configuration files:${NC}"
    echo -e "${BLUE}   Your git repositories may still have git-pair configuration files.${NC}"
    echo -e "${BLUE}   These are stored in .git/git-pair/ directories in your repos.${NC}"
    echo -e "${BLUE}   Global roster: ~/.config/git-pair/roster${NC}"
    echo
    echo -e "${BLUE}   To remove global configuration:${NC}"
    echo -e "${GREEN}   rm -rf ~/.config/git-pair/${NC}"
}

# Main uninstallation function
main() {
    echo -e "${BLUE}🗑️  git-pair uninstaller${NC}"
    echo
    
    # Check if git-pair is installed
    if ! check_installation; then
        echo -e "${GREEN}✅ Nothing to uninstall${NC}"
        exit 0
    fi
    
    echo
    echo -e "${YELLOW}⚠️  This will remove git-pair from your system.${NC}"
    echo -e "${YELLOW}   Your git repositories' pair configurations will remain untouched.${NC}"
    echo
    read -p "Do you want to continue? [y/N] " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}❌ Uninstallation cancelled${NC}"
        exit 0
    fi
    
    # Remove the binary
    remove_binary
    
    # Show cleanup instructions
    show_cleanup_instructions
    
    echo
    echo -e "${GREEN}🎉 git-pair has been uninstalled!${NC}"
    echo -e "${BLUE}   Thank you for using git-pair!${NC}"
    echo -e "${BLUE}   Feedback: https://github.com/michaldarda/git-pair/issues${NC}"
}

# Run the main function unless this script is being sourced
# This works for both direct execution and piped execution from curl/wget
if [[ "${BASH_SOURCE[0]:-$0}" == "${0}" ]] || [[ -z "${BASH_SOURCE[0]:-}" ]]; then
    main "$@"
fi
