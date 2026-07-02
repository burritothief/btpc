#compdef btpc

autoload -U is-at-least

_btpc() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_btpc_commands" \
"*::: :->btpc" \
&& ret=0
    case $state in
    (btpc)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
'()--batch=[Versioned TOML batch manifest]:BATCH:_files' \
'--mode=[Torrent protocol representation]:MODE:(v1 v2 hybrid)' \
'-o+[Destination .torrent path (defaults beside the payload)]:OUTPUT:_files' \
'--output=[Destination .torrent path (defaults beside the payload)]:OUTPUT:_files' \
'(-o --output)--output-dir=[Write batch outputs beneath this directory]:OUTPUT_DIR:_files' \
'--jobs=[Maximum concurrent batch creation jobs]:JOBS:_default' \
'*--preset=[Apply a named creation preset; may be repeated]:PRESETS:_default' \
'--piece-length=[Explicit piece length in bytes]:PIECE_LENGTH:_default' \
'--target-pieces=[Target approximate number of pieces for automatic selection]:TARGET_PIECES:_default' \
'--max-piece-length=[Cap target-based automatic piece length]:MAX_PIECE_LENGTH:_default' \
'*-a+[Add a tracker as its own tier; may be repeated]:TRACKERS:_default' \
'*--tracker=[Add a tracker as its own tier; may be repeated]:TRACKERS:_default' \
'*--tracker-tier=[Add one comma-separated tracker tier; may be repeated]:TRACKER_TIER:_default' \
'*--tracker-alias=[Add a configured tracker alias; may be repeated]:TRACKER_ALIASES:_default' \
'*--tracker-group=[Add a configured tracker group; may be repeated]:TRACKER_GROUPS:_default' \
'*--web-seed=[Add a web seed URL; may be repeated]:WEB_SEEDS:_default' \
'*--node=[Add a DHT node as HOST\:PORT; may be repeated]:NODES:_default' \
'--source=[Set the source field]:SOURCE:_default' \
'--comment=[Set the top-level comment]:COMMENT:_default' \
'(--no-created-by)--created-by=[Set the creator string]:CREATED_BY:_default' \
'--creation-date=[Include an explicit Unix creation timestamp]:CREATION_DATE:_default' \
'--entropy=[Set deterministic, random, or omitted entropy policy]:ENTROPY:_default' \
'--name=[Override the torrent root name]:NAME:_default' \
'--symlinks=[Symbolic-link policy]:SYMLINKS:(reject skip follow)' \
'--special-files=[Special-file policy]:SPECIAL_FILES:(reject skip)' \
'*--include=[Include only paths matching this glob; may be repeated]:INCLUDES:_default' \
'*--exclude=[Exclude paths matching this glob; may be repeated]:EXCLUDES:_default' \
'--threads=[v1 hashing threads; 0 selects a conservative automatic count, 1 is sequential]:THREADS:_default' \
'(--json)*--print=[Print selected result fields; may be repeated]:PRINT:(path info-hash-v1 info-hash-v2 magnet)' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--fail-fast[Stop scheduling batch jobs after the first failure]' \
'-f[Replace an existing destination]' \
'--force[Replace an existing destination]' \
'--durable[Sync the destination directory after atomic publication where supported]' \
'--clear-trackers[Clear configured and preset trackers before CLI additions]' \
'--clear-web-seeds[Clear configured and preset web seeds before CLI additions]' \
'(--public)--private[Set the private flag]' \
'--public[Set the private flag to false]' \
'(--source)--clear-source[Remove configured or preset source metadata]' \
'(--comment)--clear-comment[Remove configured or preset comment metadata]' \
'(--created-by)--no-created-by[Omit the creator string instead of using the versioned default]' \
'--exclude-hidden[Exclude dot-prefixed files and directories]' \
'--exclude-empty-files[Exclude zero-length files]' \
'--reject-empty-directories[Reject empty directories instead of ignoring them]' \
'--clear-includes[Clear configured and preset include patterns before CLI additions]' \
'--clear-excludes[Clear configured and preset exclude patterns before CLI additions]' \
'--dry-run[Plan creation without hashing or writing metainfo]' \
'--json[Emit a versioned JSON result to stdout]' \
'(--json -q --quiet)--pretty[Use the expanded human completion renderer]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
'*::inputs -- Payload files or directories:_files' \
&& ret=0
;;
(inspect)
_arguments "${_arguments_options[@]}" : \
'*--field=[Select a field; may be repeated]:FIELDS:(mode name total-size piece-length piece-count file-count hash-v1 hash-v2 private trackers web-seeds nodes comment creator creation-date source canonicality warnings files unknown-fields)' \
'--path-encoding=[Encode raw torrent paths]:PATH_ENCODING:(utf8 escaped hex)' \
'--offset=[Skip this many file rows]:OFFSET:_default' \
'--limit=[Limit returned file rows]:LIMIT:_default' \
'(--json)--format=[Select output representation]:FORMAT:(human plain json json-pretty tsv)' \
'--max-input-bytes=[Maximum metainfo bytes accepted while loading]:MAX_INPUT_BYTES:_default' \
'--max-owned-bytes=[Maximum estimated owned allocation while loading]:MAX_OWNED_BYTES:_default' \
'--max-integer-digits=[Maximum decimal digits accepted in one bencode integer]:MAX_INTEGER_DIGITS:_default' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--json[Emit versioned JSON to stdout]' \
'--files[Include the flat file listing]' \
'(--files)--tree[Render files as a deterministic tree]' \
'(--json -q --quiet)--pretty[Use the expanded human renderer]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':input -- Metainfo file to read:_files' \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
'(--json)--format=[Select output representation]:FORMAT:(human json json-pretty)' \
'--max-input-bytes=[Maximum metainfo bytes accepted while loading]:MAX_INPUT_BYTES:_default' \
'--max-owned-bytes=[Maximum estimated owned allocation while loading]:MAX_OWNED_BYTES:_default' \
'--max-integer-digits=[Maximum decimal digits accepted in one bencode integer]:MAX_INTEGER_DIGITS:_default' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--json[Emit versioned JSON to stdout]' \
'--canonical[Require canonical bencode]' \
'--warnings-as-errors[Return the warning exit code when validation warnings exist]' \
'(--json -q --quiet)--pretty[Use the expanded human renderer]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':input -- Metainfo file to validate:_files' \
&& ret=0
;;
(verify)
_arguments "${_arguments_options[@]}" : \
'--max-input-bytes=[Maximum metainfo bytes accepted while loading]:MAX_INPUT_BYTES:_default' \
'--max-owned-bytes=[Maximum estimated owned allocation while loading]:MAX_OWNED_BYTES:_default' \
'--max-integer-digits=[Maximum decimal digits accepted in one bencode integer]:MAX_INTEGER_DIGITS:_default' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--fail-fast[Stop after the first deterministic mismatch]' \
'--extra-files[Report regular files absent from metainfo]' \
'--json[Emit versioned JSON to stdout]' \
'(--json -q --quiet)--pretty[Use the expanded human renderer]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':torrent -- Metainfo file to read:_files' \
':payload -- Payload file or directory to verify:_files' \
&& ret=0
;;
(edit)
_arguments "${_arguments_options[@]}" : \
'(--in-place)-o+[Write the edited metainfo to this path]:OUTPUT:_files' \
'(--in-place)--output=[Write the edited metainfo to this path]:OUTPUT:_files' \
'*-a+[Replace trackers with this tracker tier; may be repeated]:TRACKERS:_default' \
'*--tracker=[Replace trackers with this tracker tier; may be repeated]:TRACKERS:_default' \
'*--tracker-alias=[Add a configured tracker alias; may be repeated]:TRACKER_ALIASES:_default' \
'*--tracker-group=[Add a configured tracker group; may be repeated]:TRACKER_GROUPS:_default' \
'*--web-seed=[Replace web seeds with this URL; may be repeated]:WEB_SEEDS:_default' \
'*--node=[Replace DHT nodes with HOST\:PORT; may be repeated]:NODES:_default' \
'(--clear-comment)--comment=[Set the top-level comment]:COMMENT:_default' \
'(--clear-created-by)--created-by=[Set the creator string]:CREATED_BY:_default' \
'(--clear-creation-date)--creation-date=[Set the Unix creation timestamp]:CREATION_DATE:_default' \
'(--clear-source)--source=[Set the source field]:SOURCE:_default' \
'*--file-attributes=[Set file attributes as PATH=ATTRS; may be repeated]:FILE_ATTRIBUTES:_default' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--in-place[Replace the input file atomically]' \
'-f[Replace an existing output file]' \
'--force[Replace an existing output file]' \
'--durable[Sync the destination directory after publication where supported]' \
'--dry-run[Validate and report changes without writing output]' \
'--diff[Print a deterministic field-level change summary]' \
'--json[Emit a versioned JSON result to stdout]' \
'--clear-trackers[Remove all trackers]' \
'--clear-web-seeds[Remove all web seeds]' \
'--clear-nodes[Remove all DHT nodes]' \
'--clear-comment[Remove the top-level comment]' \
'--clear-created-by[Remove the creator string]' \
'--clear-creation-date[Remove the creation timestamp]' \
'(--public --clear-private)--private[Set the private flag]' \
'(--clear-private)--public[Set the private flag to false]' \
'--clear-private[Remove the private field]' \
'--clear-source[Remove the source field]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':input -- Metainfo file to edit:_files' \
&& ret=0
;;
(magnet)
_arguments "${_arguments_options[@]}" : \
'--max-input-bytes=[Maximum metainfo bytes accepted while loading]:MAX_INPUT_BYTES:_default' \
'--max-owned-bytes=[Maximum estimated owned allocation while loading]:MAX_OWNED_BYTES:_default' \
'--max-integer-digits=[Maximum decimal digits accepted in one bencode integer]:MAX_INTEGER_DIGITS:_default' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-display-name[Omit the display name parameter]' \
'--no-trackers[Omit tracker parameters]' \
'--no-web-seeds[Omit web seed parameters]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':input -- Metainfo file to read:_files' \
&& ret=0
;;
(config)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_btpc__subcmd__config_commands" \
"*::: :->config" \
&& ret=0

    case $state in
    (config)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-command-$line[1]:"
        case $line[1] in
            (path)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--force[Replace an existing file]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--resolved[Validate and print the parsed deterministic representation]' \
'--show-secrets[Reveal configured secrets]' \
'--json[Emit JSON instead of TOML]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(explain)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_btpc__subcmd__config__subcmd__explain_commands" \
"*::: :->explain" \
&& ret=0

    case $state in
    (explain)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-explain-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
