#!/bin/bash
# Check rust-tantraos build status

echo "=== Rust-TantraOS Build Status ==="
echo

# Check if rustc is built
if [ -f "build/aarch64-apple-darwin/stage1/bin/rustc" ]; then
    echo "✅ Stage 1 rustc built"
    echo "   Version: $(./build/aarch64-apple-darwin/stage1/bin/rustc --version)"

    # Check if TantraOS target is available
    if ./build/aarch64-apple-darwin/stage1/bin/rustc --print target-list | grep -q tantraos; then
        echo "✅ TantraOS target available:"
        ./build/aarch64-apple-darwin/stage1/bin/rustc --print target-list | grep tantraos | sed 's/^/   - /'
    else
        echo "❌ TantraOS target not found in rustc"
    fi
else
    echo "⏳ rustc not yet built"
fi

echo

# Check std library status
if [ -d "build/aarch64-apple-darwin/stage1/lib/rustlib/aarch64-tantraos" ]; then
    echo "✅ std library built for aarch64-tantraos"
    echo "   Location: build/aarch64-apple-darwin/stage1/lib/rustlib/aarch64-tantraos/"
else
    echo "⏳ std library not yet built for aarch64-tantraos"
fi

echo

# Check build logs for errors
if [ -f "build/bootstrap-debug.log" ]; then
    ERRORS=$(grep -c ERROR build/bootstrap-debug.log 2>/dev/null || echo 0)
    if [ "$ERRORS" -gt 0 ]; then
        echo "⚠️  Found $ERRORS errors in build log"
        echo "   Last error:"
        grep ERROR build/bootstrap-debug.log | tail -1
    fi
fi

# Check disk usage
echo "Disk usage:"
du -sh build 2>/dev/null | sed 's/^/   /'

echo
echo "To continue build: python3 x.py build --stage 1 -j 10"
echo "To test: ./build/aarch64-apple-darwin/stage1/bin/rustc --target aarch64-tantraos test.rs"