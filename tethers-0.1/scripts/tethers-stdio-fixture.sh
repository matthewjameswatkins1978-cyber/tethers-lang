#!/bin/sh

# POSIX fixture for Linux tests that exercise the provider protocol rather than
# PowerShell. The Windows test route continues to use the .ps1 fixture.

mode=valid
marker_file=''
barrier_directory=''

while [ "$#" -gt 0 ]; do
    case "$1" in
        -Mode)
            mode="$2"
            shift 2
            ;;
        -MarkerFile)
            marker_file="$2"
            shift 2
            ;;
        -BarrierDirectory)
            barrier_directory="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

append_marker() {
    [ -n "$marker_file" ] && printf '%s\n' "$1" >> "$marker_file"
}

request_id() {
    printf '%s\n' "$1" | sed -n 's/.*"id":[[:space:]]*\([0-9][0-9]*\).*/\1/p'
}

request_method() {
    printf '%s\n' "$1" | sed -n 's/.*"method":"\([^"]*\)".*/\1/p'
}

request_message() {
    printf '%s\n' "$1" | sed -n 's/.*"message":"\([^"]*\)".*/\1/p'
}

write_error() {
    printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32602,"message":"%s"}}\n' "$1" "$2"
}

write_tools() {
    description='Echo one message for deterministic provider binding tests.'
    [ "$mode" = changed-description ] && description='Provider-controlled description changed.'
    tool_name=fixture_ping
    [ "$mode" = wrong-tool ] && tool_name=fixture_other

    input_schema='{"type":"object","properties":{"message":{"type":"string"}},"required":["message"],"additionalProperties":false}'
    output_schema='{"type":"object","properties":{"echo":{"type":"string"}},"required":["echo"],"additionalProperties":false}'
    [ "$mode" = input-schema-mismatch ] && input_schema='{"type":"object","properties":{"message":{"type":"string"}},"required":["different"],"additionalProperties":false}'
    [ "$mode" = output-schema-mismatch ] && output_schema='{"type":"object","properties":{"echo":{"type":"string"}},"required":["different"],"additionalProperties":false}'

    if [ "$mode" = missing-tool ]; then
        tools='[]'
    else
        tool=$(printf '{"name":"%s","description":"%s","inputSchema":%s,"outputSchema":%s}' "$tool_name" "$description" "$input_schema" "$output_schema")
        tools="[$tool]"
        [ "$mode" = duplicate-tool ] && tools="[$tool,$tool]"
    fi
    printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":%s}}\n' "$1" "$tools"
}

barrier_call() {
    [ -n "$barrier_directory" ] || return 0
    mkdir -p "$barrier_directory"
    message=$(request_message "$1")
    token=$(printf '%s\n' "$message" | sed -n 's/.*\(member-[a-z0-9]*\)$/\1/p')
    [ -n "$token" ] || token="pid-$$"
    touch "$barrier_directory/entered-$token"
    peer_count=2
    if [ -f "$barrier_directory/peer-count" ]; then
        peer_count=$(cat "$barrier_directory/peer-count")
    fi
    deadline=$(( $(date +%s) + 10 ))
    while [ "$(find "$barrier_directory" -maxdepth 1 -name 'entered-*' -type f | wc -l)" -lt "$peer_count" ]; do
        [ "$(date +%s)" -le "$deadline" ] || return 1
        sleep 0.01
    done
    touch "$barrier_directory/active-$token"
    while [ ! -f "$barrier_directory/release-$token" ] && [ ! -f "$barrier_directory/release" ]; do
        [ "$(date +%s)" -le "$deadline" ] || return 1
        sleep 0.01
    done
    outcome=success
    if [ -f "$barrier_directory/outcome-$token" ]; then
        outcome=$(cat "$barrier_directory/outcome-$token")
    fi
    case "$outcome" in
        failed)
            return 2
            ;;
        uncertain)
            return 3
            ;;
    esac
}

[ "$mode" = exit-early ] && exit 0

tools_list_count=0
while IFS= read -r line; do
    method=$(request_method "$line")
    id=$(request_id "$line")
    case "$method" in
        initialize)
            append_marker initialize
            case "$mode" in
                malformed-json)
                    printf '%s\n' '{not-json}'
                    ;;
                initialization-error)
                    write_error "$id" 'Initialization rejected by fixture'
                    ;;
                incompatible-version)
                    printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"1900-01-01","capabilities":{"tools":{}},"serverInfo":{"name":"tethers-stdio-fixture","version":"0.1.0"}}}\n' "$id"
                    ;;
                server-name-mismatch)
                    printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"unexpected-provider","version":"0.1.0"}}}\n' "$id"
                    ;;
                *)
                    printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"tethers-stdio-fixture","version":"0.1.0"}}}\n' "$id"
                    ;;
            esac
            ;;
        notifications/initialized)
            ;;
        tools/list)
            tools_list_count=$((tools_list_count + 1))
            append_marker tools/list
            if [ "$mode" = catalogue-change-unchanged ] || [ "$mode" = catalogue-change-drift ]; then
                if [ "$tools_list_count" -eq 1 ]; then
                    printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/tools/list_changed","params":{}}'
                fi
                if [ "$mode" = catalogue-change-drift ] && [ "$tools_list_count" -gt 1 ]; then
                    mode=input-schema-mismatch
                fi
            fi
            write_tools "$id"
            ;;
        tools/call)
            append_marker tools/call
            if [ "$mode" = c2-overlap-barrier ]; then
                barrier_call "$line"
                barrier_status=$?
                if [ "$barrier_status" -eq 1 ]; then
                    write_error "$id" 'overlap peer did not enter'
                    continue
                elif [ "$barrier_status" -eq 2 ]; then
                    write_error "$id" 'controlled provider failure'
                    continue
                elif [ "$barrier_status" -eq 3 ]; then
                    printf '{"jsonrpc":"2.0","id":%s}\n' "$id"
                    continue
                fi
            fi
            message=$(request_message "$line")
            printf '{"jsonrpc":"2.0","id":%s,"result":{"echo":"%s"}}\n' "$id" "$message"
            ;;
        *)
            [ -n "$id" ] && write_error "$id" 'method not found'
            ;;
    esac
done