'()--batch=[Versioned TOML batch manifest]:BATCH:_files' \
'--mode=[Torrent protocol representation]:MODE:(v1 v2 hybrid)' \
'-o+[Destination .torrent path (defaults beside the payload)]:OUTPUT:_files' \
'--output=[Destination .torrent path (defaults beside the payload)]:OUTPUT:_files' \
'(-o --output)--output-dir=[Write batch outputs beneath this directory]:OUTPUT_DIR:_files' \
'--jobs=[Maximum concurrent batch creation jobs]:JOBS:_default' \
'*--preset=[Apply a named creation preset; may be repeated]:PRESETS:_default' \
'--piece-length=[Explicit piece length in bytes]:PIECE_LENGTH:_default' \
'--target-pieces=[Target approximate number of pieces for automatic selection]:TARGET_PIECES:_default' \
'--max-piece-length=[Cap target-based automatic piece length]:MAX_PIECE_LENGTH:_default' \
'*-a+[Add a tracker as its own tier; may be repeated]:TRACKERS:_default' \
'*--tracker=[Add a tracker as its own tier; may be repeated]:TRACKERS:_default' \
'*--tracker-tier=[Add one comma-separated tracker tier; may be repeated]:TRACKER_TIER:_default' \
'*--tracker-alias=[Add a configured tracker alias; may be repeated]:TRACKER_ALIASES:_default' \
'*--tracker-group=[Add a configured tracker group; may be repeated]:TRACKER_GROUPS:_default' \
'*--web-seed=[Add a web seed URL; may be repeated]:WEB_SEEDS:_default' \
'*--node=[Add a DHT node as HOST\:PORT; may be repeated]:NODES:_default' \
'--source=[Set the source field]:SOURCE:_default' \
'--comment=[Set the top-level comment]:COMMENT:_default' \
'(--no-created-by)--created-by=[Set the creator string]:CREATED_BY:_default' \
'--creation-date=[Include an explicit Unix creation timestamp]:CREATION_DATE:_default' \
'--entropy=[Set deterministic, random, or omitted entropy policy]:ENTROPY:_default' \
'--name=[Override the torrent root name]:NAME:_default' \
'--symlinks=[Symbolic-link policy]:SYMLINKS:(reject skip follow)' \
'--special-files=[Special-file policy]:SPECIAL_FILES:(reject skip)' \
'*--include=[Include only paths matching this glob; may be repeated]:INCLUDES:_default' \
'*--exclude=[Exclude paths matching this glob; may be repeated]:EXCLUDES:_default' \
'--threads=[v1 hashing threads; 0 selects a conservative automatic count, 1 is sequential]:THREADS:_default' \
'(--json)*--print=[Print selected result fields; may be repeated]:PRINT:(path info-hash-v1 info-hash-v2 magnet)' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--fail-fast[Stop scheduling batch jobs after the first failure]' \
'-f[Replace an existing destination]' \
'--force[Replace an existing destination]' \
'--durable[Sync the destination directory after atomic publication where supported]' \
'--clear-trackers[Clear configured and preset trackers before CLI additions]' \
'--clear-web-seeds[Clear configured and preset web seeds before CLI additions]' \
'(--public)--private[Set the private flag]' \
'--public[Set the private flag to false]' \
'(--source)--clear-source[Remove configured or preset source metadata]' \
'(--comment)--clear-comment[Remove configured or preset comment metadata]' \
'(--created-by)--no-created-by[Omit the creator string instead of using the versioned default]' \
'--exclude-hidden[Exclude dot-prefixed files and directories]' \
'--exclude-empty-files[Exclude zero-length files]' \
'--reject-empty-directories[Reject empty directories instead of ignoring them]' \
'--clear-includes[Clear configured and preset include patterns before CLI additions]' \
'--clear-excludes[Clear configured and preset exclude patterns before CLI additions]' \
'--dry-run[Plan creation without hashing or writing metainfo]' \
'--json[Emit a versioned JSON result to stdout]' \
'(--json -q --quiet)--pretty[Use the expanded human completion renderer]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
'*::inputs -- Payload files or directories:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__explain__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-explain-help-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(tracker)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_btpc__subcmd__config__subcmd__tracker_commands" \
"*::: :->tracker" \
&& ret=0

    case $state in
    (tracker)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-tracker-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--show-secrets[Reveal tracker URLs]' \
'--json[Emit JSON]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Alias name:_default' \
':url -- Tracker announce URL:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Alias name:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__tracker__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-tracker-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(preset)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_btpc__subcmd__config__subcmd__preset_commands" \
"*::: :->preset" \
&& ret=0

    case $state in
    (preset)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-preset-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--json[Emit JSON]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--show-secrets[Reveal configured URLs]' \
'--json[Emit JSON]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Preset name:_default' \
&& ret=0
;;
(save)
_arguments "${_arguments_options[@]}" : \
'*--extends=[Parent preset; may be repeated]:EXTENDS:_default' \
'--mode=[Torrent protocol representation]:MODE:(v1 v2 hybrid)' \
'--piece-length=[Explicit piece length in bytes]:PIECE_LENGTH:_default' \
'--source=[Set the source field]:SOURCE:_default' \
'--comment=[Set the top-level comment]:COMMENT:_default' \
'--created-by=[Set the creator string]:CREATED_BY:_default' \
'--creation-date=[Set the Unix creation timestamp]:CREATION_DATE:_default' \
'--name=[Override the torrent root name]:NAME_OVERRIDE:_default' \
'--symlinks=[Symbolic-link policy]:SYMLINKS:(reject skip follow)' \
'--special-files=[Special-file policy]:SPECIAL_FILES:(reject skip)' \
'*--tracker=[Add a tracker as its own tier; may be repeated]:TRACKERS:_default' \
'*--tracker-alias=[Add a configured tracker alias; may be repeated]:TRACKER_ALIASES:_default' \
'*--tracker-group=[Add a configured tracker group; may be repeated]:TRACKER_GROUPS:_default' \
'*--web-seed=[Add a web seed URL; may be repeated]:WEB_SEEDS:_default' \
'*--include=[Include only paths matching this glob; may be repeated]:INCLUDES:_default' \
'*--exclude=[Exclude paths matching this glob; may be repeated]:EXCLUDES:_default' \
'--threads=[v1 hashing threads; 0 selects automatic, 1 is sequential]:THREADS:_default' \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--private[Set the private flag]' \
'--exclude-hidden[Exclude dot-prefixed files and directories]' \
'--exclude-empty-files[Exclude zero-length files]' \
'--reject-empty-directories[Reject empty directories instead of ignoring them]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Preset name:_default' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Preset name:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__preset__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-preset-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(save)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-help-command-$line[1]:"
        case $line[1] in
            (path)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(explain)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__help__subcmd__explain_commands" \
"*::: :->explain" \
&& ret=0

    case $state in
    (explain)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-help-explain-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(tracker)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__help__subcmd__tracker_commands" \
"*::: :->tracker" \
&& ret=0

    case $state in
    (tracker)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-help-tracker-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(preset)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__config__subcmd__help__subcmd__preset_commands" \
"*::: :->preset" \
&& ret=0

    case $state in
    (preset)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-config-help-preset-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(save)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(completion)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_btpc__subcmd__completion_commands" \
"*::: :->completion" \
&& ret=0

    case $state in
    (completion)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-completion-command-$line[1]:"
        case $line[1] in
            (generate)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':shell -- Shell whose completion syntax should be generated:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--dry-run[Print the target and generated content without changing files]' \
