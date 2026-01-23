#!/bin/bash
# .github/utils/generate_mutants_report.sh
#
# Usage: Run from the root of troc repository
#   ./.github/utils/generate_mutants_report.sh

set -e

# Paths relative to troc root directory
MUTANTS_DIR="mutants.out"
OUTPUT_FILE="mutants_report.html"

# Check we're in the right directory
if [ ! -d ".github" ]; then
    echo "Error: Must be run from troc root directory"
    echo "Usage: ./.github/utils/generate_mutants_report.sh"
    exit 1
fi

if [ ! -d "$MUTANTS_DIR" ]; then
    echo "Error: $MUTANTS_DIR directory not found"
    echo "Please run 'cargo mutants' first"
    exit 1
fi

echo "Generating mutation testing report..."
echo "Reading from: $MUTANTS_DIR"
echo "Output file: $OUTPUT_FILE"

# Function to escape content for HTML (to be used in innerHTML)
escape_for_html() {
    sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g'
}

# Function to process diff file and add styling
process_diff() {
    local diff_file="$1"
    if [ -f "$diff_file" ]; then
        while IFS= read -r line; do
            local escaped_line=$(echo "$line" | escape_for_html)
            if [[ $line == +* ]] && [[ $line != +++* ]]; then
                echo "<span class='diff-line diff-add'>$escaped_line</span>"
            elif [[ $line == -* ]] && [[ $line != ---* ]]; then
                echo "<span class='diff-line diff-remove'>$escaped_line</span>"
            else
                echo "<span class='diff-line diff-context'>$escaped_line</span>"
            fi
        done < "$diff_file"
    else
        echo "<p style='color: #95a5a6;'>No diff available</p>"
    fi
}

