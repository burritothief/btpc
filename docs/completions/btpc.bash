_btpc() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="btpc"
                ;;
            btpc,completion)
                cmd="btpc__subcmd__completion"
                ;;
            btpc,completions)
                cmd="btpc__subcmd__completions"
                ;;
            btpc,config)
                cmd="btpc__subcmd__config"
                ;;
            btpc,create)
                cmd="btpc__subcmd__create"
                ;;
            btpc,edit)
                cmd="btpc__subcmd__edit"
                ;;
            btpc,help)
                cmd="btpc__subcmd__help"
                ;;
            btpc,inspect)
                cmd="btpc__subcmd__inspect"
                ;;
            btpc,magnet)
                cmd="btpc__subcmd__magnet"
                ;;
            btpc,manpage)
                cmd="btpc__subcmd__manpage"
                ;;
            btpc,validate)
                cmd="btpc__subcmd__validate"
                ;;
            btpc,verify)
                cmd="btpc__subcmd__verify"
                ;;
            btpc__subcmd__completion,generate)
                cmd="btpc__subcmd__completion__subcmd__generate"
                ;;
            btpc__subcmd__completion,help)
                cmd="btpc__subcmd__completion__subcmd__help"
                ;;
            btpc__subcmd__completion,install)
                cmd="btpc__subcmd__completion__subcmd__install"
                ;;
            btpc__subcmd__completion,uninstall)
                cmd="btpc__subcmd__completion__subcmd__uninstall"
                ;;
            btpc__subcmd__completion__subcmd__help,generate)
                cmd="btpc__subcmd__completion__subcmd__help__subcmd__generate"
                ;;
            btpc__subcmd__completion__subcmd__help,help)
                cmd="btpc__subcmd__completion__subcmd__help__subcmd__help"
                ;;
            btpc__subcmd__completion__subcmd__help,install)
                cmd="btpc__subcmd__completion__subcmd__help__subcmd__install"
                ;;
            btpc__subcmd__completion__subcmd__help,uninstall)
                cmd="btpc__subcmd__completion__subcmd__help__subcmd__uninstall"
                ;;
            btpc__subcmd__config,check)
                cmd="btpc__subcmd__config__subcmd__check"
                ;;
            btpc__subcmd__config,explain)
                cmd="btpc__subcmd__config__subcmd__explain"
                ;;
            btpc__subcmd__config,help)
                cmd="btpc__subcmd__config__subcmd__help"
                ;;
            btpc__subcmd__config,init)
                cmd="btpc__subcmd__config__subcmd__init"
                ;;
            btpc__subcmd__config,path)
                cmd="btpc__subcmd__config__subcmd__path"
                ;;
            btpc__subcmd__config,preset)
                cmd="btpc__subcmd__config__subcmd__preset"
                ;;
            btpc__subcmd__config,show)
                cmd="btpc__subcmd__config__subcmd__show"
                ;;
            btpc__subcmd__config,tracker)
                cmd="btpc__subcmd__config__subcmd__tracker"
                ;;
            btpc__subcmd__config__subcmd__explain,create)
                cmd="btpc__subcmd__config__subcmd__explain__subcmd__create"
                ;;
            btpc__subcmd__config__subcmd__explain,help)
                cmd="btpc__subcmd__config__subcmd__explain__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__explain__subcmd__help,create)
                cmd="btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__create"
                ;;
            btpc__subcmd__config__subcmd__explain__subcmd__help,help)
                cmd="btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__help,check)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__check"
                ;;
            btpc__subcmd__config__subcmd__help,explain)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__explain"
                ;;
            btpc__subcmd__config__subcmd__help,help)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__help,init)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__init"
                ;;
            btpc__subcmd__config__subcmd__help,path)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__path"
                ;;
            btpc__subcmd__config__subcmd__help,preset)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__preset"
                ;;
            btpc__subcmd__config__subcmd__help,show)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__show"
                ;;
            btpc__subcmd__config__subcmd__help,tracker)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__tracker"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__explain,create)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__explain__subcmd__create"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__preset,list)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__list"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__preset,remove)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__remove"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__preset,save)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__save"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__preset,show)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__show"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__tracker,add)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__add"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__tracker,list)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__list"
                ;;
            btpc__subcmd__config__subcmd__help__subcmd__tracker,remove)
                cmd="btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__remove"
                ;;
            btpc__subcmd__config__subcmd__preset,help)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__preset,list)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__list"
                ;;
            btpc__subcmd__config__subcmd__preset,remove)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__remove"
                ;;
            btpc__subcmd__config__subcmd__preset,save)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__save"
                ;;
            btpc__subcmd__config__subcmd__preset,show)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__show"
                ;;
            btpc__subcmd__config__subcmd__preset__subcmd__help,help)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__preset__subcmd__help,list)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__list"
                ;;
            btpc__subcmd__config__subcmd__preset__subcmd__help,remove)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__remove"
                ;;
            btpc__subcmd__config__subcmd__preset__subcmd__help,save)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__save"
                ;;
            btpc__subcmd__config__subcmd__preset__subcmd__help,show)
                cmd="btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__show"
                ;;
            btpc__subcmd__config__subcmd__tracker,add)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__add"
                ;;
            btpc__subcmd__config__subcmd__tracker,help)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__tracker,list)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__list"
                ;;
            btpc__subcmd__config__subcmd__tracker,remove)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__remove"
                ;;
            btpc__subcmd__config__subcmd__tracker__subcmd__help,add)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__add"
                ;;
            btpc__subcmd__config__subcmd__tracker__subcmd__help,help)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__help"
                ;;
            btpc__subcmd__config__subcmd__tracker__subcmd__help,list)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__list"
                ;;
            btpc__subcmd__config__subcmd__tracker__subcmd__help,remove)
                cmd="btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__remove"
                ;;
            btpc__subcmd__help,completion)
                cmd="btpc__subcmd__help__subcmd__completion"
                ;;
            btpc__subcmd__help,completions)
                cmd="btpc__subcmd__help__subcmd__completions"
                ;;
            btpc__subcmd__help,config)
                cmd="btpc__subcmd__help__subcmd__config"
                ;;
            btpc__subcmd__help,create)
                cmd="btpc__subcmd__help__subcmd__create"
                ;;
            btpc__subcmd__help,edit)
                cmd="btpc__subcmd__help__subcmd__edit"
                ;;
            btpc__subcmd__help,help)
                cmd="btpc__subcmd__help__subcmd__help"
                ;;
            btpc__subcmd__help,inspect)
                cmd="btpc__subcmd__help__subcmd__inspect"
                ;;
            btpc__subcmd__help,magnet)
                cmd="btpc__subcmd__help__subcmd__magnet"
                ;;
            btpc__subcmd__help,manpage)
                cmd="btpc__subcmd__help__subcmd__manpage"
                ;;
            btpc__subcmd__help,validate)
                cmd="btpc__subcmd__help__subcmd__validate"
                ;;
            btpc__subcmd__help,verify)
                cmd="btpc__subcmd__help__subcmd__verify"
                ;;
            btpc__subcmd__help__subcmd__completion,generate)
                cmd="btpc__subcmd__help__subcmd__completion__subcmd__generate"
                ;;
            btpc__subcmd__help__subcmd__completion,install)
                cmd="btpc__subcmd__help__subcmd__completion__subcmd__install"
                ;;
            btpc__subcmd__help__subcmd__completion,uninstall)
                cmd="btpc__subcmd__help__subcmd__completion__subcmd__uninstall"
                ;;
            btpc__subcmd__help__subcmd__config,check)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__check"
                ;;
            btpc__subcmd__help__subcmd__config,explain)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__explain"
                ;;
            btpc__subcmd__help__subcmd__config,init)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__init"
                ;;
            btpc__subcmd__help__subcmd__config,path)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__path"
                ;;
            btpc__subcmd__help__subcmd__config,preset)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__preset"
                ;;
            btpc__subcmd__help__subcmd__config,show)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__show"
                ;;
            btpc__subcmd__help__subcmd__config,tracker)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__tracker"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__explain,create)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__explain__subcmd__create"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__preset,list)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__list"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__preset,remove)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__remove"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__preset,save)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__save"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__preset,show)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__show"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__tracker,add)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__add"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__tracker,list)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__list"
                ;;
            btpc__subcmd__help__subcmd__config__subcmd__tracker,remove)
                cmd="btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__remove"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        btpc)
            opts="-v -q -h -V --config --no-config --color --verbose --quiet --help --version create inspect validate verify edit magnet config completion completions manpage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help generate install uninstall help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__generate)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__help)
            opts="generate install uninstall help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__help__subcmd__generate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__help__subcmd__install)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__help__subcmd__uninstall)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__install)
            opts="-v -q -h --dry-run --force --config --no-config --color --verbose --quiet --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completion__subcmd__uninstall)
            opts="-v -q -h --dry-run --force --config --no-config --color --verbose --quiet --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__completions)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help path init show check explain tracker preset help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__check)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__explain)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help create help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__explain__subcmd__create)
            opts="-o -f -a -v -q -h --batch --mode --output --output-dir --jobs --fail-fast --force --durable --preset --piece-length --target-pieces --max-piece-length --tracker --clear-trackers --tracker-tier --tracker-alias --tracker-group --web-seed --clear-web-seeds --node --private --public --source --clear-source --comment --clear-comment --created-by --no-created-by --creation-date --entropy --name --exclude-hidden --symlinks --special-files --exclude-empty-files --reject-empty-directories --include --clear-includes --exclude --clear-excludes --threads --dry-run --print --json --pretty --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --batch)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "v1 v2 hybrid" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --jobs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --preset)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --piece-length)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-pieces)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-piece-length)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-tier)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-alias)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --web-seed)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --node)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --source)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --comment)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --created-by)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --creation-date)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --entropy)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --symlinks)
                    COMPREPLY=($(compgen -W "reject skip follow" -- "${cur}"))
                    return 0
                    ;;
                --special-files)
                    COMPREPLY=($(compgen -W "reject skip" -- "${cur}"))
                    return 0
                    ;;
                --include)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --print)
                    COMPREPLY=($(compgen -W "path info-hash-v1 info-hash-v2 magnet" -- "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__explain__subcmd__help)
            opts="create help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__explain__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help)
            opts="path init show check explain tracker preset help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__check)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__explain)
            opts="create"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__explain__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__init)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__path)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__preset)
            opts="list show save remove"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__save)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__preset__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__tracker)
            opts="list add remove"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__help__subcmd__tracker__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__init)
            opts="-v -q -h --force --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__path)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help list show save remove help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__help)
            opts="list show save remove help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__save)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__help__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__list)
            opts="-v -q -h --json --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__remove)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__save)
            opts="-v -q -h --extends --mode --piece-length --private --source --comment --created-by --creation-date --name --exclude-hidden --symlinks --special-files --exclude-empty-files --reject-empty-directories --tracker --tracker-alias --tracker-group --web-seed --include --exclude --threads --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --extends)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "v1 v2 hybrid" -- "${cur}"))
                    return 0
                    ;;
                --piece-length)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --source)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --comment)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --created-by)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --creation-date)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --symlinks)
                    COMPREPLY=($(compgen -W "reject skip follow" -- "${cur}"))
                    return 0
                    ;;
                --special-files)
                    COMPREPLY=($(compgen -W "reject skip" -- "${cur}"))
                    return 0
                    ;;
                --tracker)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-alias)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --web-seed)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --include)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__preset__subcmd__show)
            opts="-v -q -h --show-secrets --json --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__show)
            opts="-v -q -h --resolved --show-secrets --json --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help list add remove help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__add)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__help)
            opts="list add remove help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__help__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__list)
            opts="-v -q -h --show-secrets --json --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__config__subcmd__tracker__subcmd__remove)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__create)
            opts="-o -f -a -v -q -h --batch --mode --output --output-dir --jobs --fail-fast --force --durable --preset --piece-length --target-pieces --max-piece-length --tracker --clear-trackers --tracker-tier --tracker-alias --tracker-group --web-seed --clear-web-seeds --node --private --public --source --clear-source --comment --clear-comment --created-by --no-created-by --creation-date --entropy --name --exclude-hidden --symlinks --special-files --exclude-empty-files --reject-empty-directories --include --clear-includes --exclude --clear-excludes --threads --dry-run --print --json --pretty --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --batch)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "v1 v2 hybrid" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --jobs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --preset)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --piece-length)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-pieces)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-piece-length)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-tier)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-alias)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --web-seed)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --node)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --source)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --comment)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --created-by)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --creation-date)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --entropy)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --symlinks)
                    COMPREPLY=($(compgen -W "reject skip follow" -- "${cur}"))
                    return 0
                    ;;
                --special-files)
                    COMPREPLY=($(compgen -W "reject skip" -- "${cur}"))
                    return 0
                    ;;
                --include)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --exclude)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --threads)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --print)
                    COMPREPLY=($(compgen -W "path info-hash-v1 info-hash-v2 magnet" -- "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__edit)
            opts="-o -f -a -v -q -h --output --in-place --force --durable --dry-run --diff --json --tracker --tracker-alias --tracker-group --clear-trackers --web-seed --clear-web-seeds --node --clear-nodes --comment --clear-comment --created-by --clear-created-by --creation-date --clear-creation-date --private --public --clear-private --source --clear-source --file-attributes --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-alias)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --tracker-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --web-seed)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --node)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --comment)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --created-by)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --creation-date)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --source)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --file-attributes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help)
            opts="create inspect validate verify edit magnet config completion completions manpage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__completion)
            opts="generate install uninstall"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__completion__subcmd__generate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__completion__subcmd__install)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__completion__subcmd__uninstall)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config)
            opts="path init show check explain tracker preset"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__check)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__explain)
            opts="create"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__explain__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__init)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__path)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__preset)
            opts="list show save remove"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__save)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__preset__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__tracker)
            opts="list add remove"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__add)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__config__subcmd__tracker__subcmd__remove)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__create)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__edit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__inspect)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__magnet)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__manpage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__help__subcmd__verify)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__inspect)
            opts="-v -q -h --json --field --files --tree --path-encoding --offset --limit --format --pretty --max-input-bytes --max-owned-bytes --max-integer-digits --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --field)
                    COMPREPLY=($(compgen -W "mode name total-size piece-length piece-count file-count hash-v1 hash-v2 private trackers web-seeds nodes comment creator creation-date source canonicality warnings files unknown-fields" -- "${cur}"))
                    return 0
                    ;;
                --path-encoding)
                    COMPREPLY=($(compgen -W "utf8 escaped hex" -- "${cur}"))
                    return 0
                    ;;
                --offset)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --format)
                    COMPREPLY=($(compgen -W "human plain json json-pretty tsv" -- "${cur}"))
                    return 0
                    ;;
                --max-input-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-owned-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-integer-digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__magnet)
            opts="-v -q -h --no-display-name --no-trackers --no-web-seeds --max-input-bytes --max-owned-bytes --max-integer-digits --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-input-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-owned-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-integer-digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__manpage)
            opts="-v -q -h --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__validate)
            opts="-v -q -h --json --format --canonical --warnings-as-errors --pretty --max-input-bytes --max-owned-bytes --max-integer-digits --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "human json json-pretty" -- "${cur}"))
                    return 0
                    ;;
                --max-input-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-owned-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-integer-digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        btpc__subcmd__verify)
            opts="-v -q -h --fail-fast --extra-files --json --pretty --max-input-bytes --max-owned-bytes --max-integer-digits --config --no-config --color --verbose --quiet --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --max-input-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-owned-bytes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-integer-digits)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "auto always never" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _btpc -o nosort -o bashdefault -o default btpc
else
    complete -F _btpc -o bashdefault -o default btpc
fi