'--force[Replace an unrelated existing completion file]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
'::shell -- Shell to install; detected from environment hints when omitted:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(uninstall)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--dry-run[Print the target and generated content without changing files]' \
'--force[Replace an unrelated existing completion file]' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
'::shell -- Shell to install; detected from environment hints when omitted:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__completion__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-completion-help-command-$line[1]:"
        case $line[1] in
            (generate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(uninstall)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
':shell -- Shell whose completion syntax should be generated:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(manpage)
_arguments "${_arguments_options[@]}" : \
'(--no-config)--config=[Use this configuration file when configuration loading is enabled]:PATH:_files' \
'--color=[Control colored terminal output]:COLOR:(auto always never)' \
'--no-config[Disable implicit and environment-selected configuration]' \
'(-q --quiet)*-v[Increase diagnostic verbosity; may be repeated]' \
'(-q --quiet)*--verbose[Increase diagnostic verbosity; may be repeated]' \
'-q[Suppress human summaries, warnings, and progress]' \
'--quiet[Suppress human summaries, warnings, and progress]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-help-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(inspect)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(edit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(magnet)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(config)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__help__subcmd__config_commands" \
"*::: :->config" \
&& ret=0

    case $state in
    (config)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-help-config-command-$line[1]:"
        case $line[1] in
            (path)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(check)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(explain)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__help__subcmd__config__subcmd__explain_commands" \
"*::: :->explain" \
&& ret=0

    case $state in
    (explain)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-help-config-explain-command-$line[1]:"
        case $line[1] in
            (create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(tracker)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__help__subcmd__config__subcmd__tracker_commands" \
"*::: :->tracker" \
&& ret=0

    case $state in
    (tracker)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-help-config-tracker-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(preset)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__help__subcmd__config__subcmd__preset_commands" \
"*::: :->preset" \
&& ret=0

    case $state in
    (preset)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-help-config-preset-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(show)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(save)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(completion)
_arguments "${_arguments_options[@]}" : \
":: :_btpc__subcmd__help__subcmd__completion_commands" \
"*::: :->completion" \
&& ret=0

    case $state in
    (completion)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:btpc-help-completion-command-$line[1]:"
        case $line[1] in
            (generate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(uninstall)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(manpage)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_btpc_commands] )) ||
