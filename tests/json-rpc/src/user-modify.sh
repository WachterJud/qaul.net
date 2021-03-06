#!/usr/bin/env bash

# Set `dispaly_name` and `real_name` of authenticated user
# 
# usage
# ./user-modify.sh
#

curl -iv  \
    -H "Content-Type: application/json" \
    -d "{ 
        \"id\": \"1\", 
        \"kind\": \"user\", 
        \"method\": \"modify\",
        \"data\": {
            \"display_name\": {
				\"set\": \"testuser\"
			},
            \"real_name\": {
                \"set\": \"Test User\"
            }
        },
        \"auth\": {
            \"id\":\"$A_ID\",
            \"token\":\"$A_TOKEN\"
        }
    }" \
    "http://127.0.0.1:9900/rpc"

