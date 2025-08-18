#!/bin/bash

# Setup git configuration for WriteMagic project
# Run this script to configure git with conventional commit templates

echo "🔧 Setting up Git configuration for WriteMagic..."

# Set commit message template
git config commit.template .gitmessage
echo "✅ Set commit message template"

# Set up helpful git aliases for conventional commits
git config alias.feat "!f() { git commit -m \"feat(\$1): \$2\"; }; f"
git config alias.fix "!f() { git commit -m \"fix(\$1): \$2\"; }; f"
git config alias.docs "!f() { git commit -m \"docs(\$1): \$2\"; }; f"
git config alias.style "!f() { git commit -m \"style(\$1): \$2\"; }; f"
git config alias.refactor "!f() { git commit -m \"refactor(\$1): \$2\"; }; f"
git config alias.test "!f() { git commit -m \"test(\$1): \$2\"; }; f"
git config alias.chore "!f() { git commit -m \"chore(\$1): \$2\"; }; f"
git config alias.perf "!f() { git commit -m \"perf(\$1): \$2\"; }; f"
git config alias.ci "!f() { git commit -m \"ci(\$1): \$2\"; }; f"
echo "✅ Set up conventional commit aliases"

# Set up branch naming convention
echo "✅ Branch naming convention:"
echo "   feature/[agent]-[task-name] - for new features"
echo "   fix/[agent]-[issue-description] - for bug fixes"
echo "   docs/[scope] - for documentation updates"
echo "   refactor/[scope] - for refactoring"

# Set up git hooks directory (if needed)
if [ ! -d ".git/hooks" ]; then
    mkdir -p .git/hooks
fi

echo ""
echo "🎉 Git setup complete!"
echo ""
echo "Usage examples:"
echo "  git feat mobile 'add gesture navigation'"
echo "  git fix core 'resolve memory leak in FFI'"
echo "  git docs agents 'update sub-agent documentation'"
echo ""
echo "For detailed commits, use: git commit (opens template)"
echo "For quick commits, use aliases above"