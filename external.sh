#!/bin/bash
# filepath: /home/alan/git/flatheadmill/bkd/external.sh

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXTERNAL_DIR="$SCRIPT_DIR/external"

echo "Setting up external dependencies in: $EXTERNAL_DIR"

# Create external directory if it doesn't exist
mkdir -p "$EXTERNAL_DIR"
cd "$EXTERNAL_DIR"

# Clone or update Lucene
if [ -d "lucene" ]; then
    echo "Updating existing Lucene repository..."
    cd lucene
    git pull origin main || git pull origin master
    cd ..
else
    echo "Cloning Apache Lucene..."
    git clone https://github.com/apache/lucene.git
fi

# Clone or update Tantivy
if [ -d "tantivy" ]; then
    echo "Updating existing Tantivy repository..."
    cd tantivy
    git pull origin main
    cd ..
else
    echo "Cloning Tantivy..."
    git clone https://github.com/quickwit-oss/tantivy.git
fi

echo "External dependencies setup complete!"
echo ""
echo "Key Lucene BKD files are at:"
echo "  external/lucene/lucene/core/src/java/org/apache/lucene/util/bkd/"
echo "  external/lucene/lucene/core/src/java/org/apache/lucene/document/"
echo ""
echo "Tantivy source is at:"
echo "  external/tantivy/"
echo ""
echo "To search for specific patterns:"
echo "  find external/lucene -name '*BKD*' -o -name '*Point*' -o -name '*Shape*'"
echo "  grep -r 'BKDConfig' external/lucene --include='*.java'"