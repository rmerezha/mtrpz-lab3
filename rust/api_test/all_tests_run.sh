#!/bin/bash

BASE_URL="http://localhost:8080"
USER1_USERNAME="user1_$(date +%s)"
USER1_PASSWORD="pass1"
USER2_USERNAME="user2_$(date +%s)"
USER2_PASSWORD="pass2"

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
ERROR_MESSAGES=()

echo -e "\n${GREEN}=== Starting API Tests ===${NC}\n"
echo "Base URL: $BASE_URL"
echo "User1: $USER1_USERNAME"
echo "User2: $USER2_USERNAME"

# Helper function to make requests
make_request() {
  local method=$1
  local url=$2
  local data=$3
  local token=$4
  local response
  local status_code

  if [ -n "$token" ]; then
    response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
      -H "Content-Type: application/json" \
      -H "Authorization: $token" \
      -d "$data")
  else
    response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
      -H "Content-Type: application/json" \
      -d "$data")
  fi

  status_code=$(echo "$response" | tail -n1)
  body=$(echo "$response" | sed '$d')
  echo "$status_code $body"
}

# Test reporting function
report_test() {
  local test_name=$1
  local status=$2
  local message=$3

  ((TOTAL_TESTS++))

  if [ "$status" -eq 0 ]; then
    echo -e "${GREEN}✅   ${test_name}${NC}"
    ((PASSED_TESTS++))
  else
    echo -e "${RED}❌   ${test_name}${NC}"
    ((FAILED_TESTS++))
    ERROR_MESSAGES+=("$message")
  fi
}

# Test user signup
echo -n "Testing user1 signup..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/signup" "{\"username\":\"$USER1_USERNAME\",\"password\":\"$USER1_PASSWORD\"}" "")"
[ "$status" -eq 201 ]
report_test "User signup" $? "Signup failed (status $status)"

# Test duplicate signup
echo -n "Testing duplicate signup..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/signup" "{\"username\":\"$USER1_USERNAME\",\"password\":\"$USER1_PASSWORD\"}" "")"
[ "$status" -eq 400 ]
report_test "Duplicate signup prevention" $? "Duplicate signup didn't fail (status $status)"

# Test user1 login
echo -n "Testing user1 login..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/login" "{\"username\":\"$USER1_USERNAME\",\"password\":\"$USER1_PASSWORD\"}" "")"
[ "$status" -eq 200 ] && TOKEN1=$(jq -r '.token' <<< "$body") && REFRESH1=$(jq -r '.refresh_token' <<< "$body") && USER1_ID=$(jq -r '.user_id' <<< "$body")
report_test "User login" $? "Login failed (status $status)"

# Create test note
echo -n "Creating test note..."
if [ -n "$TOKEN1" ]; then
  read status body <<< "$(make_request POST "$BASE_URL/api/notes" "{\"title\":\"Test Note\",\"content\":\"Test Content\"}" "$TOKEN1")"
  [ "$status" -eq 200 ] && NOTE_ID=$(jq -r '.id' <<< "$body")
  report_test "Note creation" $? "Note creation failed (status $status)"
else
  report_test "Note creation" 1 "Skipped - No valid token"
fi

# Test invalid login
echo -n "Testing invalid login..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/login" "{\"username\":\"invalid\",\"password\":\"invalid\"}" "")"
[ "$status" -eq 400 ]
report_test "Invalid login handling" $? "Invalid login didn't fail (status $status)"

# Test note retrieval
echo -n "Testing note retrieval..."
if [ -n "$TOKEN1" ] && [ -n "$NOTE_ID" ]; then
  read status body <<< "$(make_request GET "$BASE_URL/api/notes/$NOTE_ID" "" "$TOKEN1")"
  [ "$status" -eq 200 ]
  report_test "Note retrieval" $? "Note retrieval failed (status $status)"
else
  report_test "Note retrieval" 1 "Skipped - Missing token or note ID"
fi

# Create second user
echo -n "Testing user2 signup..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/signup" "{\"username\":\"$USER2_USERNAME\",\"password\":\"$USER2_PASSWORD\"}" "")"
[ "$status" -eq 201 ]
report_test "Second user signup" $? "Signup2 failed (status $status)"

