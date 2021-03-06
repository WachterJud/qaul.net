#!/usr/bin/env bash

# modify a chatroom by id
# 
# usage:
# ./chat-room-modify.sh <ROOM_ID> <USER_ID> <USER_TOKEN>

curl -i  \
    -H "Content-Type: application/json" \
    -d "{ \"id\": \"/chat_room/modify\", 
          \"kind\": \"chat_room\", 
          \"method\": \"modify\",
          \"data\": {
            \"id\":\"$1\",
            \"name\": {
                \"set\": \"Test-Room Name\"
            }
          },
          \"auth\": {
            \"id\":\"$2\",
            \"token\":\"$3\"
          }
        }" \
    "http://127.0.0.1:9900/rpc"

