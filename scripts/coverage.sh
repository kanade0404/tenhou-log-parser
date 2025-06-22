#!/bin/bash

# Generate code coverage report
echo "🧪 Generating code coverage report with tarpaulin..."

# Create coverage directory if it doesn't exist
mkdir -p coverage

# Run tarpaulin to generate coverage
cargo tarpaulin \
    --out xml \
    --out html \
    --output-dir ./coverage \
    --skip-clean \
    --verbose

echo "📊 Coverage report generated in ./coverage/"
echo "📄 HTML report: ./coverage/tarpaulin-report.html"
echo "📊 XML report: ./coverage/cobertura.xml"

# Open HTML report if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🌐 Opening coverage report in browser..."
    open ./coverage/tarpaulin-report.html
fi