# Login second user
echo -n "Testing user2 login..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/login" "{\"username\":\"$USER2_USERNAME\",\"password\":\"$USER2_PASSWORD\"}" "")"
[ "$status" -eq 200 ] && TOKEN2=$(jq -r '.token' <<< "$body") && USER2_ID=$(jq -r '.user_id' <<< "$body")
report_test "Second user login" $? "Login2 failed (status $status)"

# Test unauthorized note access
echo -n "Testing unauthorized access..."
if [ -n "$TOKEN2" ] && [ -n "$NOTE_ID" ]; then
  read status body <<< "$(make_request GET "$BASE_URL/api/notes/$NOTE_ID" "" "$TOKEN2")"
  [ "$status" -eq 401 ] || [ "$status" -eq 403 ]
  report_test "Unauthorized access prevention" $? "Unauthorized access allowed (status $status)"
else
  report_test "Unauthorized access prevention" 1 "Skipped - Missing token or note ID"
fi

# Share note with user2
echo -n "Sharing note..."
if [ -n "$TOKEN1" ] && [ -n "$NOTE_ID" ] && [ -n "$USER2_ID" ]; then
  read status body <<< "$(make_request POST "$BASE_URL/api/notes/$NOTE_ID/share" "{\"target_user_id\":\"$USER2_ID\"}" "$TOKEN1")"
  [ "$status" -eq 200 ]
  report_test "Note sharing" $? "Sharing failed (status $status)"
else
  report_test "Note sharing" 1 "Skipped - Missing token, note ID, or user ID"
fi

# Test shared access
echo -n "Testing shared access..."
if [ -n "$TOKEN2" ] && [ -n "$NOTE_ID" ]; then
  read status body <<< "$(make_request GET "$BASE_URL/api/notes/$NOTE_ID" "" "$TOKEN2")"
  if [ "$status" -eq 200 ]; then
    SHARED_NOTE_ID=$(jq -r '.id' <<< "$body")
    SHARED_CONTENT=$(jq -r '.content' <<< "$body")
    [ "$SHARED_NOTE_ID" == "$NOTE_ID" ] && [ "$SHARED_CONTENT" == "Test Content" ]
    report_test "Shared note verification" $? "Shared note mismatch: ID($SHARED_NOTE_ID vs $NOTE_ID) Content('$SHARED_CONTENT' vs 'Test Content')"
  else
    report_test "Shared access" 1 "Shared access failed (status $status)"
  fi
else
  report_test "Shared access" 1 "Skipped - Missing token or note ID"
fi

# Test token refresh
echo -n "Testing token refresh..."
if [ -n "$REFRESH1" ]; then
  read status body <<< "$(make_request POST "$BASE_URL/api/auth/refresh" "{\"refresh_token\":\"$REFRESH1\"}" "$TOKEN1")"
  [ "$status" -eq 200 ] && NEW_TOKEN=$(jq -r '.token' <<< "$body")
  report_test "Token refresh" $? "Refresh failed (status $status)"
else
  report_test "Token refresh" 1 "Skipped - No refresh token"
fi

# Test invalid refresh
echo -n "Testing invalid refresh..."
read status body <<< "$(make_request POST "$BASE_URL/api/auth/refresh" "{\"refresh_token\":\"invalid\"}" "")"
[ "$status" -eq 401 ]
report_test "Invalid refresh handling" $? "Invalid refresh worked (status $status)"

# Test note deletion
echo -n "Deleting note..."
if [ -n "$TOKEN1" ] && [ -n "$NOTE_ID" ]; then
  read status body <<< "$(make_request DELETE "$BASE_URL/api/notes/$NOTE_ID" "" "$TOKEN1")"
  [ "$status" -eq 200 ]
  report_test "Note deletion" $? "Deletion failed (status $status)"
else
  report_test "Note deletion" 1 "Skipped - Missing token or note ID"
fi

# Final report
echo -e "\n${GREEN}=== Test Summary ===${NC}"
echo -e "Total tests: ${TOTAL_TESTS}"
echo -e "${GREEN}Passed: ${PASSED_TESTS}${NC}"
echo -e "${RED}Failed: ${FAILED_TESTS}${NC}"

# Display error messages if any
if [ ${#ERROR_MESSAGES[@]} -gt 0 ]; then
  echo -e "\n${RED}=== Error Details ===${NC}"
  for err in "${ERROR_MESSAGES[@]}"; do
    echo -e "${RED}✖ ${err}${NC}"
  done
fi

exit $FAILED_TESTS
