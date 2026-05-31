#!/bin/bash
# Validate Moonraker API availability on AD5M
# Usage: MOONRAKER_HOST=192.168.1.100 ./scripts/validate-api.sh

HOST="${MOONRAKER_HOST:-127.0.0.1}"
PORT="${MOONRAKER_PORT:-7125}"
BASE="http://${HOST}:${PORT}"

echo "=== VectorScreen API Validation ==="
echo "Target: ${BASE}"
echo ""

# Test each endpoint
check_endpoint() {
    local name="$1"
    local url="$2"
    local response=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "$url" 2>/dev/null)
    if [ "$response" = "200" ]; then
        echo "✅ $name: AVAILABLE (HTTP $response)"
    elif [ "$response" = "000" ]; then
        echo "❌ $name: UNREACHABLE (connection failed)"
    else
        echo "⚠️  $name: HTTP $response"
    fi
}

echo "--- Server Info ---"
check_endpoint "Server Info" "${BASE}/server/info"
check_endpoint "Server Config" "${BASE}/server/config"

echo ""
echo "--- Printer Objects ---"
check_endpoint "Printer Objects Query" "${BASE}/printer/objects/query?heater_bed&extruder"

echo ""
echo "--- Bed Mesh ---"
check_endpoint "Bed Mesh" "${BASE}/printer/objects/query?bed_mesh"

echo ""
echo "--- Input Shaper ---"
check_endpoint "Input Shaper" "${BASE}/printer/objects/query?input_shaper"
check_endpoint "Resonance Tester" "${BASE}/printer/objects/query?resonance_tester"

echo ""
echo "--- TMC Drivers ---"
check_endpoint "TMC2208 X" "${BASE}/printer/objects/query?tmc2208_x"
check_endpoint "TMC2208 Y" "${BASE}/printer/objects/query?tmc2208_y"
check_endpoint "TMC2209 X" "${BASE}/printer/objects/query?tmc2209_x"
check_endpoint "TMC2209 Y" "${BASE}/printer/objects/query?tmc2209_y"

echo ""
echo "--- File Management ---"
check_endpoint "File List" "${BASE}/server/files/list"
check_endpoint "Macros" "${BASE}/printer/gcode/macros"

echo ""
echo "--- Display Status ---"
check_endpoint "Display Status" "${BASE}/printer/objects/query?display_status"
check_endpoint "Print Stats" "${BASE}/printer/objects/query?print_stats"

echo ""
echo "=== Validation Complete ==="
echo "Review results above to determine feature availability."
