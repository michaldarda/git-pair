#!/bin/bash
# git-pair installer script
# Usage: curl -sSf https://raw.githubusercontent.com/michaldarda/git-pair/master/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="michaldarda/git-pair"
BINARY_NAME="git-pair"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$os" in
        linux*)
            case "$arch" in
                x86_64|amd64)
                    if ldd /bin/ls >/dev/null 2>&1; then
                        # glibc system
                        echo "x86_64-unknown-linux-gnu"
                    else
                        # musl system (Alpine, etc.)
                        echo "x86_64-unknown-linux-musl"
                    fi
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $arch${NC}" >&2
                    exit 1
                    ;;
            esac
            ;;
        darwin*)
            case "$arch" in
                x86_64|amd64)
                    echo "x86_64-apple-darwin"
                    ;;
                arm64|aarch64)
                    echo "aarch64-apple-darwin"
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $arch${NC}" >&2
                    exit 1
                    ;;
            esac
            ;;
        mingw*|msys*|cygwin*)
            case "$arch" in
                x86_64|amd64)
                    echo "x86_64-pc-windows-msvc"
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $arch${NC}" >&2
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $os${NC}" >&2
            exit 1
            ;;
    esac
}

# Get the latest release version from GitHub API
get_latest_version() {
    local version=""
    if command -v curl >/dev/null 2>&1; then
        version=$(curl -sSf "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | \
            grep '"tag_name":' | \
            sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || echo "")
    elif command -v wget >/dev/null 2>&1; then
        version=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | \
            grep '"tag_name":' | \
            sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || echo "")
    fi
    
    # Fallback to a default version if API call fails
    if [[ -z "$version" ]]; then
        echo -e "${YELLOW}⚠️  Could not fetch latest version from GitHub API${NC}" >&2
        echo -e "${YELLOW}   Using fallback version: v0.1.0${NC}" >&2
        version="v0.1.0"
    fi
    
    echo "$version"
}

# Download and extract the binary
download_and_install() {
    local platform="$1"
    local version="$2"
    local temp_dir=$(mktemp -d)
    
    echo -e "${BLUE}📦 Downloading git-pair $version for $platform...${NC}"
    
    local file_extension=""
    if [[ "$platform" == *"windows"* ]]; then
        file_extension=".zip"
        local archive_name="git-pair-${platform}.zip"
        local binary_name="git-pair.exe"
    else
        file_extension=".tar.gz"
        local archive_name="git-pair-${platform}.tar.gz"
        local binary_name="git-pair"
    fi
    
    local download_url="https://github.com/$REPO/releases/download/$version/$archive_name"
    
    cd "$temp_dir"
    
    # Download the archive
    if command -v curl >/dev/null 2>&1; then
        if ! curl -sSfL "$download_url" -o "$archive_name"; then
            echo -e "${RED}Error: Failed to download $archive_name${NC}" >&2
            echo -e "${RED}URL: $download_url${NC}" >&2
            echo -e "${YELLOW}Please check if the release exists at: https://github.com/$REPO/releases${NC}" >&2
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -q "$download_url" -O "$archive_name"; then
            echo -e "${RED}Error: Failed to download $archive_name${NC}" >&2
            echo -e "${RED}URL: $download_url${NC}" >&2
            echo -e "${YELLOW}Please check if the release exists at: https://github.com/$REPO/releases${NC}" >&2
            exit 1
        fi
    else
        echo -e "${RED}Error: Neither curl nor wget is available${NC}" >&2
        exit 1
    fi
    
    # Extract the archive
    echo -e "${BLUE}📂 Extracting archive...${NC}"
    if [[ "$file_extension" == ".zip" ]]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive_name"
        else
            echo -e "${RED}Error: unzip is not available${NC}" >&2
            exit 1
        fi
    else
        tar -xzf "$archive_name"
    fi
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Install the binary
    echo -e "${BLUE}🔧 Installing to $INSTALL_DIR...${NC}"
    cp "$binary_name" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$binary_name"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    echo -e "${GREEN}✅ git-pair $version installed successfully!${NC}"
}

# Check if git-pair is already installed
check_existing_installation() {
    if command -v git-pair >/dev/null 2>&1; then
        local current_version=$(git-pair --version 2>/dev/null | grep -o 'v[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")
        local latest_version="$1"
        
        echo -e "${YELLOW}⚠️  git-pair is already installed (current version: $current_version)${NC}"
        
        if [[ "$current_version" == "$latest_version" ]]; then
            echo -e "${GREEN}✅ You already have the latest version installed!${NC}"
            echo -e "${BLUE}   Re-installing anyway to ensure a clean installation...${NC}"
        elif [[ "$current_version" != "unknown" ]]; then
            echo -e "${BLUE}📈 Updating from $current_version to $latest_version${NC}"
        else
            echo -e "${YELLOW}   This will overwrite the existing installation.${NC}"
        fi
        echo
    fi
}

# Add to PATH instructions
show_path_instructions() {
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
    echo -e "${BLUE}📋 Next steps:${NC}"
    
    # Check if the install directory is already in PATH
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        echo -e "${GREEN}✅ $INSTALL_DIR is already in your PATH${NC}"
        echo -e "${GREEN}   You can now use 'git-pair' command${NC}"
    else
        echo -e "${YELLOW}⚠️  $INSTALL_DIR is not in your PATH${NC}"
        echo -e "${BLUE}   Add the following line to $shell_profile:${NC}"
        echo
        if [[ "$current_shell" == "fish" ]]; then
            echo -e "${GREEN}   set -gx PATH $INSTALL_DIR \$PATH${NC}"
        else
            echo -e "${GREEN}   export PATH=\"$INSTALL_DIR:\$PATH\"${NC}"
        fi
        echo
        echo -e "${BLUE}   Then restart your terminal or run:${NC}"
        if [[ "$current_shell" == "fish" ]]; then
            echo -e "${GREEN}   source $shell_profile${NC}"
        else
            echo -e "${GREEN}   source $shell_profile${NC}"
        fi
    fi
    
    echo
    echo -e "${BLUE}🚀 Quick start:${NC}"
    echo -e "   ${GREEN}git-pair init${NC}                    # Initialize in your git repo"
    echo -e "   ${GREEN}git-pair add John Doe john@example.com${NC}  # Add a co-author"
    echo -e "   ${GREEN}git-pair status${NC}                  # Check current co-authors"
    echo -e "   ${GREEN}git-pair --help${NC}                  # Show all commands"
}

# Main installation function
main() {
    echo -e "${BLUE}🎯 git-pair installer${NC}"
    echo -e "${BLUE}   Modern pair programming for Git${NC}"
    echo
    
    # Check for existing installation
    check_existing_installation "$version"
    
    # Detect platform
    local platform=$(detect_platform)
    echo -e "${BLUE}🔍 Detected platform: $platform${NC}"
    
    # Get latest version
    echo -e "${BLUE}🔍 Checking for latest release...${NC}"
    local version=$(get_latest_version)
    
    echo -e "${BLUE}📍 Using version: $version${NC}"
    echo
    
    # Download and install
    download_and_install "$platform" "$version"
    
    # Show PATH instructions
    show_path_instructions
    
    echo
    echo -e "${GREEN}🎉 Installation complete!${NC}"
    echo -e "${BLUE}   Documentation: https://github.com/$REPO${NC}"
    echo -e "${BLUE}   Report issues: https://github.com/$REPO/issues${NC}"
}

# Check if script is being piped from curl/wget
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
