# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_btpc_global_optspecs
    string join \n config= no-config color= v/verbose q/quiet h/help V/version
end

function __fish_btpc_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_btpc_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_btpc_using_subcommand
    set -l cmd (__fish_btpc_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c btpc -n "__fish_btpc_needs_command" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_needs_command" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_needs_command" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_needs_command" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_needs_command" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_needs_command" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_needs_command" -s V -l version -d 'Print version'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "create" -d 'Create canonical v1, v2, or hybrid metainfo from a file or directory'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "inspect" -d 'Inspect validated metainfo without reading payload files'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "validate" -d 'Validate metainfo structure without reading payload files'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "verify" -d 'Verify payload files against metainfo hashes'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "edit" -d 'Edit metainfo metadata without reading payload files'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "magnet" -d 'Print a deterministic magnet URI'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "config" -d 'Locate, inspect, validate, and update configuration'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "completion" -d 'Generate, install, or uninstall shell completions'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "completions" -d 'Deprecated alias for `btpc completion generate`'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "manpage" -d 'Generate the btpc(1) manual page on stdout'
complete -c btpc -n "__fish_btpc_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l batch -d 'Versioned TOML batch manifest' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand create" -l mode -d 'Torrent protocol representation' -r -f -a "v1\t''
v2\t''
hybrid\t''"
complete -c btpc -n "__fish_btpc_using_subcommand create" -s o -l output -d 'Destination .torrent path (defaults beside the payload)' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand create" -l output-dir -d 'Write batch outputs beneath this directory' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand create" -l jobs -d 'Maximum concurrent batch creation jobs' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l preset -d 'Apply a named creation preset; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l piece-length -d 'Explicit piece length in bytes' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l target-pieces -d 'Target approximate number of pieces for automatic selection' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l max-piece-length -d 'Cap target-based automatic piece length' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -s a -l tracker -d 'Add a tracker as its own tier; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l tracker-tier -d 'Add one comma-separated tracker tier; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l tracker-alias -d 'Add a configured tracker alias; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l tracker-group -d 'Add a configured tracker group; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l web-seed -d 'Add a web seed URL; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l node -d 'Add a DHT node as HOST:PORT; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l source -d 'Set the source field' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l comment -d 'Set the top-level comment' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l created-by -d 'Set the creator string' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l creation-date -d 'Include an explicit Unix creation timestamp' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l entropy -d 'Set deterministic, random, or omitted entropy policy' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l name -d 'Override the torrent root name' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l symlinks -d 'Symbolic-link policy' -r -f -a "reject\t''
skip\t''
follow\t''"
complete -c btpc -n "__fish_btpc_using_subcommand create" -l special-files -d 'Special-file policy' -r -f -a "reject\t''
skip\t''"
complete -c btpc -n "__fish_btpc_using_subcommand create" -l include -d 'Include only paths matching this glob; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l exclude -d 'Exclude paths matching this glob; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l threads -d 'v1 hashing threads; 0 selects a conservative automatic count, 1 is sequential' -r
complete -c btpc -n "__fish_btpc_using_subcommand create" -l print -d 'Print selected result fields; may be repeated' -r -f -a "path\t''
info-hash-v1\t''
info-hash-v2\t''
magnet\t''"
complete -c btpc -n "__fish_btpc_using_subcommand create" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand create" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand create" -l fail-fast -d 'Stop scheduling batch jobs after the first failure'
complete -c btpc -n "__fish_btpc_using_subcommand create" -s f -l force -d 'Replace an existing destination'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l durable -d 'Sync the destination directory after atomic publication where supported'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l clear-trackers -d 'Clear configured and preset trackers before CLI additions'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l clear-web-seeds -d 'Clear configured and preset web seeds before CLI additions'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l private -d 'Set the private flag'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l public -d 'Set the private flag to false'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l clear-source -d 'Remove configured or preset source metadata'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l clear-comment -d 'Remove configured or preset comment metadata'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l no-created-by -d 'Omit the creator string instead of using the versioned default'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l exclude-hidden -d 'Exclude dot-prefixed files and directories'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l exclude-empty-files -d 'Exclude zero-length files'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l reject-empty-directories -d 'Reject empty directories instead of ignoring them'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l clear-includes -d 'Clear configured and preset include patterns before CLI additions'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l clear-excludes -d 'Clear configured and preset exclude patterns before CLI additions'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l dry-run -d 'Plan creation without hashing or writing metainfo'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l json -d 'Emit a versioned JSON result to stdout'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l pretty -d 'Use the expanded human completion renderer'
complete -c btpc -n "__fish_btpc_using_subcommand create" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand create" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand create" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand create" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l field -d 'Select a field; may be repeated' -r -f -a "mode\t''
name\t''
total-size\t''
piece-length\t''
piece-count\t''
file-count\t''
hash-v1\t''
hash-v2\t''
private\t''
trackers\t''
web-seeds\t''
nodes\t''
comment\t''
creator\t''
creation-date\t''
source\t''
canonicality\t''
warnings\t''
files\t''
unknown-fields\t''"
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l path-encoding -d 'Encode raw torrent paths' -r -f -a "utf8\t''
escaped\t''
hex\t''"
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l offset -d 'Skip this many file rows' -r
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l limit -d 'Limit returned file rows' -r
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l format -d 'Select output representation' -r -f -a "human\t''
plain\t''
json\t''
json-pretty\t''
tsv\t''"
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l max-input-bytes -d 'Maximum metainfo bytes accepted while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l max-owned-bytes -d 'Maximum estimated owned allocation while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l max-integer-digits -d 'Maximum decimal digits accepted in one bencode integer' -r
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l json -d 'Emit versioned JSON to stdout'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l files -d 'Include the flat file listing'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l tree -d 'Render files as a deterministic tree'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l pretty -d 'Use the expanded human renderer'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand inspect" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l format -d 'Select output representation' -r -f -a "human\t''
json\t''
json-pretty\t''"
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l max-input-bytes -d 'Maximum metainfo bytes accepted while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l max-owned-bytes -d 'Maximum estimated owned allocation while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l max-integer-digits -d 'Maximum decimal digits accepted in one bencode integer' -r
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l json -d 'Emit versioned JSON to stdout'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l canonical -d 'Require canonical bencode'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l warnings-as-errors -d 'Return the warning exit code when validation warnings exist'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l pretty -d 'Use the expanded human renderer'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand validate" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l max-input-bytes -d 'Maximum metainfo bytes accepted while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l max-owned-bytes -d 'Maximum estimated owned allocation while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l max-integer-digits -d 'Maximum decimal digits accepted in one bencode integer' -r
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l fail-fast -d 'Stop after the first deterministic mismatch'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l extra-files -d 'Report regular files absent from metainfo'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l json -d 'Emit versioned JSON to stdout'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l pretty -d 'Use the expanded human renderer'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand verify" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -s o -l output -d 'Write the edited metainfo to this path' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand edit" -s a -l tracker -d 'Replace trackers with this tracker tier; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l tracker-alias -d 'Add a configured tracker alias; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l tracker-group -d 'Add a configured tracker group; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l web-seed -d 'Replace web seeds with this URL; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l node -d 'Replace DHT nodes with HOST:PORT; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l comment -d 'Set the top-level comment' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l created-by -d 'Set the creator string' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l creation-date -d 'Set the Unix creation timestamp' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l source -d 'Set the source field' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l file-attributes -d 'Set file attributes as PATH=ATTRS; may be repeated' -r
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l in-place -d 'Replace the input file atomically'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -s f -l force -d 'Replace an existing output file'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l durable -d 'Sync the destination directory after publication where supported'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l dry-run -d 'Validate and report changes without writing output'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l diff -d 'Print a deterministic field-level change summary'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l json -d 'Emit a versioned JSON result to stdout'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-trackers -d 'Remove all trackers'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-web-seeds -d 'Remove all web seeds'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-nodes -d 'Remove all DHT nodes'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-comment -d 'Remove the top-level comment'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-created-by -d 'Remove the creator string'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-creation-date -d 'Remove the creation timestamp'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l private -d 'Set the private flag'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l public -d 'Set the private flag to false'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-private -d 'Remove the private field'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l clear-source -d 'Remove the source field'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand edit" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l max-input-bytes -d 'Maximum metainfo bytes accepted while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l max-owned-bytes -d 'Maximum estimated owned allocation while loading' -r
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l max-integer-digits -d 'Maximum decimal digits accepted in one bencode integer' -r
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l no-display-name -d 'Omit the display name parameter'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l no-trackers -d 'Omit tracker parameters'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l no-web-seeds -d 'Omit web seed parameters'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand magnet" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "path" -d 'Print the selected configuration path'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "init" -d 'Create a minimal configuration file'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "show" -d 'Print configuration with secrets redacted by default'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "check" -d 'Validate schema, references, cycles, conflicts, and permissions'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "explain" -d 'Explain resolved command values without executing the command'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "tracker" -d 'Manage named tracker aliases'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "preset" -d 'Manage named creation presets'
complete -c btpc -n "__fish_btpc_using_subcommand config; and not __fish_seen_subcommand_from path init show check explain tracker preset help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from path" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from path" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from path" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from path" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from path" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from path" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -l force -d 'Replace an existing file'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -l resolved -d 'Validate and print the parsed deterministic representation'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -l show-secrets -d 'Reveal configured secrets'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -l json -d 'Emit JSON instead of TOML'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from check" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from check" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from check" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from check" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from check" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from check" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -f -a "create" -d 'Explain effective create values and provenance'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from explain" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -f -a "list" -d 'List tracker aliases'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -f -a "add" -d 'Add or replace a tracker alias'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -f -a "remove" -d 'Remove a tracker alias'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from tracker" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -f -a "list" -d 'List preset names'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -f -a "show" -d 'Show one preset'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -f -a "save" -d 'Save or replace a preset'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -f -a "remove" -d 'Remove a preset'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from preset" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "path" -d 'Print the selected configuration path'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "init" -d 'Create a minimal configuration file'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "show" -d 'Print configuration with secrets redacted by default'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "check" -d 'Validate schema, references, cycles, conflicts, and permissions'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "explain" -d 'Explain resolved command values without executing the command'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "tracker" -d 'Manage named tracker aliases'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "preset" -d 'Manage named creation presets'
complete -c btpc -n "__fish_btpc_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -f -a "generate" -d 'Generate a shell completion script on stdout'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -f -a "install" -d 'Install shell completions in the standard per-user directory'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -f -a "uninstall" -d 'Remove BTPC-generated shell completions'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and not __fish_seen_subcommand_from generate install uninstall help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from generate" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from generate" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from generate" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from generate" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from generate" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from generate" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -l dry-run -d 'Print the target and generated content without changing files'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -l force -d 'Replace an unrelated existing completion file'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from install" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -l dry-run -d 'Print the target and generated content without changing files'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -l force -d 'Replace an unrelated existing completion file'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from uninstall" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from help" -f -a "generate" -d 'Generate a shell completion script on stdout'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from help" -f -a "install" -d 'Install shell completions in the standard per-user directory'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from help" -f -a "uninstall" -d 'Remove BTPC-generated shell completions'
complete -c btpc -n "__fish_btpc_using_subcommand completion; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand completions" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand completions" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand completions" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand completions" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand completions" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand completions" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand manpage" -l config -d 'Use this configuration file when configuration loading is enabled' -r -F
complete -c btpc -n "__fish_btpc_using_subcommand manpage" -l color -d 'Control colored terminal output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c btpc -n "__fish_btpc_using_subcommand manpage" -l no-config -d 'Disable implicit and environment-selected configuration'
complete -c btpc -n "__fish_btpc_using_subcommand manpage" -s v -l verbose -d 'Increase diagnostic verbosity; may be repeated'
complete -c btpc -n "__fish_btpc_using_subcommand manpage" -s q -l quiet -d 'Suppress human summaries, warnings, and progress'
complete -c btpc -n "__fish_btpc_using_subcommand manpage" -s h -l help -d 'Print help'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "create" -d 'Create canonical v1, v2, or hybrid metainfo from a file or directory'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "inspect" -d 'Inspect validated metainfo without reading payload files'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "validate" -d 'Validate metainfo structure without reading payload files'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "verify" -d 'Verify payload files against metainfo hashes'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "edit" -d 'Edit metainfo metadata without reading payload files'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "magnet" -d 'Print a deterministic magnet URI'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "config" -d 'Locate, inspect, validate, and update configuration'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "completion" -d 'Generate, install, or uninstall shell completions'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "completions" -d 'Deprecated alias for `btpc completion generate`'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "manpage" -d 'Generate the btpc(1) manual page on stdout'
complete -c btpc -n "__fish_btpc_using_subcommand help; and not __fish_seen_subcommand_from create inspect validate verify edit magnet config completion completions manpage help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "path" -d 'Print the selected configuration path'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "init" -d 'Create a minimal configuration file'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "show" -d 'Print configuration with secrets redacted by default'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "check" -d 'Validate schema, references, cycles, conflicts, and permissions'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "explain" -d 'Explain resolved command values without executing the command'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "tracker" -d 'Manage named tracker aliases'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "preset" -d 'Manage named creation presets'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from completion" -f -a "generate" -d 'Generate a shell completion script on stdout'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from completion" -f -a "install" -d 'Install shell completions in the standard per-user directory'
complete -c btpc -n "__fish_btpc_using_subcommand help; and __fish_seen_subcommand_from completion" -f -a "uninstall" -d 'Remove BTPC-generated shell completions'
