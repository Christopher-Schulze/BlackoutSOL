#!/bin/bash

# BlackoutSOL Optimized Test Runner
# Runs all tests and generates a detailed report
# Enhanced version with error handling and performance metrics

# Color definitions for better output
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[0;33m"
BLUE="\033[0;34m"
NC="\033[0m" # No Color
BOLD="\033[1m"

# Display banner
echo -e "${BOLD}===============================================${NC}"
echo -e "${BOLD}${GREEN}   BLACKOUT SOL OPTIMIZED TEST SUITE v1.0   ${NC}"
echo -e "${BOLD}===============================================${NC}"
echo -e "Started at: ${BLUE}$(date '+%Y-%m-%d %H:%M:%S')${NC}"

# Determine test directory and change to it
CD_DIR=$(dirname "$0")
cd "$CD_DIR/.."

# Prepare log file
LOG_DIR="./test-reports"
LOG_FILE="${LOG_DIR}/test-report-$(date '+%Y%m%d-%H%M%S').log"

# Ensure log directory exists
mkdir -p "${LOG_DIR}"

# Function for logging
log() {
    echo -e "$1"
    echo -e "$1" | sed 's/\x1B\[[0-9;]\{1,\}[A-Za-z]//g' >> "${LOG_FILE}"
}

# Log start of tests
log "${BOLD}[SETUP]${NC} Preparing test environment..."

# Check if advanced performance tests should be run
RUN_PERF_TESTS=0
while getopts "p" opt; do
  case $opt in
    p)
      RUN_PERF_TESTS=1
      log "${YELLOW}[INFO]${NC} Advanced performance tests activated"
      export RUN_EXTENDED_PERF_TESTS=1
      ;;
    \?)
      log "${RED}[ERROR]${NC} Invalid option: -$OPTARG"
      exit 1
      ;;
  esac
done

# Start time measurement
START_TIME=$(date +%s)

# Run main tests
log "\n${BOLD}[TEST]${NC} Running main tests..."
log "${BLUE}----------------------------------------------${NC}"

cargo test -- --nocapture 2>&1 | tee -a "${LOG_FILE}"
MAIN_EXIT_CODE=${PIPESTATUS[0]}

# Capture end time
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Log result
log "\n${BLUE}----------------------------------------------${NC}"
if [ $MAIN_EXIT_CODE -eq 0 ]; then
    log "${GREEN}[SUCCESS]${NC} All tests completed successfully!"
    log "${BOLD}Test duration:${NC} ${DURATION} seconds"
    
    # Additional statistics, if available
    NUM_PASSED=$(grep -c "test.*ok" "${LOG_FILE}" || echo "unbekannt")
    log "${BOLD}Number of passed tests:${NC} ${NUM_PASSED}"

    # Extract performance metrics (if performance tests were run)
    if [ $RUN_PERF_TESTS -eq 1 ]; then
        # Example extraction of performance metrics
        CU_EFFICIENCY=$(grep -o "Compute Unit Efficiency: [0-9]\+\.[0-9]\+%" "${LOG_FILE}" | tail -1 || echo "no data")
        if [ "$CU_EFFICIENCY" != "no data" ]; then
            log "${BOLD}Performance metrics:${NC}"
            log "  $CU_EFFICIENCY"
        fi
    fi
else
    log "${RED}[ERROR]${NC} Tests failed with exit code ${MAIN_EXIT_CODE}"
    # Extract error details
    FAILURES=$(grep -A 3 "thread '.*' panicked" "${LOG_FILE}" || echo "No detailed error information available")
    log "${BOLD}Error messages:${NC}\n$FAILURES"
    log "${YELLOW}[INFO]${NC} Complete log has been saved at: ${LOG_FILE}"
fi

log "\n${BOLD}Tests completed at:${NC} ${BLUE}$(date '+%Y-%m-%d %H:%M:%S')${NC}"

# Optional: Open the log in the default text editor if an error occurred
if [ $MAIN_EXIT_CODE -ne 0 ] && command -v open >/dev/null 2>&1; then
    log "${YELLOW}[INFO]${NC} Opening log file for troubleshooting..."
    open "${LOG_FILE}"
fi

exit $MAIN_EXIT_CODE