# Create HTML header
cat > "$OUTPUT_FILE" << 'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Mutation Testing Report - Troc</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        
        h1 {
            color: #2c3e50;
            margin-bottom: 10px;
            font-size: 2.5em;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
        }
        
        h2 {
            color: #34495e;
            margin: 40px 0 20px 0;
            font-size: 2em;
            border-bottom: 2px solid #ecf0f1;
            padding-bottom: 8px;
        }
        
        h3 {
            color: #7f8c8d;
            margin: 30px 0 15px 0;
            font-size: 1.5em;
        }
        
        .summary {
            background: #ecf0f1;
            padding: 20px;
            border-radius: 6px;
            margin: 20px 0;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
        }
        
        .summary-item {
            text-align: center;
        }
        
        .summary-item .number {
            font-size: 2em;
            font-weight: bold;
            display: block;
        }
        
        .summary-item .label {
            color: #7f8c8d;
            font-size: 0.9em;
        }
        
        .missed { color: #e74c3c; }
        .caught { color: #27ae60; }
        .timeout { color: #f39c12; }
        .unviable { color: #95a5a6; }
        
        .index-list {
            list-style: none;
            padding: 0;
        }
        
        .index-list li {
            padding: 8px 0;
            border-bottom: 1px solid #ecf0f1;
        }
        
        .index-list li:last-child {
            border-bottom: none;
        }
        
        .index-list a {
            color: #3498db;
            text-decoration: none;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.95em;
        }
        
        .index-list a:hover {
            color: #2980b9;
            text-decoration: underline;
        }
        
        .mutant-entry {
            margin: 30px 0;
            padding: 20px;
            background: #f8f9fa;
            border-left: 4px solid #3498db;
            border-radius: 4px;
        }
        
        .mutant-entry.missed {
            border-left-color: #e74c3c;
        }
        
        .mutant-entry.caught {
            border-left-color: #27ae60;
        }
        
        .mutant-title {
            font-size: 1.2em;
            font-weight: 600;
            color: #2c3e50;
            margin-bottom: 15px;
            font-family: 'Monaco', 'Courier New', monospace;
        }
        
        .log-content {
            background: #282c34;
            color: #abb2bf;
            border-radius: 4px;
            padding: 15px;
            overflow-x: auto;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.85em;
            line-height: 1.5;
            max-height: 600px;
            overflow-y: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
        }
        
        .diff-content {
            background: #f8f9fa;
            color: #333;
            border: 1px solid #dee2e6;
            border-radius: 4px;
            padding: 15px;
            overflow-x: auto;
            font-family: 'Monaco', 'Courier New', monospace;
            font-size: 0.85em;
            line-height: 1.5;
        }
        
        .diff-line {
            display: block;
            white-space: pre;
        }
        
        .diff-add {
            background: #d4edda;
            color: #155724;
        }
        
        .diff-remove {
            background: #f8d7da;
            color: #721c24;
        }
        
        .diff-context {
            color: #6c757d;
        }
        
        .back-to-top {
            position: fixed;
            bottom: 30px;
            right: 30px;
            background: #3498db;
            color: white;
            padding: 12px 20px;
            border-radius: 6px;
            text-decoration: none;
            box-shadow: 0 2px 8px rgba(0,0,0,0.2);
            transition: background 0.3s;
        }
        
        .back-to-top:hover {
            background: #2980b9;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🧬 Mutation Testing Report - Troc</h1>
HTMLEOF

echo "        <p style=\"color: #7f8c8d; margin-bottom: 20px;\">Generated: $(date '+%Y-%m-%d %H:%M:%S')</p>" >> "$OUTPUT_FILE"

cat >> "$OUTPUT_FILE" << 'HTMLEOF'
        
        <div class="summary" id="summary"></div>
        
        <h2 id="index">Index</h2>
        
        <h3>Missed Mutants</h3>
        <ul class="index-list" id="index-missed"></ul>
        
        <h3>Caught Mutants</h3>
        <ul class="index-list" id="index-caught"></ul>
        
        <h2 id="missed-section">Missed Mutants</h2>
        <div id="missed-content"></div>
        
        <h2 id="caught-section">Caught Mutants</h2>
        <div id="caught-content"></div>
    </div>
    
    <a href="#index" class="back-to-top">↑ Back to Top</a>
</body>
</html>
HTMLEOF

# Initialize counters and arrays
caught_count=0
missed_count=0
timeout_count=0
unviable_count=0

declare -a missed_mutants
declare -a caught_mutants

# Parse mutants.out directory
echo "Parsing mutation results..."

if [ ! -d "$MUTANTS_DIR/log" ]; then
    echo "Warning: $MUTANTS_DIR/log directory not found"
    echo "No mutants to process"
else
    for log_file in "$MUTANTS_DIR"/log/*.log; do
        [ -f "$log_file" ] || continue
        
        mutant_name=$(basename "$log_file" .log)
        
        # Determine outcome by checking the log content
        if grep -q "NOT CAUGHT" "$log_file" || grep -qi "missed" "$log_file"; then
            outcome="missed"
            ((missed_count++))
            missed_mutants+=("$mutant_name")
        elif grep -q "timeout" "$log_file" || grep -qi "timed out" "$log_file"; then
            outcome="timeout"
            ((timeout_count++))
        elif grep -q "UNVIABLE" "$log_file" || grep -qi "unviable" "$log_file"; then
            outcome="unviable"
            ((unviable_count++))
        elif grep -q "CAUGHT" "$log_file" || grep -q "test result: FAILED" "$log_file"; then
            outcome="caught"
            ((caught_count++))
            caught_mutants+=("$mutant_name")
        else
            # Default to missed if uncertain
            outcome="missed"
            ((missed_count++))
            missed_mutants+=("$mutant_name")
        fi
        
        echo -n "."
    done
    echo ""
fi

# Now generate the actual content

# Generate missed mutants index and content
echo "Generating missed mutants..."
for mutant_name in "${missed_mutants[@]}"; do
    log_file="$MUTANTS_DIR/log/${mutant_name}.log"
    
    if [ ! -f "$log_file" ]; then
        continue
    fi
    
    # Extract file path from log
    file_path=$(grep -m 1 "Mutating" "$log_file" | sed 's/.*Mutating //' | sed 's/:.*$//' || echo "$mutant_name")
    
    # Read and escape log content for HTML
    log_content=$(cat "$log_file" | escape_for_html)
    
    # Add to index
    echo "<li><a href=\"#missed-${mutant_name}\">${file_path}</a></li>" >> "${OUTPUT_FILE}.index.missed.tmp"
    
    # Add full log entry
    cat >> "${OUTPUT_FILE}.missed.tmp" << ENTRYEOF
<div class="mutant-entry missed" id="missed-${mutant_name}">
    <div class="mutant-title">${file_path}</div>
    <div class="log-content">${log_content}</div>
</div>
ENTRYEOF
done

# Generate caught mutants index and content
echo "Generating caught mutants..."
for mutant_name in "${caught_mutants[@]}"; do
    log_file="$MUTANTS_DIR/log/${mutant_name}.log"
    diff_file="$MUTANTS_DIR/diff/${mutant_name}.diff"
    
    if [ ! -f "$log_file" ]; then
        continue
    fi
    
    # Extract file path from log
    file_path=$(grep -m 1 "Mutating" "$log_file" | sed 's/.*Mutating //' | sed 's/:.*$//' || echo "$mutant_name")
    
    # Process diff
    diff_content=$(process_diff "$diff_file")
    
    # Add to index
    echo "<li><a href=\"#caught-${mutant_name}\">${file_path}</a></li>" >> "${OUTPUT_FILE}.index.caught.tmp"
    
    # Add full diff entry
    cat >> "${OUTPUT_FILE}.caught.tmp" << ENTRYEOF
<div class="mutant-entry caught" id="caught-${mutant_name}">
    <div class="mutant-title">${file_path}</div>
    <div class="diff-content">${diff_content}</div>
</div>
ENTRYEOF
done

# Calculate mutation score
total_count=$((caught_count + missed_count + timeout_count + unviable_count))

if [ $total_count -eq 0 ]; then
    mutation_score="0.00"
else
    mutation_score=$(awk "BEGIN {printf \"%.2f\", ($caught_count / $total_count) * 100}")
fi

# Now inject all content using JavaScript
cat >> "$OUTPUT_FILE" << JSEOF
<script>
// Inject summary
document.getElementById('summary').innerHTML = \`
    <div class="summary-item">
        <span class="number">$total_count</span>
        <span class="label">Total Mutants</span>
    </div>
    <div class="summary-item">
        <span class="number caught">$caught_count</span>
        <span class="label">Caught</span>
    </div>
    <div class="summary-item">
        <span class="number missed">$missed_count</span>
        <span class="label">Missed</span>
    </div>
    <div class="summary-item">
        <span class="number timeout">$timeout_count</span>
        <span class="label">Timeout</span>
    </div>
    <div class="summary-item">
        <span class="number unviable">$unviable_count</span>
        <span class="label">Unviable</span>
    </div>
    <div class="summary-item">
        <span class="number" style="color: #3498db;">${mutation_score}%</span>
        <span class="label">Mutation Score</span>
    </div>
\`;

// Inject missed index
JSEOF

if [ -f "${OUTPUT_FILE}.index.missed.tmp" ]; then
    echo "document.getElementById('index-missed').innerHTML = \`" >> "$OUTPUT_FILE"
    cat "${OUTPUT_FILE}.index.missed.tmp" >> "$OUTPUT_FILE"
    echo "\`;" >> "$OUTPUT_FILE"
    rm "${OUTPUT_FILE}.index.missed.tmp"
else
    echo "document.getElementById('index-missed').innerHTML = '<li style=\"color: #95a5a6;\">No missed mutants</li>';" >> "$OUTPUT_FILE"
fi

cat >> "$OUTPUT_FILE" << 'JSEOF'

// Inject caught index
JSEOF

if [ -f "${OUTPUT_FILE}.index.caught.tmp" ]; then
    echo "document.getElementById('index-caught').innerHTML = \`" >> "$OUTPUT_FILE"
    cat "${OUTPUT_FILE}.index.caught.tmp" >> "$OUTPUT_FILE"
    echo "\`;" >> "$OUTPUT_FILE"
    rm "${OUTPUT_FILE}.index.caught.tmp"
else
    echo "document.getElementById('index-caught').innerHTML = '<li style=\"color: #95a5a6;\">No caught mutants</li>';" >> "$OUTPUT_FILE"
fi

cat >> "$OUTPUT_FILE" << 'JSEOF'

// Inject missed content
JSEOF

if [ -f "${OUTPUT_FILE}.missed.tmp" ]; then
    echo "document.getElementById('missed-content').innerHTML = \`" >> "$OUTPUT_FILE"
    cat "${OUTPUT_FILE}.missed.tmp" >> "$OUTPUT_FILE"
    echo "\`;" >> "$OUTPUT_FILE"
    rm "${OUTPUT_FILE}.missed.tmp"
else
    echo "document.getElementById('missed-content').innerHTML = '<p style=\"color: #95a5a6;\">No missed mutants</p>';" >> "$OUTPUT_FILE"
fi

cat >> "$OUTPUT_FILE" << 'JSEOF'

// Inject caught content
JSEOF

if [ -f "${OUTPUT_FILE}.caught.tmp" ]; then
    echo "document.getElementById('caught-content').innerHTML = \`" >> "$OUTPUT_FILE"
    cat "${OUTPUT_FILE}.caught.tmp" >> "$OUTPUT_FILE"
    echo "\`;" >> "$OUTPUT_FILE"
    rm "${OUTPUT_FILE}.caught.tmp"
else
    echo "document.getElementById('caught-content').innerHTML = '<p style=\"color: #95a5a6;\">No caught mutants</p>';" >> "$OUTPUT_FILE"
fi

echo "</script>" >> "$OUTPUT_FILE"

# Clean up any remaining temp files
rm -f "${OUTPUT_FILE}".*.tmp

echo ""
echo "✅ Report generated successfully: $OUTPUT_FILE"
echo ""
echo "📊 Summary:"
echo "  Total mutants:   $total_count"
echo "  Caught:          $caught_count"
echo "  Missed:          $missed_count"
echo "  Timeout:         $timeout_count"
echo "  Unviable:        $unviable_count"
echo "  Mutation Score:  ${mutation_score}%"
echo ""
echo "Open the report with:"
echo "  xdg-open $OUTPUT_FILE  # Linux"