_btpc_commands() {
    local commands; commands=(
'create:Create canonical v1, v2, or hybrid metainfo from a file or directory' \
'inspect:Inspect validated metainfo without reading payload files' \
'validate:Validate metainfo structure without reading payload files' \
'verify:Verify payload files against metainfo hashes' \
'edit:Edit metainfo metadata without reading payload files' \
'magnet:Print a deterministic magnet URI' \
'config:Locate, inspect, validate, and update configuration' \
'completion:Generate, install, or uninstall shell completions' \
'completions:Deprecated alias for \`btpc completion generate\`' \
'manpage:Generate the btpc(1) manual page on stdout' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion_commands] )) ||
_btpc__subcmd__completion_commands() {
    local commands; commands=(
'generate:Generate a shell completion script on stdout' \
'install:Install shell completions in the standard per-user directory' \
'uninstall:Remove BTPC-generated shell completions' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc completion commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__generate_commands] )) ||
_btpc__subcmd__completion__subcmd__generate_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion generate commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__help_commands] )) ||
_btpc__subcmd__completion__subcmd__help_commands() {
    local commands; commands=(
'generate:Generate a shell completion script on stdout' \
'install:Install shell completions in the standard per-user directory' \
'uninstall:Remove BTPC-generated shell completions' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc completion help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__help__subcmd__generate_commands] )) ||
_btpc__subcmd__completion__subcmd__help__subcmd__generate_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion help generate commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__help__subcmd__help_commands] )) ||
_btpc__subcmd__completion__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion help help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__help__subcmd__install_commands] )) ||
_btpc__subcmd__completion__subcmd__help__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion help install commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__help__subcmd__uninstall_commands] )) ||
_btpc__subcmd__completion__subcmd__help__subcmd__uninstall_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion help uninstall commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__install_commands] )) ||
_btpc__subcmd__completion__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion install commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completion__subcmd__uninstall_commands] )) ||
_btpc__subcmd__completion__subcmd__uninstall_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completion uninstall commands' commands "$@"
}
(( $+functions[_btpc__subcmd__completions_commands] )) ||
_btpc__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'btpc completions commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config_commands] )) ||
_btpc__subcmd__config_commands() {
    local commands; commands=(
'path:Print the selected configuration path' \
'init:Create a minimal configuration file' \
'show:Print configuration with secrets redacted by default' \
'check:Validate schema, references, cycles, conflicts, and permissions' \
'explain:Explain resolved command values without executing the command' \
'tracker:Manage named tracker aliases' \
'preset:Manage named creation presets' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__check_commands] )) ||
_btpc__subcmd__config__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config check commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__explain_commands] )) ||
_btpc__subcmd__config__subcmd__explain_commands() {
    local commands; commands=(
'create:Explain effective create values and provenance' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config explain commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__explain__subcmd__create_commands] )) ||
_btpc__subcmd__config__subcmd__explain__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config explain create commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__explain__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__explain__subcmd__help_commands() {
    local commands; commands=(
'create:Explain effective create values and provenance' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config explain help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__create_commands] )) ||
_btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config explain help create commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config explain help help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__help_commands() {
    local commands; commands=(
'path:Print the selected configuration path' \
'init:Create a minimal configuration file' \
'show:Print configuration with secrets redacted by default' \
'check:Validate schema, references, cycles, conflicts, and permissions' \
'explain:Explain resolved command values without executing the command' \
'tracker:Manage named tracker aliases' \
'preset:Manage named creation presets' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__check_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help check commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__explain_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__explain_commands() {
    local commands; commands=(
'create:Explain effective create values and provenance' \
    )
    _describe -t commands 'btpc config help explain commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__explain__subcmd__create_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__explain__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help explain create commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__init_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help init commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__path_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__path_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help path commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__preset_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__preset_commands() {
    local commands; commands=(
'list:List preset names' \
'show:Show one preset' \
'save:Save or replace a preset' \
'remove:Remove a preset' \
    )
    _describe -t commands 'btpc config help preset commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__list_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help preset list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__remove_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help preset remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__save_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__save_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help preset save commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__show_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help preset show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__show_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__tracker_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__tracker_commands() {
    local commands; commands=(
'list:List tracker aliases' \
'add:Add or replace a tracker alias' \
'remove:Remove a tracker alias' \
    )
    _describe -t commands 'btpc config help tracker commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__add_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help tracker add commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__list_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help tracker list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__remove_commands] )) ||
_btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config help tracker remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__init_commands] )) ||
_btpc__subcmd__config__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config init commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__path_commands] )) ||
_btpc__subcmd__config__subcmd__path_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config path commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset_commands] )) ||
_btpc__subcmd__config__subcmd__preset_commands() {
    local commands; commands=(
'list:List preset names' \
'show:Show one preset' \
'save:Save or replace a preset' \
'remove:Remove a preset' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config preset commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__help_commands() {
    local commands; commands=(
'list:List preset names' \
'show:Show one preset' \
'save:Save or replace a preset' \
'remove:Remove a preset' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config preset help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset help help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__list_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset help list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__remove_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset help remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__save_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__save_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset help save commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__show_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset help show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__list_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__remove_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__save_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__save_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset save commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__preset__subcmd__show_commands] )) ||
_btpc__subcmd__config__subcmd__preset__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config preset show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__show_commands] )) ||
_btpc__subcmd__config__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker_commands] )) ||
_btpc__subcmd__config__subcmd__tracker_commands() {
    local commands; commands=(
'list:List tracker aliases' \
'add:Add or replace a tracker alias' \
'remove:Remove a tracker alias' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config tracker commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__add_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker add commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__help_commands() {
    local commands; commands=(
'list:List tracker aliases' \
'add:Add or replace a tracker alias' \
'remove:Remove a tracker alias' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc config tracker help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__add_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker help add commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__help_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker help help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__list_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker help list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__remove_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker help remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__list_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__config__subcmd__tracker__subcmd__remove_commands] )) ||
_btpc__subcmd__config__subcmd__tracker__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc config tracker remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__create_commands] )) ||
_btpc__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'btpc create commands' commands "$@"
}
(( $+functions[_btpc__subcmd__edit_commands] )) ||
_btpc__subcmd__edit_commands() {
    local commands; commands=()
    _describe -t commands 'btpc edit commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help_commands] )) ||
_btpc__subcmd__help_commands() {
    local commands; commands=(
'create:Create canonical v1, v2, or hybrid metainfo from a file or directory' \
'inspect:Inspect validated metainfo without reading payload files' \
'validate:Validate metainfo structure without reading payload files' \
'verify:Verify payload files against metainfo hashes' \
'edit:Edit metainfo metadata without reading payload files' \
'magnet:Print a deterministic magnet URI' \
'config:Locate, inspect, validate, and update configuration' \
'completion:Generate, install, or uninstall shell completions' \
'completions:Deprecated alias for \`btpc completion generate\`' \
'manpage:Generate the btpc(1) manual page on stdout' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'btpc help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__completion_commands] )) ||
_btpc__subcmd__help__subcmd__completion_commands() {
    local commands; commands=(
'generate:Generate a shell completion script on stdout' \
'install:Install shell completions in the standard per-user directory' \
'uninstall:Remove BTPC-generated shell completions' \
    )
    _describe -t commands 'btpc help completion commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__completion__subcmd__generate_commands] )) ||
_btpc__subcmd__help__subcmd__completion__subcmd__generate_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help completion generate commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__completion__subcmd__install_commands] )) ||
_btpc__subcmd__help__subcmd__completion__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help completion install commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__completion__subcmd__uninstall_commands] )) ||
_btpc__subcmd__help__subcmd__completion__subcmd__uninstall_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help completion uninstall commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__completions_commands] )) ||
_btpc__subcmd__help__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help completions commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config_commands] )) ||
_btpc__subcmd__help__subcmd__config_commands() {
    local commands; commands=(
'path:Print the selected configuration path' \
'init:Create a minimal configuration file' \
'show:Print configuration with secrets redacted by default' \
'check:Validate schema, references, cycles, conflicts, and permissions' \
'explain:Explain resolved command values without executing the command' \
'tracker:Manage named tracker aliases' \
'preset:Manage named creation presets' \
    )
    _describe -t commands 'btpc help config commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__check_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__check_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config check commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__explain_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__explain_commands() {
    local commands; commands=(
'create:Explain effective create values and provenance' \
    )
    _describe -t commands 'btpc help config explain commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__explain__subcmd__create_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__explain__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config explain create commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__init_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config init commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__path_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__path_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config path commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__preset_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__preset_commands() {
    local commands; commands=(
'list:List preset names' \
'show:Show one preset' \
'save:Save or replace a preset' \
'remove:Remove a preset' \
    )
    _describe -t commands 'btpc help config preset commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__list_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config preset list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__remove_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config preset remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__save_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__save_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config preset save commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__show_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config preset show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__show_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__show_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config show commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__tracker_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__tracker_commands() {
    local commands; commands=(
'list:List tracker aliases' \
'add:Add or replace a tracker alias' \
'remove:Remove a tracker alias' \
    )
    _describe -t commands 'btpc help config tracker commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__add_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config tracker add commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__list_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config tracker list commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__remove_commands] )) ||
_btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help config tracker remove commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__create_commands] )) ||
_btpc__subcmd__help__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help create commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__edit_commands] )) ||
_btpc__subcmd__help__subcmd__edit_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help edit commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__help_commands] )) ||
_btpc__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help help commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__inspect_commands] )) ||
_btpc__subcmd__help__subcmd__inspect_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help inspect commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__magnet_commands] )) ||
_btpc__subcmd__help__subcmd__magnet_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help magnet commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__manpage_commands] )) ||
_btpc__subcmd__help__subcmd__manpage_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help manpage commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__validate_commands] )) ||
_btpc__subcmd__help__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help validate commands' commands "$@"
}
(( $+functions[_btpc__subcmd__help__subcmd__verify_commands] )) ||
_btpc__subcmd__help__subcmd__verify_commands() {
    local commands; commands=()
    _describe -t commands 'btpc help verify commands' commands "$@"
}
(( $+functions[_btpc__subcmd__inspect_commands] )) ||
_btpc__subcmd__inspect_commands() {
    local commands; commands=()
    _describe -t commands 'btpc inspect commands' commands "$@"
}
(( $+functions[_btpc__subcmd__magnet_commands] )) ||
_btpc__subcmd__magnet_commands() {
    local commands; commands=()
    _describe -t commands 'btpc magnet commands' commands "$@"
}
(( $+functions[_btpc__subcmd__manpage_commands] )) ||
_btpc__subcmd__manpage_commands() {
    local commands; commands=()
    _describe -t commands 'btpc manpage commands' commands "$@"
}
(( $+functions[_btpc__subcmd__validate_commands] )) ||
_btpc__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'btpc validate commands' commands "$@"
}
(( $+functions[_btpc__subcmd__verify_commands] )) ||
_btpc__subcmd__verify_commands() {
    local commands; commands=()
    _describe -t commands 'btpc verify commands' commands "$@"
}

if [ "$funcstack[1]" = "_btpc" ]; then
    _btpc "$@"
else
    compdef _btpc btpc
fi
