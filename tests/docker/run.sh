#!/usr/bin/env bash
# Build the tmuxido Docker test image and run the container integration tests.
#
# Usage:
#   ./tests/docker/run.sh            # build + run
#   ./tests/docker/run.sh --no-cache # force rebuild from scratch
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
IMAGE_NAME="tmuxido-test"

# Propagate --no-cache if requested
BUILD_FLAGS=()
if [[ "${1:-}" == "--no-cache" ]]; then
    BUILD_FLAGS+=(--no-cache)
fi

echo "╔══════════════════════════════════════════════════════════╗"
echo "║         tmuxido — Docker Integration Test Runner        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Project root : $PROJECT_ROOT"
echo "Dockerfile   : $SCRIPT_DIR/Dockerfile"
echo "Image name   : $IMAGE_NAME"
echo ""

# ---- Build ----------------------------------------------------------------
echo "Building image (stage 1: rust compile, stage 2: ubuntu test env)..."
docker build \
    "${BUILD_FLAGS[@]}" \
    --tag "$IMAGE_NAME" \
    --file "$SCRIPT_DIR/Dockerfile" \
    "$PROJECT_ROOT"

echo ""
echo "Build complete. Running tests..."
echo ""

# ---- Run ------------------------------------------------------------------
docker run \
    --rm \
    --name "${IMAGE_NAME}-run" \
    "$IMAGE_NAME"

EXIT=$?

if [ "$EXIT" -eq 0 ]; then
    echo "All tests passed."
else
    echo "Some tests FAILED (exit $EXIT)."
fi

exit "$EXIT"
